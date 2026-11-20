//! The start of an issue session (SUA-1, SUA-5, SUA-10): in a zellij session, a ready issue nothing takes gets its
//! worktree (`cardinal run worktree open <stem>`) and a zellij tab named after its stem, running an interactive
//! `claude` that hands `issues/<stem>.md` to `issue-workflow`. An issue whose `working` claim names a session that
//! ended is resumed instead: the tab opens in the worktree of the claim, and the new session takes the claim back
//! (CLM-2). Each step runs only once the one before succeeded.

use std::path::Path;

use crate::source;

/// Why nothing starts outside zellij (SUA-5).
pub const NEEDS_ZELLIJ: &str = "start needs zellij: run cardinal-ui in a zellij session";

/// Starts the session of the issue `stem` of the project at `root`: what was started, or why nothing was.
pub fn start(root: &Path, stem: &str) -> Result<String, String> {
    if !source::in_zellij() {
        return Err(NEEDS_ZELLIJ.to_string());
    }
    let queue = source::queue(root, &[])?;
    let issue = queue
        .issues
        .iter()
        .find(|i| i.stem == stem)
        .ok_or_else(|| format!("{stem}: no open issue of this name"))?;
    // a working claim whose session ended is resumed in its worktree: the new session takes it back (CLM-2)
    let resumed = match &issue.taken {
        None => None,
        Some(source::Taken::Claim {
            state,
            worktree,
            alive: false,
            ..
        }) if state == "working" => Some(worktree.clone()),
        Some(_) => return Err(format!("{stem}: taken")),
    };
    if issue.state != "ready" {
        return Err(format!("{stem}: {}, not ready", issue.state));
    }
    let (worktree, done) = match resumed {
        Some(w) => (resumable(root, stem, w)?, "resumed"),
        None => (open_worktree(root, stem)?, "started"),
    };
    let first_message = format!("/issue-workflow issues/{stem}.md");
    let tab = "zellij action new-tab";
    let args = [
        "action",
        "new-tab",
        "--cwd",
        &worktree,
        "--name",
        stem,
        "--",
        "claude",
        &first_message,
    ];
    let (_, err, code) = source::run(tab, &source::zellij(), &args, root)?;
    if code != 0 {
        return Err(source::failure(tab, code, &err));
    }
    Ok(format!("{done} {stem} in {worktree}"))
}

/// The worktree `w` of the claim of `stem` whose session ended, once `git worktree list` lists it and no live
/// session holds its lock (SUA-5).
fn resumable(root: &Path, stem: &str, w: String) -> Result<String, String> {
    let worktrees = source::worktrees(root)?;
    // the claim holds `--show-toplevel`, symlinks resolved; the list holds the path given to `worktree add`
    let real = |p: &str| std::fs::canonicalize(p).unwrap_or_else(|_| p.into());
    let Some(listed) = worktrees.iter().find(|t| real(&t.path) == real(&w)) else {
        return Err(format!(
            "{stem}: worktree {w} gone: cardinal run issue-claim --release {stem}"
        ));
    };
    if let Some(session) = listed.session.as_ref().filter(|s| s.alive) {
        return Err(format!("{stem}: session {} works in {w}", session.id));
    }
    Ok(w)
}

/// Opens the worktree of `stem` with `cardinal run worktree open`: its path.
fn open_worktree(root: &Path, stem: &str) -> Result<String, String> {
    let open = format!("worktree open {stem}");
    let args = ["run", "worktree", "open", stem];
    let (out, err, code) = source::run(&open, &source::cardinal(), &args, root)?;
    if code != 0 {
        return Err(source::failure(&open, code, &err));
    }
    // the launcher may build a source first: the path is the last line the tool prints
    out.lines()
        .map(str::trim)
        .rfind(|l| !l.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("{open}: printed no path"))
}
