//! The start of a review of the priorities (SUA-16, SUA-17, SUA-18): in a zellij session, once the queue lists an
//! open issue, a zellij tab `prioritize` opens in the main worktree, running an interactive `claude` whose first
//! message is `/prioritize-issues`, so `skill:prioritize-issues` keeps the review, its proposal, its stop and its
//! commit; the UI writes no front matter. Nothing opens from a detached, dirty or locked main worktree, with no open
//! issue, in a project that turned the skill off, nor next to a prioritize tab still running.

use std::path::Path;

use crate::{merge, skill_tab, source};

/// Why nothing starts outside zellij (SUA-17).
pub const NEEDS_ZELLIJ: &str = "prioritize needs zellij: run cardinal-ui in a zellij session";

/// The name of the tab of the review (SUA-18).
const TAB: &str = "prioritize";

/// The first message of its session (SUA-16).
const MESSAGE: &str = "/prioritize-issues";

/// Starts the review of the priorities of the project at `root`: what was started, or why nothing was.
pub fn start(root: &Path) -> Result<String, String> {
    if !source::in_zellij() {
        return Err(NEEDS_ZELLIJ.to_string());
    }
    let (main, _) = merge::free_main_worktree(root)?;
    if source::queue(&main, &[])?.issues.is_empty() {
        return Err("no open issue to prioritize".to_string());
    }
    if !main
        .join(".claude/skills/prioritize-issues/SKILL.md")
        .is_file()
    {
        return Err("the skill prioritize-issues is turned off".to_string());
    }
    if skill_tab::running(&main, TAB, MESSAGE) {
        return Err("a prioritize tab is already open".to_string());
    }
    let cwd = skill_tab::open(&main, TAB, MESSAGE)?;
    Ok(format!("started the review of the priorities in {cwd}"))
}
