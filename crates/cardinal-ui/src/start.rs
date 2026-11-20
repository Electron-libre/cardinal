//! The start of an issue session (STU-9, STU-11, STU-12): in a zellij session, a ready issue nothing takes gets its
//! worktree (`cardinal run worktree open <stem>`) and a zellij tab named after its stem, running an interactive
//! `claude` that hands `issues/<stem>.md` to `issue-workflow`. Each step runs only once the one before succeeded.

use std::path::Path;

use crate::source;

/// Why nothing starts outside zellij (STU-11).
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
    if issue.taken.is_some() {
        return Err(format!("{stem}: taken"));
    }
    if issue.state != "ready" {
        return Err(format!("{stem}: {}, not ready", issue.state));
    }
    let open = format!("worktree open {stem}");
    let args = ["run", "worktree", "open", stem];
    let (out, err, code) = source::run(&open, &source::cardinal(), &args, root)?;
    if code != 0 {
        return Err(source::failure(&open, code, &err));
    }
    // the launcher may build a source first: the path is the last line the tool prints
    let worktree = out
        .lines()
        .map(str::trim)
        .rfind(|l| !l.is_empty())
        .ok_or_else(|| format!("{open}: printed no path"))?
        .to_string();
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
    Ok(format!("started {stem} in {worktree}"))
}
