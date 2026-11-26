//! Behavior test for the revision column of `cardinal tool session-metrics` (EVA-8): the pin in force at the start of
//! a session, on the default branch or merged into it, through the old and the new configuration file.

mod session_metrics_common;

use session_metrics_common::*;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Commits `files` (path, content) in `repo` at `date` (committer and author); returns the commit hash.
fn commit_at(repo: &Path, date: &str, files: &[(&str, &str)]) -> String {
    for (path, body) in files {
        let file = repo.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, body).unwrap();
    }
    sh(repo, &["add", "-A", "--force"]);
    let out = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "-m",
            date,
        ])
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "commit failed in {}: {}{}",
        repo.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .unwrap();
    String::from_utf8_lossy(&head.stdout).trim().to_string()
}

fn new_repo(dir: &Path) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    sh(dir, &["init", "-q", "-b", "main"]);
    dir.to_path_buf()
}

/// A `.claude/harness.toml` pinning `rev`.
fn toml(rev: &str) -> String {
    format!("[harness]\nsource = \"https://example.org/harness\"\nrev = \"{rev}\"  # v1\n")
}

/// A repository pinning `aaa` then `bbb` on `main`, `ddd` merged from a branch, and a linked worktree pinning `ccc`;
/// returns (repository, worktree).
fn pinned_repo(tmp: &Path) -> (PathBuf, PathBuf) {
    let pinned = new_repo(&tmp.join("pinned"));
    commit_at(
        &pinned,
        "2026-01-01T10:00:00Z",
        &[(".claude/harness.toml", &toml("aaa"))],
    );
    commit_at(
        &pinned,
        "2026-01-01T12:00:00Z",
        &[(".claude/harness.toml", &toml("bbb"))],
    );
    let worktree = tmp.join("pinned-branch");
    sh(
        &pinned,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "branch",
            worktree.to_str().unwrap(),
        ],
    );
    commit_at(
        &worktree,
        "2026-01-01T12:30:00Z",
        &[(".claude/harness.toml", &toml("ccc"))],
    );
    sh(&pinned, &["checkout", "-q", "-b", "bump"]);
    commit_at(
        &pinned,
        "2026-01-01T12:15:00Z",
        &[(".claude/harness.toml", &toml("ddd"))],
    );
    sh(&pinned, &["checkout", "-q", "main"]);
    let merge = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "merge",
            "-q",
            "--no-ff",
            "-m",
            "merge",
            "bump",
        ])
        .env("GIT_AUTHOR_DATE", "2026-01-01T14:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-01-01T14:00:00Z")
        .current_dir(&pinned)
        .output()
        .unwrap();
    assert!(merge.status.success(), "merge failed");
    (pinned, worktree)
}

/// A repository pinning `old` in `.claude/harness.toml`, then `new` in `.claude/cardinal.toml`.
fn migrated_repo(tmp: &Path) -> PathBuf {
    let migrated = new_repo(&tmp.join("migrated"));
    commit_at(
        &migrated,
        "2026-01-01T10:00:00Z",
        &[(".claude/harness.toml", &toml("old"))],
    );
    std::fs::remove_file(migrated.join(".claude/harness.toml")).unwrap();
    commit_at(
        &migrated,
        "2026-01-01T12:00:00Z",
        &[(
            ".claude/cardinal.toml",
            &toml("new").replace("[harness]", "[cardinal]"),
        )],
    );
    migrated
}

// spec: EVA-8
fn revision_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let tmp = tmp.join("revision");
    let (pinned, worktree) = pinned_repo(&tmp);
    let own = new_repo(&tmp.join("own"));
    commit_at(
        &own,
        "2026-01-01T10:00:00Z",
        &[
            (".claude/harness.toml", "[harness]\nsource = \".\"\n"),
            ("files/a", "1"),
        ],
    );
    let files_commit = commit_at(&own, "2026-01-01T11:00:00Z", &[("files/a", "2")]);
    commit_at(&own, "2026-01-01T11:10:00Z", &[("other", "x")]);
    let unpinned = new_repo(&tmp.join("unpinned"));
    commit_at(
        &unpinned,
        "2026-01-01T10:00:00Z",
        &[(
            ".claude/harness.toml",
            "[harness]\nsource = \"https://example.org/harness\"\n",
        )],
    );
    let migrated = migrated_repo(&tmp);
    let projects = tmp.join("projects");
    let sessions: [(&str, &Path, &str); 10] = [
        ("before_migration", &migrated, "2026-01-01T11:00:00.000Z"),
        ("after_migration", &migrated, "2026-01-01T13:00:00.000Z"),
        ("before", &pinned, "2026-01-01T09:00:00.000Z"),
        ("first", &pinned, "2026-01-01T11:00:00.000Z"),
        ("second", &pinned, "2026-01-01T13:00:00.000Z"),
        ("merged", &pinned, "2026-01-01T15:00:00.000Z"),
        ("on_branch", &worktree, "2026-01-01T13:00:00.000Z"),
        ("own", &own, "2026-01-01T11:30:00.000Z"),
        ("unpinned", &unpinned, "2026-01-01T11:00:00.000Z"),
        (
            "gone",
            Path::new("/nonexistent/gone"),
            "2026-01-01T11:00:00.000Z",
        ),
    ];
    for (id, cwd, ts) in sessions {
        let proj = projects.join(format!("-{id}"));
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join(format!("{id}.jsonl")), typed(cwd, ts, "ok")).unwrap();
    }
    let out = tool_command(dir.join("session_metrics.rs"))
        .arg(&projects)
        .output()
        .unwrap();
    let rows = rows(&String::from_utf8_lossy(&out.stdout)).unwrap_or_default();
    let rev = |id: &str| {
        rows.get(id)
            .and_then(|r| r.get("revision"))
            .cloned()
            .unwrap_or_else(|| "<none>".into())
    };
    vec![
        ("no revision before the first pin", rev("before").is_empty()),
        (
            "the pin in force at the start of the session",
            rev("first") == "aaa",
        ),
        (
            "a later pin applies to a later session, not one merged later",
            rev("second") == "bbb",
        ),
        (
            "a pin merged into the default branch applies after the merge",
            rev("merged") == "ddd",
        ),
        (
            "a linked worktree reads the pin of the default branch",
            rev("on_branch") == "bbb",
        ),
        (
            "the harness repository itself gives its last commit touching files/",
            rev("own") == format!("self:{files_commit}"),
        ),
        (
            "a harness.toml with no rev is unpinned",
            rev("unpinned") == "unpinned",
        ),
        (
            "an unreadable project gives an empty revision",
            rev("gone").is_empty(),
        ),
        (
            "the old configuration pins before the migration",
            rev("before_migration") == "old",
        ),
        (
            "the new configuration pins after the migration",
            rev("after_migration") == "new",
        ),
    ]
}

#[test]
fn revision_column() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    assert_checks(revision_checks(&dir, &tmp));
}
