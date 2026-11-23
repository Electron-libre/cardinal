//! The start of an audit of the code (SUA-23, SUA-24, SUA-25): in a zellij session, a zellij tab `audit` opens in the
//! main worktree, running an interactive `claude` whose first message is `/audit-code`, so `skill:audit-code` keeps
//! the audit, its proposal, its stop and the issues it writes. Nothing opens from a detached, dirty or locked main
//! worktree, in one without the skill, nor next to an audit tab still running.

use std::path::Path;

use crate::{merge, skill_tab, source};

/// Why nothing starts outside zellij (SUA-24).
pub const NEEDS_ZELLIJ: &str = "audit needs zellij: run cardinal-ui in a zellij session";

/// The name of the tab of the audit (SUA-25).
const TAB: &str = "audit";

/// The first message of its session (SUA-23).
const MESSAGE: &str = "/audit-code";

/// Starts the audit of the code of the project at `root`: what was started, or why nothing was.
pub fn start(root: &Path) -> Result<String, String> {
    if !source::in_zellij() {
        return Err(NEEDS_ZELLIJ.to_string());
    }
    let (main, _) = merge::free_main_worktree(root)?;
    if !main.join(".claude/skills/audit-code/SKILL.md").is_file() {
        return Err("the skill audit-code is missing: run cardinal sync".to_string());
    }
    if skill_tab::running(&main, TAB, MESSAGE) {
        return Err("an audit tab is already open".to_string());
    }
    let cwd = skill_tab::open(&main, TAB, MESSAGE)?;
    Ok(format!("started the audit of the code in {cwd}"))
}
