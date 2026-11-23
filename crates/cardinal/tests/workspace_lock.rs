//! Behavior test for the hooks `cardinal tool workspace-lock check | start | release` (`specs/workspace-lock.md`):
//! hook events on stdin, the process of the session in `CLAUDE_PID`, against a throwaway repository with a linked
//! worktree.

mod workspace_lock_common;

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use workspace_lock_common::*;

// spec: WSL-1
#[test]
fn check_takes_a_free_or_stale_lock_and_lets_the_call_through() {
    let r = repo();
    let (code, _, err) = hook("check", &r.main, "a", &dead());
    assert_eq!(code, Some(0), "{err}");
    assert!(lock(&r.main).contains("session a\n"));
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "a dead holder frees the lock: {err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
}

// spec: WSL-1
#[test]
fn check_takes_a_lock_whose_pid_a_later_process_took() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    dirty(&r.main);
    // long before `other` started: the session that wrote it ended and `other` took its number
    written_at(&r.main, UNIX_EPOCH + Duration::from_hours(24));
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "a stale lock is free: {err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
}

// spec: WSL-1, WSL-3
#[test]
fn check_refuses_while_the_holder_started_before_the_last_write() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    dirty(&r.main);
    // half a second before now, once `other` runs: written after it started, by less than the 2 s margin
    written_at(&r.main, SystemTime::now() - Duration::from_millis(500));
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "{err}");
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-2
#[test]
fn check_lets_the_holder_and_its_process_through() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    let (code, _, _) = hook("check", &r.main, "a", &live());
    assert_eq!(code, Some(0), "the same session in a new process");
    assert_eq!(lock(&r.main), format!("session a\npid {}\n", live()));
    let (code, _, _) = hook("check", &r.main, "c", &live());
    assert_eq!(code, Some(0), "a cleared session of the same process");
    assert!(lock(&r.main).contains("session c\n"));
}

// spec: WSL-3
#[test]
fn check_refuses_another_session_while_the_holder_runs() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    dirty(&r.main);
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "{err}");
    assert!(
        err.contains("another live session") && err.contains("(a)") && err.contains("worktree"),
        "{err}"
    );
    assert!(
        err.contains(&format!("process {}", other.pid()))
            && err.contains("    ?? wip")
            && err.contains("git stash -u"),
        "the refusal names the process and the paths that keep the lock: {err}"
    );
    assert!(
        err.contains(&format!("this session (b) of process {}", live())),
        "the refusal names the calling session and process: {err}"
    );
    for i in 0..12 {
        std::fs::write(r.main.join(format!("more{i}")), "").unwrap();
    }
    let (_, _, err) = hook("check", &r.main, "b", &live());
    assert!(
        err.contains("and 3 more"),
        "a long list says what it leaves out: {err}"
    );
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-4
#[test]
fn two_sessions_taking_the_lock_at_once_leave_one_holder() {
    for _ in 0..10 {
        let r = repo();
        dirty(&r.main);
        let other = Other::new();
        let a = spawn("check", &r.main, "a", &live(), None);
        let b = spawn("check", &r.main, "b", &other.pid(), None);
        let (a, b) = (finish(a).0, finish(b).0);
        assert!(
            (a == Some(0)) ^ (b == Some(0)),
            "exactly one holds it: {a:?} {b:?}"
        );
        let winner = if a == Some(0) { "a" } else { "b" };
        assert!(lock(&r.main).contains(&format!("session {winner}\n")));
    }
}

// spec: WSL-5
#[test]
fn start_takes_the_lock_or_tells_the_agent_the_worktree_is_busy() {
    let r = repo();
    let other = Other::new();
    let (code, out, _) = hook("start", &r.main, "a", &other.pid());
    assert_eq!(code, Some(0));
    assert!(out.is_empty(), "{out}");
    assert!(lock(&r.main).contains("session a\n"));
    dirty(&r.main);
    let (code, out, _) = hook("start", &r.main, "b", &live());
    assert_eq!(code, Some(0), "SessionStart cannot block");
    assert!(out.contains("busy") && out.contains("refused"), "{out}");
    assert!(
        out.contains(&format!("process {}", other.pid())) && out.contains("    ?? wip"),
        "{out}"
    );
    assert!(
        out.contains(&format!("this session (b) of process {}", live())),
        "{out}"
    );
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-10
#[test]
fn a_holder_with_nothing_in_progress_hands_the_lock_over() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "a clean worktree and no claim: {err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
    dirty(&r.main);
    let (code, _, err) = hook("check", &r.main, "a", &other.pid());
    assert_eq!(
        code,
        Some(2),
        "the old holder is refused once b has work in progress: {err}"
    );
    assert!(lock(&r.main).contains("session b\n"));
}

// spec: WSL-10
#[test]
fn start_takes_over_the_lock_of_a_holder_with_nothing_in_progress() {
    let r = repo();
    let other = Other::new();
    hook("start", &r.main, "a", &other.pid());
    let (code, out, _) = hook("start", &r.main, "b", &live());
    assert_eq!(code, Some(0));
    assert!(out.is_empty(), "no busy warning: {out}");
    assert!(lock(&r.main).contains("session b\n"));
}

// spec: WSL-10
#[test]
fn a_holder_with_a_working_claim_keeps_the_lock_in_a_clean_worktree() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    claim(&r.main, "foo", "a", &other.pid());
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "claimed by its session: {err}");
    assert!(
        err.contains("the issue foo it claims") && !err.contains("git stash"),
        "the refusal names the claim, not changes to stash: {err}"
    );
    claim(&r.main, "foo", "cleared", &other.pid());
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "claimed by its process: {err}");
    claim(&r.main, "foo", "z", &dead());
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "a claim of another session: {err}");
    assert!(lock(&r.main).contains("session b\n"));
}

// spec: WSL-10
#[test]
fn a_claim_whose_pid_the_holder_took_later_keeps_no_lock() {
    let r = repo();
    let holder = Other::new();
    hook("check", &r.main, "h", &holder.pid());
    // session `a` claimed `foo` on 2 January 1970 and crashed; `h` took its pid since
    claim_since(&r.main, "foo", "a", &holder.pid(), 86400);
    let (code, _, err) = hook("check", &r.main, "c", &live());
    assert_eq!(
        code,
        Some(0),
        "the claim of a pid taken later is not the holder's: {err}"
    );
    assert!(lock(&r.main).contains("session c\n"));
}

// spec: WSL-H1
#[test]
fn a_busy_worktree_refuses_another_session_until_its_holder_ends() {
    let r = repo();
    let other = Other::new();
    let (code, _, _) = hook("start", &r.main, "a", &other.pid());
    assert_eq!(code, Some(0));
    dirty(&r.main);
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "{err}");
    assert!(err.contains("(a)"), "{err}");
    let (code, out, err) = hook("release", &r.main, "a", &other.pid());
    assert_eq!((code, out.as_str(), err.as_str()), (Some(0), "", ""));
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "{err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
}

// spec: WSL-6
#[test]
fn release_deletes_only_its_own_lock() {
    let r = repo();
    hook("check", &r.main, "a", &live());
    let (code, _, _) = hook("release", &r.main, "b", &live());
    assert_eq!(code, Some(0));
    assert!(lock(&r.main).contains("session a\n"));
    hook("release", &r.main, "a", &live());
    assert_eq!(lock(&r.main), "");
}

// spec: WSL-7
#[test]
fn a_broken_input_or_no_repository_lets_the_call_through() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    let (code, _, _) = finish(spawn("check", &r.main, "b", &live(), Some("not json")));
    assert_eq!(code, Some(0), "unreadable input");
    let (code, _, _) = hook("check", &r.main, "b", "");
    assert_eq!(code, Some(0), "no CLAUDE_PID");
    let outside = tempfile::tempdir().unwrap();
    let (code, _, _) = hook("check", outside.path(), "b", &live());
    assert_eq!(code, Some(0), "no repository");
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-9
#[test]
fn a_linked_worktree_has_its_own_lock() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    let (code, _, err) = hook("check", &r.ws, "b", &live());
    assert_eq!(code, Some(0), "{err}");
    assert!(lock(&r.ws).contains("session b\n"));
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-8
#[test]
fn the_shipped_settings_entries_declare_the_three_hooks() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../entries/.claude/settings.json"),
    )
    .unwrap();
    let entries: serde_json::Value = serde_json::from_str(&text).unwrap();
    let declared = |event: &str, matcher: Option<&str>, command: &str| {
        entries["hooks"][event].as_array().is_some_and(|groups| {
            groups.iter().any(|g| {
                matcher.is_none_or(|m| g["matcher"] == m)
                    && g["hooks"]
                        .as_array()
                        .is_some_and(|h| h.iter().any(|h| h["command"] == command))
            })
        })
    };
    assert!(declared(
        "PreToolUse",
        Some("Bash|Edit|Write|NotebookEdit"),
        "cardinal run workspace-lock check"
    ));
    assert!(declared(
        "SessionStart",
        None,
        "cardinal run workspace-lock start"
    ));
    assert!(declared(
        "SessionEnd",
        None,
        "cardinal run workspace-lock release"
    ));
}
