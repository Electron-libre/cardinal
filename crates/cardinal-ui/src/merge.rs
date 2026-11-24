//! The merge of an issue closed on its branch (SUM-1, SUM-2): when a claim in the state `merge pending` takes it,
//! the trunk of the main worktree is fast-forwarded to the branch of that claim, then `cardinal check` runs there.
//! Nothing is merged from a detached, dirty or locked main worktree; a branch the trunk cannot fast-forward to is
//! first rebased by `cardinal run rebase-issue` and checked by `cardinal run project-check` in its worktree (SUM-3),
//! once the session of that worktree that only waits for the merge ended (SUM-4);
//! the UI's own git commands run with the main worktree as their directory, never a linked one.

use std::path::{Path, PathBuf};

use crate::end;
use crate::source::{self, Taken};

/// Runs `git args` in `dir`: its stdout, or the failure line of SUM-2.
pub fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let name = format!("git {}", args.join(" "));
    let (out, err, code) = source::run(&name, Path::new("git"), args, dir)?;
    if code != 0 {
        return Err(source::failure(&name, code, &err));
    }
    Ok(out)
}

/// The main worktree of the repository of `dir` and its branch, `None` when detached.
pub fn main_worktree(dir: &Path) -> Result<(PathBuf, Option<String>), String> {
    let list = git(dir, &["worktree", "list", "--porcelain"])?;
    cardinal_state::worktree::main(&cardinal_state::worktree::parse(&list))
}

/// The main worktree of the repository of `dir` and its trunk, once it is on a branch, clean and held by no running
/// session; else why not (SUM-2, SUA-11).
pub fn free_main_worktree(dir: &Path) -> Result<(PathBuf, String), String> {
    let (main, trunk) = main_worktree(dir)?;
    let trunk = trunk.ok_or("main worktree not on a branch")?;
    if !git(&main, &["status", "--porcelain"])?.trim().is_empty() {
        return Err("main worktree has uncommitted changes or untracked files".to_string());
    }
    if let Some(s) = source::session(&main).filter(|s| s.alive) {
        return Err(format!("session {} works in the main worktree", s.id));
    }
    Ok((main, trunk))
}

/// The branch of the claim `merge pending` of the issue `stem` of the queue of `root`.
fn pending_branch(root: &Path, stem: &str) -> Result<String, String> {
    let queue = source::queue(root, &[])?;
    queue
        .issues
        .iter()
        .find(|i| i.stem == stem)
        .and_then(|i| match &i.taken {
            Some(Taken::Claim { state, branch, .. }) if state == "merge pending" => {
                Some(branch.clone())
            }
            _ => None,
        })
        .ok_or_else(|| format!("{stem}: not merge pending"))
}

/// The worktree of the repository of `dir` that has `branch` checked out.
fn worktree_of(dir: &Path, branch: &str) -> Result<PathBuf, String> {
    let list = git(dir, &["worktree", "list", "--porcelain"])?;
    let line = format!("branch refs/heads/{branch}");
    list.split("\n\n")
        .find(|r| r.lines().any(|l| l == line))
        .and_then(|r| r.lines().find_map(|l| l.strip_prefix("worktree ")))
        .map(PathBuf::from)
        .ok_or_else(|| format!("{branch} is checked out in no worktree"))
}

/// Rebases `branch` onto `trunk` with `cardinal run rebase-issue`, run in the main worktree `main`, then runs the
/// project check in the worktree of the branch; else why the merge must not go on (SUM-3).
fn rebase(main: &Path, stem: &str, branch: &str, trunk: &str) -> Result<(), String> {
    let name = "cardinal run rebase-issue";
    let (_, err, code) = source::run(
        name,
        &source::cardinal(),
        &["run", "rebase-issue", stem],
        main,
    )?;
    if code != 0 {
        return Err(err.lines().find(|l| !l.trim().is_empty()).map_or_else(
            || source::failure(name, code, &err),
            |l| l.trim().to_string(),
        ));
    }
    let worktree = worktree_of(main, branch)?;
    let check = "project-check";
    let rebased = format!("rebased {branch} onto {trunk}");
    match source::run(
        check,
        &source::cardinal(),
        &["run", "project-check"],
        &worktree,
    ) {
        Ok((_, _, 0)) => Ok(()),
        Ok((_, err, code)) => Err(format!("{rebased}; {}", source::failure(check, code, &err))),
        Err(e) => Err(format!("{rebased}; {e}")),
    }
}

/// Merges the issue `stem` of the project at `root` into the trunk: what was merged, or why nothing was, or why the
/// check after the merge failed (SUM-1, SUM-2).
pub fn merge(root: &Path, stem: &str) -> Result<String, String> {
    merge_branch(root, stem).map(|(_, merged)| merged)
}

/// The merge of `merge`, with the branch of the claim it merged when it succeeded (SUE-1).
pub fn merge_branch(root: &Path, stem: &str) -> Result<(String, String), String> {
    let branch = pending_branch(root, stem)?;
    // the full name: a tag of the same name would win over the branch
    let head = format!("refs/heads/{branch}");
    if !source::holds_close(root, &branch, stem)? {
        return Err(format!(
            "{branch} does not hold the close of {stem}: commit it first"
        ));
    }
    let (main, trunk) = free_main_worktree(root)?;
    let ancestor = "git merge-base --is-ancestor";
    let (_, err, code) = source::run(
        ancestor,
        Path::new("git"),
        &["merge-base", "--is-ancestor", "HEAD", &head],
        &main,
    )?;
    // what the end of the waiting session did before the rebase, named in any failure that follows (SUM-4)
    let mut ended = Vec::new();
    let rebased = match code {
        0 => false,
        1 => {
            ended = end::end_waiting(root, stem, &branch)?;
            rebase(&main, stem, &branch, &trunk)
                // the rebase and its check take minutes: the main worktree is checked again before the merge
                .and_then(|()| free_main_worktree(root))
                .map_err(|e| end::failed(&ended, e))?;
            true
        }
        _ => return Err(source::failure(ancestor, code, &err)),
    };
    git(&main, &["merge", "--ff-only", "--quiet", &head]).map_err(|e| end::failed(&ended, e))?;
    let merged = if rebased {
        format!("rebased and merged {stem} ({branch}) into {trunk}")
    } else {
        format!("merged {stem} ({branch}) into {trunk}")
    };
    let check = "cardinal check";
    match source::run(check, &source::cardinal(), &["check"], &main) {
        Ok((_, _, 0)) => Ok((branch, merged)),
        Ok((_, err, code)) => Err(end::failed(
            &ended,
            format!("{merged}; {}", source::failure(check, code, &err)),
        )),
        Err(e) => Err(end::failed(&ended, format!("{merged}; {e}"))),
    }
}
