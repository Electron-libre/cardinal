//! Behavior test for the hooks `cardinal tool session-wait wait | work` (`specs/session-wait.md`): hook events on
//! stdin, against a throwaway repository with a linked worktree.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "user.email=t@t", "-c", "user.name=t"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8_lossy(&out.stdout).trim_end().to_string()
}

/// A repository `main` and its linked worktree `ws`.
struct Repo {
    root: tempfile::TempDir,
    main: PathBuf,
    ws: PathBuf,
}

fn repo() -> Repo {
    let root = tempfile::tempdir().unwrap();
    let main = root.path().join("main");
    std::fs::create_dir_all(&main).unwrap();
    git(&main, &["init", "-q", "-b", "trunk", "."]);
    git(&main, &["commit", "-q", "--allow-empty", "-m", "init"]);
    let ws = root.path().join("ws");
    git(
        &main,
        &["worktree", "add", "-q", "-b", "ws", ws.to_str().unwrap()],
    );
    Repo { root, main, ws }
}

/// Runs `cardinal tool session-wait <mode>` with `input` on stdin; returns (exit code, stdout, stderr).
fn run(mode: &str, input: &str) -> (Option<i32>, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "session-wait", mode])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn event(name: &str, session: &str, cwd: &Path) -> serde_json::Value {
    serde_json::json!({"session_id": session, "cwd": cwd, "hook_event_name": name})
}

/// Runs `mode` for the event `name` of `session` in `cwd`, asserting it exits 0 silently.
fn hook(mode: &str, name: &str, session: &str, cwd: &Path) {
    let (code, out, err) = run(mode, &event(name, session, cwd).to_string());
    assert_eq!((code, out.as_str(), err.as_str()), (Some(0), "", ""));
}

/// The path of the mark of the worktree `dir`.
fn mark_path(dir: &Path) -> PathBuf {
    let path = git(
        dir,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "cardinal-waiting",
        ],
    );
    PathBuf::from(path)
}

fn mark(dir: &Path) -> Option<String> {
    std::fs::read_to_string(mark_path(dir)).ok()
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// spec: SWT-H1
// spec: SWT-1
// spec: SWT-3
#[test]
fn a_turn_end_marks_the_worktree_until_the_user_answers() {
    let r = repo();
    let before = now();
    hook("wait", "Stop", "a", &r.ws);
    let text = mark(&r.ws).expect("a mark in the git directory of the worktree");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "session a");
    let since: u64 = lines[1].strip_prefix("since ").unwrap().parse().unwrap();
    assert!(since >= before && since <= now(), "{since}");
    assert_eq!(lines[2], "reason stop");
    assert_eq!(lines.len(), 3);
    assert!(
        mark(&r.main).is_none(),
        "the main worktree has its own mark"
    );
    hook("work", "UserPromptSubmit", "a", &r.ws);
    assert!(mark(&r.ws).is_none());
}

// spec: SWT-1
#[test]
fn a_notification_gives_the_first_line_of_its_message() {
    let r = repo();
    let mut e = event("Notification", "a", &r.main);
    e["message"] = "Claude needs your permission to use Bash\nmore".into();
    let (code, _, _) = run("wait", &e.to_string());
    assert_eq!(code, Some(0));
    let text = mark(&r.main).unwrap();
    assert!(
        text.ends_with("reason Claude needs your permission to use Bash\n"),
        "{text}"
    );
}

// spec: SWT-1
#[test]
fn a_notification_without_a_message_still_marks() {
    let r = repo();
    hook("wait", "Notification", "a", &r.main);
    assert!(mark(&r.main).unwrap().contains("reason notification\n"));
}

// spec: SWT-2
#[test]
fn work_of_another_session_leaves_the_mark() {
    let r = repo();
    hook("wait", "Stop", "a", &r.ws);
    hook("work", "PostToolUse", "b", &r.ws);
    assert!(mark(&r.ws).unwrap().starts_with("session a\n"));
    hook("work", "SessionEnd", "a", &r.ws);
    assert!(mark(&r.ws).is_none());
}

// spec: SWT-2
#[test]
fn work_without_a_mark_writes_nothing() {
    let r = repo();
    hook("work", "PostToolUse", "a", &r.main);
    assert!(mark(&r.main).is_none());
}

// spec: SWT-2
#[test]
fn work_from_a_subdirectory_clears_the_mark_of_its_worktree() {
    let r = repo();
    let sub = r.ws.join("sub");
    std::fs::create_dir(&sub).unwrap();
    hook("wait", "Stop", "a", &sub);
    assert!(mark(&r.ws).is_some());
    hook("work", "PostToolUse", "a", &sub);
    assert!(mark(&r.ws).is_none());
}

// spec: SWT-4
#[test]
fn a_broken_input_or_place_changes_nothing() {
    let r = repo();
    let outside = r.root.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    let no_session = serde_json::json!({"cwd": r.main}).to_string();
    let no_cwd = serde_json::json!({"session_id": "a"}).to_string();
    let elsewhere = event("Stop", "a", &outside).to_string();
    let stop = event("Stop", "a", &r.main).to_string();
    for (mode, input) in [
        ("wait", "not json"),
        ("wait", no_session.as_str()),
        ("wait", no_cwd.as_str()),
        ("wait", elsewhere.as_str()),
        ("other", stop.as_str()),
    ] {
        let (code, out, err) = run(mode, input);
        assert_eq!(
            (code, out.as_str(), err.as_str()),
            (Some(0), "", ""),
            "{mode} {input}"
        );
    }
    let (code, out, err) = run("", &stop);
    assert_eq!((code, out.as_str(), err.as_str()), (Some(0), "", ""));
    assert!(mark(&r.main).is_none());
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);
}

// spec: SWT-5
#[test]
fn the_shipped_settings_entries_declare_the_hooks() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../entries/.claude/settings.json"),
    )
    .unwrap();
    let entries: serde_json::Value = serde_json::from_str(&text).unwrap();
    let declared = |event: &str, command: &str| {
        entries["hooks"][event].as_array().is_some_and(|groups| {
            groups.iter().any(|g| {
                matches!(g["matcher"].as_str(), None | Some("" | "*"))
                    && g["hooks"]
                        .as_array()
                        .is_some_and(|h| h.iter().any(|h| h["command"] == command))
            })
        })
    };
    for event in ["Stop", "Notification"] {
        assert!(declared(event, "cardinal run session-wait wait"), "{event}");
    }
    for event in ["UserPromptSubmit", "PostToolUse", "SessionEnd"] {
        assert!(declared(event, "cardinal run session-wait work"), "{event}");
    }
}
