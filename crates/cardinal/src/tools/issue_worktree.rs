//! Give an issue its own git worktree and branch, so that two agents can work on two issues at once.
//! `start issues/<stem>.md` creates the worktree `<main worktree>/.claude/worktrees/<stem>`, on a new branch
//! `<stem>` cut from the branch of the main worktree, and makes git ignore `/.claude/worktrees/` (`info/exclude`).
//! `finish <stem> [--force]` removes that worktree and branch once the
//! branch is merged into the branch of the main worktree (ancestor, or every commit has a patch-equivalent there,
//! or a merge into it would change no file, which covers a rebase and a squash of any number of commits); `--force` also
//! accepts a branch whose work is not on it, after the user agreed, and still refuses a worktree with uncommitted
//! changes or holding another worktree. It also removes a worktree left at `<main worktree>-<stem>`, where `start` put
//! it before ADR 0008. `start` runs from any worktree; `finish` too, except from inside the worktree it would remove (it refuses and names the main worktree, `--force` or not).
//! Usage: `just start-issue issues/foo.md`, `just finish-issue foo` (`just finish-issue foo --force`).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn git_ok(dir: &Path, args: &[&str]) -> bool {
    git(dir, args).is_ok()
}

/// The main worktree and the branch it has checked out (`None` when detached).
fn main_worktree(cwd: &Path) -> Result<(PathBuf, Option<String>), String> {
    let list = git(cwd, &["worktree", "list", "--porcelain"])
        .map_err(|_| "not inside a git repository".to_string())?;
    let record = list.split("\n\n").next().unwrap_or_default();
    if record.lines().any(|l| l == "bare") {
        return Err("a bare repository has no main worktree".to_string());
    }
    let path = record
        .lines()
        .find_map(|l| l.strip_prefix("worktree "))
        .ok_or("no worktree found")?;
    let branch = record
        .lines()
        .find_map(|l| l.strip_prefix("branch refs/heads/"))
        .map(str::to_string);
    Ok((PathBuf::from(path), branch))
}

/// The path of the worktree that has `branch` checked out.
fn worktree_of(cwd: &Path, branch: &str) -> Option<PathBuf> {
    let list = git(cwd, &["worktree", "list", "--porcelain"]).ok()?;
    list.split("\n\n").find_map(|record| {
        let on = record
            .lines()
            .any(|l| l.strip_prefix("branch refs/heads/") == Some(branch));
        on.then(|| {
            record
                .lines()
                .find_map(|l| l.strip_prefix("worktree "))
                .map(PathBuf::from)
        })
        .flatten()
    })
}

/// The worktree of an issue: under `.claude/worktrees/`, the only place Claude Code enters without asking (ADR 0008).
fn nested(main: &Path, stem: &str) -> PathBuf {
    main.join(WORKTREES).join(stem)
}

/// Where `start` put the worktree of an issue before ADR 0008, still accepted by `finish`.
fn sibling(main: &Path, stem: &str) -> Option<PathBuf> {
    let name = main.file_name()?.to_string_lossy();
    Some(main.with_file_name(format!("{name}-{stem}")))
}

const WORKTREES: &str = ".claude/worktrees";

/// Make git ignore the nested worktrees in every worktree, through the `info/exclude` of the common directory: a
/// `.gitignore` belongs to the project, and this one may un-ignore `.claude/`.
fn exclude_worktrees(main: &Path) -> Result<(), String> {
    let common = PathBuf::from(git(
        main,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?);
    let exclude = common.join("info/exclude");
    let line = format!("/{WORKTREES}/");
    let fail = |e: std::io::Error| format!("cannot update {}: {e}", exclude.display());
    // bytes, not text: the user's patterns need not be UTF-8, and they are only ever appended to
    let bytes = match std::fs::read(&exclude) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        read => read.map_err(fail)?,
    };
    if bytes.split(|b| *b == b'\n').any(|l| l == line.as_bytes()) {
        return Ok(());
    }
    let separator = if bytes.is_empty() || bytes.ends_with(b"\n") {
        ""
    } else {
        "\n"
    };
    std::fs::create_dir_all(common.join("info")).map_err(fail)?;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&exclude)
        .map_err(fail)?;
    file.write_all(format!("{separator}{line}\n").as_bytes())
        .map_err(fail)
}

/// The worktrees nested in `dir`, such as an agent's: `git status` in `dir` does not see them, being ignored.
fn worktrees_inside(main: &Path, dir: &Path) -> Result<Vec<PathBuf>, String> {
    let list = git(main, &["worktree", "list", "--porcelain"])?;
    Ok(list
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .map(PathBuf::from)
        .filter(|w| w != dir && w.starts_with(dir))
        .collect())
}

fn valid_stem(stem: &str) -> bool {
    !stem.is_empty()
        && !stem.starts_with('-')
        && stem
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn start(cwd: &Path, issue: &str) -> Result<String, String> {
    let path = Path::new(issue);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if path.extension().is_none_or(|e| e != "md")
        || path
            .parent()
            .and_then(Path::file_name)
            .is_none_or(|d| d != "issues")
        || !valid_stem(stem)
    {
        return Err(format!(
            "{issue} is not an issue file directly under an `issues/` directory"
        ));
    }
    let (main, branch) = main_worktree(cwd)?;
    let base = branch.ok_or("the main worktree has no branch checked out")?;
    if git_ok(
        &main,
        &["rev-parse", "--verify", "-q", &format!("refs/heads/{stem}")],
    ) {
        return Err(format!(
            "the branch {stem} already exists: the issue is taken"
        ));
    }
    if !git_ok(
        &main,
        &["cat-file", "-e", &format!("{base}:issues/{stem}.md")],
    ) {
        return Err(format!(
            "{issue} is not committed on {base}: commit it first, the worktree would not hold it"
        ));
    }
    if !git(
        &main,
        &["status", "--porcelain", "--", &format!("issues/{stem}.md")],
    )?
    .is_empty()
    {
        return Err(format!("{issue} has uncommitted changes in the main worktree: commit them first, the worktree would not hold them"));
    }
    let dir = nested(&main, stem);
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    git(
        &main,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            stem,
            &dir.to_string_lossy(),
            &base,
        ],
    )?;
    exclude_worktrees(&main)?;
    // git ignores the personal configuration, so the checkout lacks it (PAR-17)
    let personal = main.join(crate::PERSONAL_CONFIG);
    if personal.is_file() {
        let copy = dir.join(crate::PERSONAL_CONFIG);
        std::fs::create_dir_all(copy.parent().unwrap())
            .and_then(|()| std::fs::copy(&personal, &copy))
            .map_err(|e| format!("cannot copy {}: {e}", personal.display()))?;
    }
    Ok(dir.display().to_string())
}

fn finish(cwd: &Path, stem: &str, force: bool) -> Result<String, String> {
    if !valid_stem(stem) {
        return Err(format!("not an issue stem: {stem}"));
    }
    let (main, branch) = main_worktree(cwd)?;
    let trunk = branch.ok_or("the main worktree has no branch checked out")?;
    if !git_ok(
        &main,
        &["rev-parse", "--verify", "-q", &format!("refs/heads/{stem}")],
    ) {
        return Err(format!("no branch {stem}"));
    }
    let found = worktree_of(&main, stem);
    let dir = found
        .clone()
        .filter(|f| sibling(&main, stem).as_ref() == Some(f))
        .unwrap_or_else(|| nested(&main, stem));
    // Removing the directory the caller stands in breaks its shell: refuse, whatever `--force` says.
    let inside = std::fs::canonicalize(cwd)
        .ok()
        .zip(std::fs::canonicalize(&dir).ok())
        .is_some_and(|(here, target)| here.starts_with(target));
    if inside {
        return Err(format!("the current directory is inside {}: run finish-issue from the main worktree {}; nothing was removed", dir.display(), main.display()));
    }
    let ancestor = git_ok(&main, &["merge-base", "--is-ancestor", stem, &trunk]);
    // A rebase leaves every commit with a patch-equivalent (`-`) on trunk.
    let equivalent = ancestor
        || git(&main, &["cherry", &trunk, stem])
            .is_ok_and(|c| !c.lines().any(|l| l.starts_with('+')));
    // A squash: merging the branch into trunk would change no file, so the merge tree is trunk's tree
    // (keep in sync with `has_own_commits` of `next_issue.rs`; git before 2.38 lacks `--write-tree` and counts as not merged).
    let merged = equivalent || {
        let tree = git(&main, &["merge-tree", "--write-tree", &trunk, stem])
            .ok()
            .and_then(|t| t.lines().next().map(str::to_string));
        tree.is_some() && tree == git(&main, &["rev-parse", &format!("{trunk}^{{tree}}")]).ok()
    };
    if !merged && !force {
        return Err(format!("the branch {stem} is not merged into {trunk}: merge it first, or pass --force to drop it (it deletes work that is not on {trunk}; squash detection needs git 2.38); nothing was removed"));
    }
    match found {
        Some(found) if found != dir => {
            return Err(format!(
                "the branch {stem} is checked out in {}, not in {}: nothing was removed",
                found.display(),
                dir.display()
            ));
        }
        Some(_) if !dir.exists() => {
            git(&main, &["worktree", "prune"])?;
        }
        Some(_) => {
            if !git(&dir, &["status", "--porcelain"])?.is_empty() {
                return Err(format!(
                    "{} has uncommitted changes: nothing was removed",
                    dir.display()
                ));
            }
            if let Some(inner) = worktrees_inside(&main, &dir)?.first() {
                return Err(format!(
                    "{} holds the worktree {}: remove that one first; nothing was removed",
                    dir.display(),
                    inner.display()
                ));
            }
            git(&main, &["worktree", "remove", &dir.to_string_lossy()])?;
        }
        None => {}
    }
    // `-d` only knows ancestors; the other cases were checked above or forced.
    git(&main, &["branch", if ancestor { "-d" } else { "-D" }, stem])?;
    Ok(format!("removed the worktree and the branch {stem}"))
}

pub(super) fn main(args: &[String]) -> u8 {
    let args: Vec<String> = args.to_vec();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["start", issue] => start(Path::new("."), issue),
        ["finish", stem] => finish(Path::new("."), stem, false),
        ["finish", stem, "--force"] | ["finish", "--force", stem] => {
            finish(Path::new("."), stem, true)
        }
        _ => Err(
            "usage: cardinal run issue-worktree start issues/<stem>.md | finish <stem> [--force]"
                .to_string(),
        ),
    };
    match result {
        Ok(message) => {
            println!("{message}");
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}
