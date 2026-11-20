//! Behavior test for the `milestone` column of `cardinal tool delivered-quality` (QUA-19): the milestone of the
//! `roadmap` documents that lists each done issue, in its rows and its summary.

use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed");
}

fn put(repo: &Path, rel: &str, text: &str) {
    let path = repo.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// A repository whose `ROADMAP.md` lists `a` in `One` and whose `ROADMAP-done.md` lists `a` and `b` in `Two`, with
/// the done issues `a`, `b` and `c`, each closed in a commit of its own.
fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    git(repo, &["init", "-q", "-b", "main", "."]);
    put(
        repo,
        ".claude/cardinal.toml",
        "[project]\nroadmap = [\"ROADMAP.md\", \"ROADMAP-done.md\"]\n",
    );
    put(
        repo,
        "ROADMAP.md",
        "# Roadmap\n\n## One\n\n- State: done\n- Issues:\n  - [a](issues/done/a.md)\n",
    );
    put(
        repo,
        "ROADMAP-done.md",
        "# Done\n\n## Two\n\n- State: done\n- Issues:\n  - [a](issues/a.md)\n  - [b](issues/done/b.md)\n",
    );
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-qm", "chore: roadmap", "--no-verify"]);
    for stem in ["a", "b", "c"] {
        put(
            repo,
            &format!("issues/done/{stem}.md"),
            &format!("# {stem}\n\n# Tracking\n\n1. [x] one\n"),
        );
        put(repo, &format!("src/{stem}.rs"), "x\n");
        git(repo, &["add", "-A"]);
        git(
            repo,
            &["commit", "-qm", &format!("feat: {stem}"), "--no-verify"],
        );
    }
    dir
}

/// The stdout of `delivered-quality <repo> <args>`, with no transcripts.
fn quality(repo: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "delivered-quality"])
        .arg(repo)
        .args(args)
        .arg("--projects")
        .arg(repo.join("no-projects"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The cells of `column` of the lines of `out`, keyed by the cells of `key`.
fn column(out: &str, key: &str, column: &str) -> Vec<(String, String)> {
    let mut lines = out.lines();
    let header: Vec<&str> = lines.next().unwrap().split('\t').collect();
    let at = |name: &str| header.iter().position(|c| *c == name).unwrap();
    let (k, c) = (at(key), at(column));
    lines
        .map(|l| {
            let cells: Vec<&str> = l.split('\t').collect();
            (cells[k].to_string(), cells[c].to_string())
        })
        .collect()
}

// spec: QUA-19
#[test]
fn each_issue_reads_the_first_milestone_that_lists_it() {
    let dir = repository();
    let out = quality(dir.path(), &[]);
    let header: Vec<&str> = out.lines().next().unwrap().split('\t').collect();
    assert_eq!(&header[..4], ["issue", "revision", "setup", "milestone"]);
    assert_eq!(
        column(&out, "issue", "milestone"),
        [
            ("a".to_string(), "One".to_string()),
            ("b".to_string(), "Two".to_string()),
            ("c".to_string(), String::new()),
        ]
    );
}

// spec: QUA-19
#[test]
fn the_summary_sums_each_milestone_apart() {
    let dir = repository();
    let out = quality(dir.path(), &["--summary"]);
    let header: Vec<&str> = out.lines().next().unwrap().split('\t').collect();
    assert_eq!(&header[..4], ["revision", "setup", "milestone", "issues"]);
    assert_eq!(
        column(&out, "milestone", "issues"),
        [
            (String::new(), "1".to_string()),
            ("One".to_string(), "1".to_string()),
            ("Two".to_string(), "1".to_string()),
        ]
    );
}

// spec: QUA-19
#[test]
fn no_configuration_leaves_every_milestone_empty() {
    let dir = repository();
    std::fs::remove_file(dir.path().join(".claude/cardinal.toml")).unwrap();
    let out = quality(dir.path(), &[]);
    assert!(column(&out, "issue", "milestone")
        .iter()
        .all(|(_, m)| m.is_empty()));
}
