//! The start of a release session (SUA-4, SUA-11, SUA-12): in a zellij session, once a milestone awaits its
//! release, a zellij tab `release` opens in the main worktree, running an interactive `claude` whose first message is
//! `/release`, so `skill:release` keeps the version, its reread, its review and its tag; a release committed but not
//! tagged yet starts the same session, which resumes it. Nothing opens from a detached, dirty or locked main
//! worktree, with nothing to release, in a project that turned the skill off, nor next to a release tab still running.

use std::path::Path;

use serde_json::Value;

use crate::{merge, source};

/// Why nothing starts outside zellij (SUA-11).
pub const NEEDS_ZELLIJ: &str = "release needs zellij: run cardinal-ui in a zellij session";

/// The name of the tab of the release session (SUA-12).
const TAB: &str = "release";

/// Whether a tab `release` holds a terminal pane still running `claude /release`; `false` when the panes cannot be
/// listed (SUA-11).
fn tab_open(dir: &Path) -> bool {
    let args = ["action", "list-panes", "--json", "--all"];
    let Ok((out, _, 0)) = source::run("zellij", &source::zellij(), &args, dir) else {
        return false;
    };
    let panes = serde_json::from_str::<Value>(out.trim()).unwrap_or_default();
    source::array(&panes).iter().any(|p| {
        p["is_plugin"] == false
            && p["tab_name"] == TAB
            && p["terminal_command"] == "claude /release"
            && p["exited"] != true
    })
}

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
    if tab_open(&main) {
        return Err("a release tab is already open".to_string());
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
