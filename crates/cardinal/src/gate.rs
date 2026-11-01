//! The native git `pre-commit` hook that runs the review gate of the project (`specs/sync.md`, SYN-18 to SYN-23).
//!
//! The hook is a small `sh` script that starts `cardinal run review-gate native` (the tool of the revision the project
//! pins, `specs/tool-binaries.md`), then the `pre-commit` the user already had (moved to `pre-commit.user`, never
//! overwritten). Once the source no longer manages the gate, `sync` removes it and moves that hook back (SYN-60).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{bail, Result};

use crate::entries::Entry;
use crate::{is_executable, set_executable, write, Outcome};

#[cfg(unix)]
use std::os::unix::fs::symlink;

/// The command of the hooks that run the review gate: the gate exists only where Cardinal manages such a hook, or the
/// review hook script of an older revision (`LEGACY_HOOK`).
const REVIEW_TOOL: &str = "cardinal run review-gate";
/// The review hook of the revisions before the tools moved into `harness`, among their managed files.
pub const LEGACY_HOOK: &str = ".claude/hooks/require_code_review.rs";
/// Name of the gate in a report.
pub const GATE: &str = "native review gate";
/// The line that tells the installed hook from one of the user, written by every revision; it keeps the old name
/// because an older `cardinal` that took the gate for the user's would move it to `pre-commit.user`, which would then
/// run itself in a loop. The second is read too: two revisions wrote it.
const MARKS: [&str; 2] = [
    "# harness: native review gate",
    "# cardinal: native review gate",
];
/// Where a `pre-commit` of the user goes, and is run from.
const CHAINED: &str = "pre-commit.user";

/// The managed hook. Git runs it from the top of the work tree, which may be of an older revision whose review gate
/// is still the `rust-script` hook. A work tree without either, a machine without the program it needs, or a launcher
/// that cannot run the tool (exit 3, BIN-14) is a warning, not a refusal, so a branch or a machine without Cardinal
/// can still commit (BIN-9).
const SCRIPT: &str = r#"#!/bin/sh
# harness: native review gate (installed by `cardinal sync`, do not edit)
legacy=.claude/hooks/require_code_review.rs
if [ -f "$legacy" ] && command -v rust-script >/dev/null 2>&1; then
    rust-script "$legacy" native || exit $?
elif [ ! -f "$legacy" ] && [ -f .claude/cardinal.toml ] && cardinal help run >/dev/null 2>&1; then
    cardinal run review-gate native
    code=$?
    if [ "$code" -eq 3 ]; then
        echo "cardinal: review gate skipped, cardinal could not run it" >&2
    elif [ "$code" -ne 0 ]; then
        exit "$code"
    fi
else
    echo "cardinal: review gate skipped, its hook or the program to run it (rust-script, or cardinal) is missing" >&2
fi
case "$0" in */*) dir="${0%/*}" ;; *) dir=. ;; esac
user="$dir/pre-commit.user"
if [ -x "$user" ]; then
    exec "$user" "$@"
fi
exit 0
"#;

/// What the hooks directory holds as `pre-commit`.
#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Missing,
    /// A `pre-commit` that is not ours.
    Foreign,
    /// Ours, but edited or no longer executable.
    Modified,
    UpToDate,
    /// `core.hooksPath` sends git to another directory: the gate is not installed there.
    Redirected,
}

/// What `sync` will do about the gate, decided before anything is written.
#[derive(Debug)]
pub struct Plan {
    hooks: PathBuf,
    state: State,
    /// Whether the gate is wanted: if not, ours is removed.
    wanted: bool,
    /// Whether a hook of the user is chained to the gate, read before the gate is written or removed.
    chained: bool,
}

/// The output of `git args` run in `dir`, `None` when it fails. The repository is the project's, not the one a hook or an
/// alias that runs `sync` is in.
fn git_in(dir: &Path, args: &[&str]) -> Option<String> {
    let out = git_raw(dir, args)?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_raw(dir: &Path, args: &[&str]) -> Option<Output> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .ok()
}

/// Whether `core.hooksPath` is set for the repository of `root` (`git config` exits 1 for an unset key).
fn hooks_path_set(root: &Path) -> bool {
    // any other failure (an unreadable config) is treated as set: better no gate than one in the wrong directory
    git_raw(root, &["config", "--get", "core.hooksPath"])
        .is_none_or(|out| out.status.code() != Some(1))
}

/// The hooks directory of the git repository of `root`, `None` when `root` is not the top of one.
fn hooks_dir(root: &Path) -> Option<PathBuf> {
    let git = |args: &[&str]| git_in(root, args);
    let top = fs::canonicalize(git(&["rev-parse", "--show-toplevel"])?).ok()?;
    if top != fs::canonicalize(root).ok()? {
        return None;
    }
    // for a linked worktree this is the hooks directory of the common git directory
    Some(root.join(git(&["rev-parse", "--git-path", "hooks"])?))
}

pub fn state(hooks: &Path) -> State {
    let path = hooks.join("pre-commit");
    match fs::read_to_string(&path) {
        Err(_) if fs::symlink_metadata(&path).is_err() => State::Missing,
        // the mark is the second line, so a hook of the user that only quotes it is not taken for ours
        Ok(text)
            if text
                .lines()
                .nth(1)
                .is_some_and(|l| MARKS.iter().any(|mark| l.starts_with(mark))) =>
        {
            if text == SCRIPT && is_executable(&path) {
                State::UpToDate
            } else {
                State::Modified
            }
        }
        _ => State::Foreign,
    }
}

/// Whether Cardinal manages the review gate: a managed hook of `entries` runs it, or the managed `files` hold the
/// review hook of an older revision.
#[must_use]
pub fn manages<'a>(entries: &[Entry], mut files: impl Iterator<Item = &'a String>) -> bool {
    entries.iter().any(|e| e.label().contains(REVIEW_TOOL)) || files.any(|f| f == LEGACY_HOOK)
}

/// The hooks directory of the project and what its `pre-commit` is, `None` when the project is not the top of a git
/// repository. One decision for `sync` and `check`, so they never disagree about where the gate goes.
fn inspect(root: &Path) -> Option<(PathBuf, State)> {
    let hooks = hooks_dir(root)?;
    let state = if hooks_path_set(root) {
        State::Redirected
    } else {
        state(&hooks)
    };
    Some((hooks, state))
}

/// The line `check` prints when the project should have the gate and does not (or not as managed), `None` when it is
/// there, cannot be installed (`core.hooksPath`, no repository) or is not expected. `expected` tells whether the lock
/// records a managed hook of the review gate, whether or not the settings still hold it.
#[must_use]
pub fn problem(root: &Path, expected: bool) -> Option<String> {
    if !expected {
        return None;
    }
    match inspect(root)?.1 {
        State::Missing | State::Foreign => Some(format!("missing: {GATE}")),
        State::Modified => Some(format!("modified: {GATE}")),
        State::UpToDate | State::Redirected => None,
    }
}

/// Decide what to do for the project at `root`, whose source manages a hook of the review gate when `has_review_hook`:
/// install the gate, or remove the one installed before when the source no longer manages it (SYN-60).
///
/// # Errors
/// When a `pre-commit` of the user must move to `pre-commit.user` and that file exists already.
pub fn plan(root: &Path, has_review_hook: bool) -> Result<Option<Plan>> {
    let Some((hooks, state)) = inspect(root) else {
        return Ok(None);
    };
    if !has_review_hook {
        // a gate left in place would refuse every commit, nothing recording a review any more
        let ours = matches!(state, State::Modified | State::UpToDate);
        let chained = fs::symlink_metadata(hooks.join(CHAINED)).is_ok();
        return Ok(ours.then_some(Plan {
            hooks,
            state,
            wanted: false,
            chained,
        }));
    }
    if state == State::Foreign && hooks.join(CHAINED).exists() {
        bail!(
            "{} and {} both exist: merge them by hand, `cardinal sync` never overwrites a hook of yours",
            hooks.join("pre-commit").display(),
            hooks.join(CHAINED).display()
        );
    }
    Ok(Some(Plan {
        hooks,
        state,
        wanted: true,
        chained: false,
    }))
}

impl Plan {
    /// What the report says, in order.
    #[must_use]
    pub fn outcomes(&self) -> Vec<(String, Outcome)> {
        let mut out = Vec::new();
        if !self.wanted {
            out.push((GATE.to_string(), Outcome::Removed));
            if self.chained {
                out.push(("pre-commit.user -> pre-commit".to_string(), Outcome::Moved));
            }
            return out;
        }
        if self.state == State::Redirected {
            let why = "not installed: core.hooksPath is set, so git reads its hooks elsewhere; add `cardinal run review-gate native` to your own pre-commit, run from the top of the work tree";
            return vec![(GATE.to_string(), Outcome::Skipped(why))];
        }
        if self.state == State::Foreign {
            out.push(("pre-commit -> pre-commit.user".to_string(), Outcome::Moved));
        }
        let outcome = match self.state {
            State::Missing | State::Foreign => Outcome::Added,
            State::Modified => Outcome::Updated,
            State::UpToDate | State::Redirected => Outcome::Unchanged,
        };
        out.push((GATE.to_string(), outcome));
        out
    }

    /// Write the hook, or remove it when it is no longer wanted.
    ///
    /// # Errors
    /// When a file cannot be moved, written or removed.
    pub fn apply(&self) -> Result<()> {
        if !self.wanted {
            return self.uninstall();
        }
        if matches!(self.state, State::UpToDate | State::Redirected) {
            return Ok(());
        }
        let hook = self.hooks.join("pre-commit");
        // the new hook is complete before the hook of the user moves, so a failure never leaves the repository
        // with no hook at all
        let staged = self.hooks.join("pre-commit.cardinal-new");
        write(&staged, SCRIPT.as_bytes())?;
        set_executable(&staged, true)?;
        if self.state == State::Foreign {
            let chained = self.hooks.join(CHAINED);
            if fs::symlink_metadata(&hook)?.is_symlink() {
                // a relative link would point elsewhere once moved: keep the target it reaches now
                let target = fs::canonicalize(&hook)?;
                fs::remove_file(&hook)?;
                symlink(&target, &chained)?;
            } else {
                fs::rename(&hook, &chained)?;
            }
        } else if self.state == State::Modified {
            // never write through a link
            fs::remove_file(&hook)?;
        }
        fs::rename(&staged, &hook)?;
        Ok(())
    }

    /// Remove the gate and put the hook of the user it chained back in its place, a link moved as a link.
    fn uninstall(&self) -> Result<()> {
        let hook = self.hooks.join("pre-commit");
        if self.chained {
            // one rename replaces the gate, so the repository is never left without the user's hook
            fs::rename(self.hooks.join(CHAINED), &hook)?;
        } else {
            fs::remove_file(&hook)?;
        }
        Ok(())
    }
}
