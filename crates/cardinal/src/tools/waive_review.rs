//! `waive-review`: the review waiver through `cardinal` alone (ADR 0028), for the user only. It writes an empty
//! waiver for the worktree it runs in, where `review-gate native` reads it, and prints its path; an empty one, since
//! `native` drops a waiver stamped by a spent commit. With the feature `review-gate` off, `sync` leaves out the deny
//! rules that keep it from the agent, so it refuses. Spec: `specs/review-gate.md` (REV-17, REV-22).

use super::{repo_root, review_gate};
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

/// Writes the waiver of the current worktree and returns its path.
fn waive() -> Result<String, String> {
    let root = repo_root()?;
    if features::is_off(&root, features::REVIEW_GATE) {
        return Err(
            "the project turns off the feature review-gate: there is no review to waive".into(),
        );
    }
    let cwd =
        std::env::current_dir().map_err(|e| format!("cannot read the current directory: {e}"))?;
    let path = review_gate::waiver_path(&cwd).ok_or("cannot find the git directory")?;
    std::fs::write(&path, "").map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    // relative to `cwd` from a subdirectory (`sub/../.git/…`): print the path the user can read
    let path = path.canonicalize().unwrap_or(path);
    Ok(path.display().to_string())
}
