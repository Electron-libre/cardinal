//! `waive-review`: the review waiver through `cardinal` alone (ADR 0028), for the user only. It creates the waiver of
//! the worktree it runs in, where `review-gate native` reads it, keeps the content of one already there, and prints
//! its path. With the feature `review-gate` off, `sync` leaves out the deny rules that keep it from the agent, so it
//! refuses. Spec: `specs/review-gate.md` (REV-17, REV-22).

use std::fs::OpenOptions;
use std::process::Command;

use super::repo_root;
use crate::features;

const USAGE: &str = "usage: cardinal run waive-review";

pub(super) fn main(args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("{USAGE}");
        return 2;
    }
    match waive() {
        Ok(path) => {
            println!("{path}");
            0
        }
        Err(e) => {
            eprintln!("waive-review: {e}");
            1
        }
    }
}

/// Creates the waiver of the current worktree and returns its path.
fn waive() -> Result<String, String> {
    let root = repo_root()?;
    if features::is_off(&root, features::REVIEW_GATE) {
        return Err(
            "the project turns off the feature review-gate: there is no review to waive".into(),
        );
    }
    let out = Command::new("git")
        .args([
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "claude-review-waived",
        ])
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // like `touch`: create it, never truncate it, since `native` reads a stamp in its content
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("cannot create {path}: {e}"))?;
    Ok(path)
}
