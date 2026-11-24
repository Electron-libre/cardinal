//! Behavior test for `cardinal tool release --json`: the release proposal a program reads instead of the text.

use std::path::Path;
use std::process::{Command, Output};

use serde_json::{json, Value};
use tempfile::TempDir;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args([
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "tag.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed: {out:?}");
}

/// A repository on `main` with one first commit.
fn repo() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main", "."]);
    commit(dir.path(), "chore: start", "start.txt");
    dir
}

/// Commit a change of the file `file` with the message `message`.
fn commit(dir: &Path, message: &str, file: &str) {
    let path = dir.join(file);
    let before = std::fs::read_to_string(&path).unwrap_or_default();
    std::fs::write(&path, format!("{before}{message}\n")).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", message, "--no-verify"]);
}

fn tag(dir: &Path, name: &str) {
    git(dir, &["tag", "-a", name, "-m", name]);
}

fn config(dir: &Path, text: &str) {
    std::fs::create_dir_all(dir.join(".claude")).unwrap();
    std::fs::write(dir.join(".claude/cardinal.toml"), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", "chore: configure", "--no-verify"]);
}

fn release(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "release"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

/// The document `release --json` prints, checked to be one line.
fn document(dir: &Path) -> Value {
    let out = release(dir, &["--json"]);
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text.trim_end().lines().count(), 1, "{text}");
    serde_json::from_str(&text).unwrap()
}

/// The status of the work tree, to prove nothing was written.
fn status(dir: &Path) -> String {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap()
}

// spec: REL-14, REL-15, REL-H2
#[test]
fn prints_the_version_the_milestones_and_the_commits_of_the_next_release() {
    let dir = repo();
    std::fs::write(
        dir.path().join("ROADMAP.md"),
        "# Roadmap\n\n## Old\n\n- State: done\n- Released: v0.2.0\n\n## New\n\n- State: done\n\n## Later\n\n- State: in issues\n",
    )
    .unwrap();
    config(dir.path(), "[project]\nroadmap = [\"ROADMAP.md\"]\n");
    tag(dir.path(), "v0.2.0");
    commit(dir.path(), "fix: a", "a.txt");
    commit(dir.path(), "feat: b", "b.txt");
    let out = release(dir.path(), &["--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "{\"version\":1,\"proposed\":\"0.3.0\",\"prepared\":false,\"last\":\"0.2.0\",\"commits\":2,\"milestones\":[{\"name\":\"New\",\"document\":\"ROADMAP.md\"}]}\n"
    );
    assert_eq!(status(dir.path()), "");
}

// spec: REL-15
#[test]
fn proposes_nothing_when_no_commit_follows_the_last_tag() {
    let dir = repo();
    tag(dir.path(), "v0.3.0");
    assert_eq!(
        document(dir.path()),
        json!({"version": 1, "proposed": null, "prepared": false, "last": "0.3.0", "commits": 0, "milestones": []})
    );
}

// spec: REL-15
#[test]
fn with_no_tag_counts_every_commit_of_head() {
    let dir = repo();
    commit(dir.path(), "feat: b", "b.txt");
    let doc = document(dir.path());
    assert_eq!(doc["proposed"], "0.1.0");
    assert_eq!(doc["last"], Value::Null);
    assert_eq!(doc["commits"], 2);
}

// spec: REL-16
#[test]
fn refuses_a_version_a_tag_or_a_refused_archive() {
    let dir = repo();
    tag(dir.path(), "v0.2.0");
    commit(dir.path(), "feat: add a flag", "a.txt");
    for args in [
        &["--json", "0.3.0"][..],
        &["--json", "--tag"],
        &["0.3.0", "--json"],
    ] {
        let out = release(dir.path(), args);
        assert!(!out.status.success(), "{args:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{args:?}: {out:?}");
        assert_eq!(status(dir.path()), "", "{args:?}");
    }
    config(
        dir.path(),
        "[project]\nroadmap = [\"ROADMAP.md\"]\nroadmap_archive = \"ROADMAP-done.md\"\n",
    );
    let out = release(dir.path(), &["--json"]);
    assert!(!out.status.success(), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
}

// spec: REL-15
#[test]
fn a_prepared_release_is_marked_so() {
    let dir = repo();
    std::fs::write(
        dir.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.1.0] - 2026-01-01\n\nFirst numbered release.\n",
    )
    .unwrap();
    git(dir.path(), &["add", "-A"]);
    git(
        dir.path(),
        &["commit", "-qm", "chore: release 0.1.0", "--no-verify"],
    );
    let doc = document(dir.path());
    assert_eq!(doc["proposed"], "0.1.0");
    assert_eq!(doc["prepared"], true);
}

// spec: REL-15
#[test]
fn proposes_nothing_when_the_proposed_tag_exists_elsewhere() {
    let dir = repo();
    tag(dir.path(), "v0.2.0");
    git(dir.path(), &["checkout", "-q", "-b", "other"]);
    commit(dir.path(), "feat: b", "b.txt");
    tag(dir.path(), "v0.3.0");
    git(dir.path(), &["checkout", "-q", "main"]);
    commit(dir.path(), "feat: c", "c.txt");
    let doc = document(dir.path());
    assert_eq!(doc["proposed"], Value::Null);
    assert_eq!(doc["last"], "0.2.0");
}

// spec: REL-14
#[test]
fn prints_the_warnings_of_the_convention_on_stderr_only() {
    let dir = repo();
    config(
        dir.path(),
        "[cardinal]\ndisabled = [\"conventional-commits\"]\n",
    );
    tag(dir.path(), "v0.2.0");
    commit(dir.path(), "Add b", "b.txt");
    let out = release(dir.path(), &["--json"]);
    assert!(out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("conventional-commits is off"),
        "{out:?}"
    );
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["proposed"], "0.2.1");
}

// spec: REL-16
#[test]
fn refuses_a_configuration_it_cannot_parse() {
    for text in [
        "[project\n",
        "[project]\nrelease_breaking_files = [\"cardinal-format\", 1]\n",
        "[project]\nroadmap = [\"ROADMAP.md\"]\nroadmap_archive = 1\n",
    ] {
        let dir = repo();
        config(dir.path(), text);
        tag(dir.path(), "v0.2.0");
        commit(dir.path(), "feat: b", "b.txt");
        let out = release(dir.path(), &["--json"]);
        assert!(!out.status.success(), "{text:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{text:?}: {out:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("cardinal.toml"),
            "{text:?}: {out:?}"
        );
    }
}
