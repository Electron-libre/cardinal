//! Behavior test for the rows per issue of `cardinal tool delivered-quality` (DQS-5, DQS-6, QUA-22, QUA-23): the time
//! and the tool calls of the sessions of each done issue, the lines it changed, and the rows written beside the summary.

mod delivered_quality_common;

use delivered_quality_common::*;
use std::path::{Path, PathBuf};

/// A transcript line of `kind` in `cwd` at `ts` on the default branch, with `fields` merged in.
fn line(kind: &str, cwd: &Path, ts: &str, fields: &serde_json::Value) -> String {
    let mut line = serde_json::json!({"type": kind, "cwd": cwd, "timestamp": ts, "sessionId": "x", "gitBranch": "main"});
    line.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    format!("{line}\n")
}

fn typed(cwd: &Path, ts: &str) -> String {
    line(
        "user",
        cwd,
        ts,
        &serde_json::json!({"message": {"role": "user", "content": "SECRET ask"}}),
    )
}

/// An assistant message `id` at `ts` holding one `tool_use` block per id of `tools`, with a usage of 1 output token.
fn message(cwd: &Path, ts: &str, id: &str, tools: &[&str]) -> String {
    let blocks: Vec<serde_json::Value> = tools
        .iter()
        .map(|t| serde_json::json!({"type": "tool_use", "id": t, "name": "Read", "input": {"file_path": "SECRET"}}))
        .collect();
    line(
        "assistant",
        cwd,
        ts,
        &serde_json::json!({"message": {"id": id, "content": blocks, "usage": {"input_tokens": 1, "output_tokens": 1}}}),
    )
}

fn write(path: &Path, lines: &[String]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, lines.concat()).unwrap();
}

/// A repository where `a` (closed at 09:30, renaming a file unchanged, changing two locks and 3 lines of `src/a.rs`)
/// then `b` (closed at 10:00) were made in one session on the default branch; `c` has no session.
fn fixture_rows(tmp: &Path) -> (PathBuf, PathBuf) {
    let repo = tmp.join("proj");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[
            (".claude/harness.toml", Some(&toml("r1"))),
            ("src/old.rs", Some("o1\no2\no3\no4\no5\n")),
            ("Cargo.lock", Some("l1\n")),
            ("web/package-lock.json", Some("p1\n")),
        ],
    );
    commit_at(
        &repo,
        "2026-01-01T09:30:00Z",
        "feat(files): a",
        &[
            ("src/old.rs", None),
            ("src/new.rs", Some("o1\no2\no3\no4\no5\n")),
            ("Cargo.lock", Some("l2\n")),
            ("web/package-lock.json", Some("p2\n")),
            ("src/a.rs", Some("a1\na2\na3\n")),
            ("issues/done/a.md", Some("# a\n\n# Tracking\n")),
        ],
    );
    commit_at(
        &repo,
        "2026-01-01T10:00:00Z",
        "feat(files): b",
        &[
            ("src/b.rs", Some("b1\n")),
            ("issues/done/b.md", Some("# b\n\n# Tracking\n")),
        ],
    );
    commit_at(
        &repo,
        "2026-01-02T00:00:00Z",
        "feat(files): c",
        &[("issues/done/c.md", Some("# c\n\n# Tracking\n"))],
    );
    let projects = tmp.join("projects");
    write(
        &projects.join("-proj/s1.jsonl"),
        &[
            typed(&repo, "2026-01-01T09:00:00Z"),
            message(&repo, "2026-01-01T09:10:00Z", "m1", &["t1"]),
            message(&repo, "2026-01-01T09:30:00Z", "m2", &[]),
            typed(&repo, "2026-01-01T09:40:00Z"),
            message(&repo, "2026-01-01T10:00:00Z", "m3", &["t4"]),
            typed(&repo, "2026-01-01T10:05:00Z"),
        ],
    );
    write(
        &projects.join("-proj/s1/subagents/agent-1.jsonl"),
        &[message(&repo, "2026-01-01T09:15:00Z", "m4", &["t2", "t3"])],
    );
    (repo, projects)
}

// spec: DQS-5, DQS-6, QUA-22, QUA-23, QUA-H4
fn rows_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let (repo, projects) = fixture_rows(tmp);
    let projects_arg = projects.to_str().unwrap();
    let file = tmp.join("issues.tsv");
    let rows_arg = file.to_str().unwrap();
    let (ok, summed) = run(
        dir,
        &repo,
        &["--projects", projects_arg, "--summary", "--rows", rows_arg],
    );
    let written = std::fs::read_to_string(&file).unwrap_or_default();
    let r = rows(&written);
    let (_, printed) = run(dir, &repo, &["--projects", projects_arg]);
    let (_, alone) = run(dir, &repo, &["--projects", projects_arg, "--summary"]);
    let order: Vec<&str> = written
        .lines()
        .skip(1)
        .filter_map(|l| l.split('\t').next())
        .collect();
    vec![
        ("the script runs with --rows".to_string(), ok),
        (
            "--rows writes the rows of the issues, the newest last, and leaves the summary printed"
                .to_string(),
            written == printed && order == ["a", "b", "c"] && summed == alone,
        ),
        (
            "the time of an issue runs from its first to its last event in each session"
                .to_string(),
            has(&r, "a", "duration_s", "1800") && has(&r, "b", "duration_s", "1200"),
        ),
        (
            "the tool calls of an issue include its sub-agents".to_string(),
            has(&r, "a", "tool_calls", "3") && has(&r, "b", "tool_calls", "1"),
        ),
        (
            "the changed lines leave out a rename, the lock and the issue files".to_string(),
            has(&r, "a", "changed_lines", "3") && has(&r, "b", "changed_lines", "1"),
        ),
        (
            "an issue with no session has an empty time and empty tool calls".to_string(),
            has(&r, "c", "duration_s", "")
                && has(&r, "c", "tool_calls", "")
                && has(&r, "c", "changed_lines", "0"),
        ),
        (
            "the summary holds none of the three columns".to_string(),
            !summed
                .lines()
                .next()
                .unwrap_or_default()
                .contains("duration_s")
                && !summed.contains("tool_calls")
                && !summed.contains("changed_lines"),
        ),
        (
            "no text of a message is written".to_string(),
            !written.contains("SECRET"),
        ),
    ]
}

// spec: QUA-23
fn refusal_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = tmp.join("proj");
    let missing = tmp.join("missing/issues.tsv");
    // `run` adds `--projects` after the arguments: `--rows` goes last by hand
    let no_value = tool_command(dir.join("delivered_quality.rs"))
        .arg(&repo)
        .args([
            "--projects",
            repo.join("no-projects").to_str().unwrap(),
            "--rows",
        ])
        .output()
        .unwrap()
        .status
        .success();
    let (unwritable, _) = run(dir, &repo, &["--rows", missing.to_str().unwrap()]);
    vec![
        ("--rows with no value is refused".to_string(), !no_value),
        (
            "a rows file that cannot be written fails the script".to_string(),
            !unwritable,
        ),
    ]
}

#[test]
fn delivered_quality_rows() {
    let dir = PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    let checks: Vec<_> = rows_checks(&dir, &tmp)
        .into_iter()
        .chain(refusal_checks(&dir, &tmp))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
