//! Behavior test for `cardinal-ui --print`: the worktrees panel, against throwaway git repositories and a fake
//! `cardinal` (STU-5) whose answers each test writes.

mod common;

use common::{git, line, ok, project, ui, ui_env, Project, QUEUE, ROADMAP};
use std::process::Command;

// spec: STU-1, STU-7
#[test]
fn an_ended_session_is_shown_ended() {
    let p = project();
    let queue = QUEUE
        .replace("WS", p.ws.to_str().unwrap())
        .replace("\"session_alive\":true", "\"session_alive\":false");
    let cardinal = p.cardinal(&ok(&queue), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", "2147483646");
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(line(&out, "  a  ").contains("session ended"), "{out}");
    assert!(
        line(&out, &format!("│{}  ", p.ws.display())).contains("session s1 ended"),
        "{out}"
    );
}

// spec: STU-7
#[test]
fn a_session_whose_pid_a_later_process_took_is_shown_ended() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", &std::process::id().to_string());
    let lock = common::git_path(&p.ws, "cardinal-session");
    // long before the test binary started: the session that wrote it ended and the test took its number
    std::fs::File::options()
        .write(true)
        .open(&lock)
        .unwrap()
        .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_hours(24))
        .unwrap();
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(
        line(&out, &format!("│{}  ", p.ws.display())).contains("session s1 ended"),
        "{out}"
    );
}

// spec: STU-8
#[test]
fn outside_a_repository_the_worktrees_panel_shows_an_error() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let outside = p.root.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
        .arg("--print")
        .current_dir(&outside)
        .env("CARDINAL_UI_CARDINAL", &cardinal)
        .env("GIT_CEILING_DIRECTORIES", p.root.path())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("git worktree list: exit"), "{text}");
    assert_eq!(out.status.code(), Some(1));
}

// spec: STU-5
#[test]
fn from_a_subdirectory_the_tools_run_at_the_root() {
    let p = project();
    let sub = p.repo.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let (code, out, err) = ui(&sub, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("> b"), "{out}");
    let missing = p.root.path().join("them.toml");
    let (code, _, err) = ui_env(
        &p.repo,
        &cardinal,
        &[("CARDINAL_UI_THEME", Some(&missing))],
        &["--print"],
    );
    assert_eq!(code, Some(0), "{err}");
    assert!(
        err.starts_with(&format!("theme: {}: ", missing.display())),
        "a file the variable names is never missing in silence: {err}"
    );
}

// spec: STU-7
#[test]
fn a_lock_without_a_pid_is_free() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let path = git(
        &p.ws,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "cardinal-session",
        ],
    );
    std::fs::write(path, "session s1\n").unwrap();
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(
        line(&out, &format!("│{}  ", p.ws.display())).contains("free"),
        "{out}"
    );
}

/// 2025-01-02 09:05:00 UTC.
const NINE_O_FIVE: u64 = 1_735_808_700;

// spec: STU-H3, STU-13
#[test]
fn a_session_that_waits_for_the_user_is_flagged() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", &std::process::id().to_string());
    Project::waiting(&p.ws, "s1", NINE_O_FIVE, "stop");
    let (code, out, err) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    let ws = line(&out, &format!("│{}  ", p.ws.display()));
    assert!(
        ws.contains("session s1  waiting since 2025-01-02 09:05: stop"),
        "a day other than today shows its date: {ws}"
    );
    assert!(!line(&out, &format!("│{}  ", p.repo.display())).contains("waiting"));
}

// spec: STU-13
#[test]
fn a_session_waiting_since_today_shows_the_time_alone() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", &std::process::id().to_string());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Project::waiting(&p.ws, "s1", now, "stop");
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let ws = line(&out, &format!("│{}  ", p.ws.display()));
    let at = ws.find("waiting since ").expect(ws) + "waiting since ".len();
    assert_eq!(&ws[at + 2..at + 3], ":", "HH:MM alone: {ws}");
    assert_eq!(&ws[at + 5..at + 7], ": ", "{ws}");
}

// spec: STU-13
#[test]
fn a_mark_of_an_ended_or_another_session_is_not_shown() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", "2147483646");
    Project::waiting(&p.ws, "s1", NINE_O_FIVE, "stop");
    Project::lock(&p.repo, "s2", &std::process::id().to_string());
    Project::waiting(&p.repo, "s3", NINE_O_FIVE, "stop");
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let ws = line(&out, &format!("│{}  ", p.ws.display()));
    assert!(ws.contains("ended") && !ws.contains("waiting"), "{ws}");
    assert!(
        !line(&out, &format!("│{}  ", p.repo.display())).contains("waiting"),
        "{out}"
    );
}
