//! The start of a release session (SUA-4, SUA-11, SUA-12): in a zellij session, once a milestone awaits its
//! release, a zellij tab `release` opens in the main worktree, running an interactive `claude` whose first message is
//! `/release`, so `skill:release` keeps the version, its reread, its review and its tag; a release committed but not
//! tagged yet starts the same session, which resumes it. Nothing opens from a detached, dirty or locked main
//! worktree, with nothing to release, in a project that turned the skill off, nor next to a release tab still running.

use std::path::Path;

use crate::{merge, skill_tab, source};

/// Why nothing starts outside zellij (SUA-11).
pub const NEEDS_ZELLIJ: &str = "release needs zellij: run cardinal-ui in a zellij session";

/// The name of the tab of the release session (SUA-12).
const TAB: &str = "release";

/// The first message of its session (SUA-4).
const MESSAGE: &str = "/release";

/// Starts the release session of the project at `root`: what was started, or why nothing was.
pub fn start(root: &Path) -> Result<String, String> {
    if !source::in_zellij() {
        return Err(NEEDS_ZELLIJ.to_string());
    }
    let (main, _) = merge::free_main_worktree(root)?;
    let (doc, _) = source::document("release", &[], &main, false)?;
    // a release committed but not tagged lists no milestone any more: the skill resumes it
    if source::array(&doc["milestones"]).is_empty() && doc["prepared"] != true {
        return Err("no milestone awaits its release".to_string());
    }
    if !main.join(".claude/skills/release/SKILL.md").is_file() {
        return Err(
            "the skill release is turned off: run the steps cardinal run release prints"
                .to_string(),
        );
    }
    if skill_tab::running(&main, TAB, MESSAGE) {
        return Err("a release tab is already open".to_string());
    }
    let cwd = skill_tab::open(&main, TAB, MESSAGE)?;
    Ok(format!("started the release in {cwd}"))
}
