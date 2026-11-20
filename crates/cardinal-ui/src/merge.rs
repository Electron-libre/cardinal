//! The merge of an issue closed on its branch (STU-20, STU-21): when a claim in the state `merge pending` takes it,
//! the trunk of the main worktree is fast-forwarded to the branch of that claim, then `cardinal check` runs there.
//! Nothing is merged from a detached, dirty or locked main worktree, nor onto a branch the trunk cannot
//! fast-forward to; git always runs with the main worktree as its directory, never a linked one.

use std::path::{Path, PathBuf};

use crate::source::{self, Taken};

/// Runs `git args` in `dir`: its stdout, or the failure line of STU-21.
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
    let record = list.split("\n\n").next().unwrap_or_default();
    if record.lines().any(|l| l == "bare") {
        return Err("no main worktree: the repository is bare".to_string());
    }
    let path = record
        .lines()
        .find_map(|l| l.strip_prefix("worktree "))
        .ok_or("git worktree list: no main worktree")?;
    let branch = record
        .lines()
        .find_map(|l| l.strip_prefix("branch refs/heads/"))
        .map(str::to_string);
    Ok((PathBuf::from(path), branch))
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

/// Merges the issue `stem` of the project at `root` into the trunk: what was merged, or why nothing was, or why the
/// check after the merge failed (STU-20, STU-21).
pub fn merge(root: &Path, stem: &str) -> Result<String, String> {
    let branch = pending_branch(root, stem)?;
    // the full name: a tag of the same name would win over the branch
    let head = format!("refs/heads/{branch}");
    if !source::holds_close(root, &branch, stem)? {
        return Err(format!(
            "{branch} does not hold the close of {stem}: commit it first"
        ));
    }
    let (main, trunk) = main_worktree(root)?;
    let trunk = trunk.ok_or("main worktree not on a branch")?;
    if !git(&main, &["status", "--porcelain"])?.trim().is_empty() {
        return Err("main worktree has uncommitted changes or untracked files".to_string());
    }
    if let Some(s) = source::session(&main).filter(|s| s.alive) {
        return Err(format!("session {} works in the main worktree", s.id));
    }
    let ancestor = "git merge-base --is-ancestor";
    let (_, err, code) = source::run(
        ancestor,
        Path::new("git"),
        &["merge-base", "--is-ancestor", "HEAD", &head],
        &main,
    )?;
    match code {
        0 => {}
        1 => {
            return Err(format!(
                "{branch} diverged from {trunk}: rebase it in its worktree"
            ))
        }
        _ => return Err(source::failure(ancestor, code, &err)),
    }
    git(&main, &["merge", "--ff-only", "--quiet", &head])?;
    let merged = format!("merged {stem} ({branch}) into {trunk}");
    let check = "cardinal check";
    match source::run(check, &source::cardinal(), &["check"], &main) {
        Ok((_, _, 0)) => Ok(merged),
        Ok((_, err, code)) => Err(format!("{merged}; {}", source::failure(check, code, &err))),
        Err(e) => Err(format!("{merged}; {e}")),
    }
}
