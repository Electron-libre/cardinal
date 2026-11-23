//! The cut of a milestone into issues (SUA-19, SUA-20, SUA-21): in a zellij session, on a milestone `roadmap` lists
//! in the state `shaped` or `idea`, a zellij tab named after it opens in the main worktree, running an interactive
//! `claude` whose first message asks `skill:create-issue` to cut it, so the proposal of the issues, its stop and
//! their commit stay with the skill, which agrees the goal of an `idea` milestone with the user. Nothing opens from a
//! detached, dirty or locked main worktree, on another milestone, in a project that turned the skill off, nor next to
//! a tab of that milestone still running.

use std::path::Path;

use crate::{merge, skill_tab, source};

/// Why nothing starts outside zellij (SUA-20).
pub const NEEDS_ZELLIJ: &str = "cut needs zellij: run cardinal-ui in a zellij session";

/// The first message of the session of the milestone `name` (SUA-21).
fn message(name: &str) -> String {
    format!("/create-issue cut the milestone {name} into issues")
}

/// Starts the cut of the milestone `name` of the project at `root`: what was started, or why nothing was.
pub fn start(root: &Path, name: &str) -> Result<String, String> {
    if !source::in_zellij() {
        return Err(NEEDS_ZELLIJ.to_string());
    }
    let (main, _) = merge::free_main_worktree(root)?;
    let (doc, _) = source::document("roadmap", &["--open"], &main, false)?;
    let milestones = source::array(&doc["milestones"]);
    let milestone = milestones
        .iter()
        .find(|m| m["name"] == name)
        .ok_or_else(|| format!("{name}: no open milestone of that name"))?;
    let state = source::string(&milestone["state"]).unwrap_or_else(|| "(none)".to_string());
    if state != "shaped" && state != "idea" {
        return Err(format!("{name}: {state}, not shaped nor idea"));
    }
    if !main.join(".claude/skills/create-issue/SKILL.md").is_file() {
        return Err("the skill create-issue is turned off".to_string());
    }
    let message = message(name);
    if skill_tab::running(&main, name, &message) {
        return Err(format!("a tab {name} is already open"));
    }
    let cwd = skill_tab::open(&main, name, &message)?;
    Ok(format!("started the cut of {name} in {cwd}"))
}
