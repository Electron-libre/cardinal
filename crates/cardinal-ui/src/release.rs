//! The start of a release session (STU-26, STU-27, STU-28): in a zellij session, once a milestone awaits its
//! release, a zellij tab `release` opens in the main worktree, running an interactive `claude` whose first message is
//! `/release`, so `skill:release` keeps the version, its reread, its review and its tag. Nothing opens from a
//! detached, dirty or locked main worktree, in a project that turned the skill off, nor next to an open release tab.

use std::path::Path;

use crate::{merge, source};

/// Why nothing starts outside zellij (STU-27).
pub const NEEDS_ZELLIJ: &str = "release needs zellij: run cardinal-ui in a zellij session";

/// The name of the tab of the release session (STU-28).
const TAB: &str = "release";

/// Starts the release session of the project at `root`: what was started, or why nothing was.
pub fn start(root: &Path) -> Result<String, String> {
    if !source::in_zellij() {
        return Err(NEEDS_ZELLIJ.to_string());
    }
    let (main, _) = merge::free_main_worktree(root)?;
    if !main.join(".claude/skills/release/SKILL.md").is_file() {
        return Err(
            "the skill release is turned off: run the steps cardinal run release prints"
                .to_string(),
        );
    }
    if source::tabs(&main).iter().any(|t| t == TAB) {
        return Err("a release tab is already open".to_string());
    }
    let (doc, _) = source::document("release", &[], &main, false)?;
    if source::array(&doc["milestones"]).is_empty() {
        return Err("no milestone awaits its release".to_string());
    }
    let cwd = main.display().to_string();
    let tab = "zellij action new-tab";
    let args = [
        "action", "new-tab", "--cwd", &cwd, "--name", TAB, "--", "claude", "/release",
    ];
    let (_, err, code) = source::run(tab, &source::zellij(), &args, &main)?;
    if code != 0 {
        return Err(source::failure(tab, code, &err));
    }
    Ok(format!("started the release in {cwd}"))
}
