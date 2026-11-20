//! Behavior test for `cardinal tool next-issue --json`: the document a program reads instead of the text.

use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args([
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok, "git {args:?} failed");
}

/// A git repository holding `issues/<stem>.md` for each `(stem, text)`, committed on `trunk`.
fn repo(issues: &[(&str, &str)]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("issues/done")).unwrap();
    for (stem, text) in issues {
        std::fs::write(root.path().join(format!("issues/{stem}.md")), text).unwrap();
    }
    git(root.path(), &["init", "-q", "-b", "trunk", "."]);
    git(root.path(), &["add", "-A"]);
    git(
        root.path(),
        &["commit", "-qm", "init", "--no-verify", "--allow-empty"],
    );
    root
}

/// `cardinal tool <args>` in `dir` as the session `s` of the process `pid`: exit code, stdout, stderr.
fn tool(dir: &Path, pid: &str, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .arg("tool")
        .args(args)
        .current_dir(dir)
        .env("CLAUDE_CODE_SESSION_ID", "s")
        .env("CLAUDE_PID", pid)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The document `next-issue --json` prints in `dir`, checked to be one line, and its exit code.
fn document(dir: &Path) -> (Value, Option<i32>, String) {
    let (code, out, err) = tool(dir, "", &["next-issue", "--json"]);
    assert_eq!(out.lines().count(), 1, "one line: {out}");
    (serde_json::from_str(&out).expect(&out), code, err)
}

/// The issue `stem` of a document.
fn issue<'a>(doc: &'a Value, stem: &str) -> &'a Value {
    doc["issues"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["stem"] == stem)
        .unwrap_or_else(|| panic!("no {stem} in {doc}"))
}

// spec: NXT-19
#[test]
fn no_open_issue_prints_an_empty_queue() {
    let r = repo(&[]);
    let (code, out, _) = tool(r.path(), "", &["next-issue", "--json"]);
    assert_eq!(out, "{\"version\":1,\"next\":null,\"issues\":[]}\n");
    assert_eq!(code, Some(0));
}

// spec: NXT-18
#[test]
fn the_problems_stay_on_stderr_and_in_the_exit_code() {
    let r = repo(&[("a", "# Title a\n")]);
    let (doc, code, err) = document(r.path());
    assert_eq!(issue(&doc, "a")["priority"], Value::Null);
    assert!(err.contains("issue without a priority: a"), "{err}");
    assert_eq!(code, Some(1));
}

// spec: NXT-19
#[test]
fn each_issue_has_its_state_in_the_order_of_the_text() {
    let r = repo(&[
        (
            "a",
            "---\npriority: high\ndepends_on: [b]\nwhy: 'needs b'\n---\n# Title a\n",
        ),
        (
            "b",
            "---\npriority: low\n---\n# Title b\n\n# Milestone\nOne\n",
        ),
        (
            "c",
            "---\npriority: medium\non_hold: later\n---\n# Title c\n",
        ),
    ]);
    let (doc, code, _) = document(r.path());
    assert_eq!(code, Some(0));
    assert_eq!(doc["version"], 1);
    assert_eq!(doc["next"], "b");
    let stems: Vec<&str> = doc["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["stem"].as_str().unwrap())
        .collect();
    assert_eq!(stems, ["b", "a", "c"]);
    assert_eq!(
        *issue(&doc, "a"),
        json!({
            "stem": "a", "title": "Title a", "priority": "high", "why": "needs b", "milestone": null,
            "depends_on": ["b"], "state": "blocked", "blocked_by": ["b"], "hold": null, "taken": null,
            "stale": false
        })
    );
    let b = issue(&doc, "b");
    assert_eq!(b["state"], "ready");
    assert_eq!(b["milestone"], "One");
    assert_eq!(b["why"], "");
    assert_eq!(b["depends_on"], json!([]));
    let c = issue(&doc, "c");
    assert_eq!(c["state"], "on hold");
    assert_eq!(c["hold"], "later");
}

// spec: NXT-19
#[test]
fn an_invalid_depends_on_is_null_and_blocks() {
    let r = repo(&[("a", "---\npriority: low\ndepends_on: b\n---\n# Title a\n")]);
    let (doc, code, _) = document(r.path());
    let a = issue(&doc, "a");
    assert_eq!(a["depends_on"], Value::Null);
    assert_eq!(a["state"], "blocked");
    assert_eq!(a["blocked_by"], json!([]));
    assert_eq!(doc["next"], Value::Null);
    assert_eq!(code, Some(1));
}

// spec: NXT-H2, NXT-18, NXT-19
#[test]
fn a_program_reads_the_next_issue_and_the_session_that_claimed_another() {
    let r = repo(&[
        ("a", "---\npriority: high\n---\n# Title a\n"),
        ("b", "---\npriority: low\n---\n# Title b\n"),
    ]);
    let live = std::process::id().to_string();
    let (code, out, err) = tool(r.path(), &live, &["issue-claim", "issues/a.md"]);
    assert_eq!(code, Some(0), "{out}{err}");
    let (doc, _, _) = document(r.path());
    assert_eq!(doc["next"], "b");
    let taken = &issue(&doc, "a")["taken"];
    assert_eq!(taken["by"], "claim");
    assert_eq!(taken["state"], "working");
    assert_eq!(taken["branch"], "trunk");
    assert_eq!(taken["session_alive"], true);
    let top = std::fs::canonicalize(r.path()).unwrap();
    assert_eq!(taken["worktree"], top.to_str().unwrap());
    assert_eq!(issue(&doc, "b")["taken"], Value::Null);
}

// spec: NXT-19
#[test]
fn a_claim_whose_session_ended_offers_nothing() {
    let r = repo(&[("a", "---\npriority: high\n---\n# Title a\n")]);
    let (code, out, err) = tool(r.path(), "2147483646", &["issue-claim", "issues/a.md"]);
    assert_eq!(code, Some(0), "{out}{err}");
    let (doc, _, _) = document(r.path());
    assert_eq!(doc["next"], Value::Null);
    assert_eq!(issue(&doc, "a")["taken"]["session_alive"], false);
}

// spec: NXT-19
#[test]
fn a_branch_named_after_an_issue_takes_it_or_is_stale() {
    let r = repo(&[
        ("a", "---\npriority: high\n---\n# Title a\n"),
        ("b", "---\npriority: low\n---\n# Title b\n"),
    ]);
    git(r.path(), &["checkout", "-q", "-b", "a"]);
    std::fs::write(r.path().join("work.txt"), "work\n").unwrap();
    git(r.path(), &["add", "-A"]);
    git(r.path(), &["commit", "-qm", "a", "--no-verify"]);
    git(r.path(), &["checkout", "-q", "trunk"]);
    git(r.path(), &["branch", "b"]);
    let (doc, _, _) = document(r.path());
    assert_eq!(
        issue(&doc, "a")["taken"],
        json!({"by": "branch", "branch": "a", "worktree": null})
    );
    assert_eq!(issue(&doc, "a")["stale"], false);
    assert_eq!(issue(&doc, "b")["taken"], Value::Null);
    assert_eq!(issue(&doc, "b")["stale"], true);
    assert_eq!(doc["next"], "b");
}
