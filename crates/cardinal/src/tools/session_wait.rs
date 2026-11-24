//! The wait mark of a session (spec `specs/session-wait.md`, ADR 0031): `wait`, at `Stop` and `Notification`, writes
//! the file `cardinal-waiting` of the git directory of the worktree of the event's `cwd`, naming the session, when it
//! began to wait and why (a mark of the same session keeps its time); `work`, at `UserPromptSubmit`, `PostToolUse`,
//! `PostToolUseFailure` and `SessionEnd`, removes it when it names the session, unless a subagent sent the event. A
//! program such as `cardinal-ui` reads it to show the sessions that wait for the user. The hooks never block nor
//! speak: any failure leaves things as they are, prints nothing, exits 0.
//!
//! Usage: hooks `cardinal run session-wait wait` (`Stop`, `Notification`), `work` (`UserPromptSubmit`, `PostToolUse`,
//! `PostToolUseFailure`, `SessionEnd`).

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The name of the mark in the git directory of a worktree (SWT-3).
const MARK: &str = "cardinal-waiting";

pub(super) fn main(args: &[String]) -> u8 {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).ok();
    let Ok(event) = serde_json::from_str::<serde_json::Value>(&input) else {
        return 0;
    };
    let (Some(session), Some(cwd)) = (event["session_id"].as_str(), event["cwd"].as_str()) else {
        return 0;
    };
    let mode = args.first().map(String::as_str);
    if session.is_empty() || !matches!(mode, Some("wait" | "work")) {
        return 0;
    }
    let Some(mark) = mark_path(Path::new(cwd)) else {
        return 0;
    };
    if mode == Some("wait") {
        wait(&mark, session, &reason(&event));
    } else if event.get("agent_id").is_none() {
        // a subagent works while the main thread may wait for the user (SWT-2)
        work(&mark, session);
    }
    0
}

/// The path of the mark of the worktree of `cwd`; `None` outside a git repository (SWT-3, SWT-4).
fn mark_path(cwd: &Path) -> Option<PathBuf> {
    crate::git::run(
        cwd,
        &["rev-parse", "--path-format=absolute", "--git-path", MARK],
    )
    .ok()
    .filter(|path| !path.is_empty())
    .map(PathBuf::from)
}

/// Why the session waits: `stop` at the end of a turn, else the first line of the notification (SWT-1).
fn reason(event: &serde_json::Value) -> String {
    if event["hook_event_name"] == "Stop" {
        return "stop".to_string();
    }
    event["message"]
        .as_str()
        .and_then(|m| m.lines().map(str::trim).find(|l| !l.is_empty()))
        .unwrap_or("notification")
        .to_string()
}

/// Writes the mark whole: a temporary file beside it, renamed over it (SWT-3); a mark of the same session keeps the
/// time its wait began (SWT-1).
fn wait(mark: &Path, session: &str, reason: &str) {
    let session = session.replace('\n', " ");
    let earlier = fs::read_to_string(mark).ok().and_then(|text| {
        (cardinal_state::field(&text, "session") == Some(session.as_str()))
            .then(|| {
                text.lines()
                    .find_map(|l| l.strip_prefix("since ")?.parse::<u64>().ok())
            })
            .flatten()
    });
    let since = earlier.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    });
    let text = format!("session {session}\nsince {since}\nreason {reason}\n");
    let tmp = mark.with_extension(format!("{}.tmp", std::process::id()));
    if fs::write(&tmp, text).is_ok() && fs::rename(&tmp, mark).is_err() {
        fs::remove_file(&tmp).ok();
    }
}

/// Removes the mark when it names `session` (SWT-2).
fn work(mark: &Path, session: &str) {
    let Ok(text) = fs::read_to_string(mark) else {
        return;
    };
    if cardinal_state::field(&text, "session") == Some(session) {
        fs::remove_file(mark).ok();
    }
}
