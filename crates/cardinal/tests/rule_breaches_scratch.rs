//! Behavior test for rule `scratch-outside-tmpdir` of `cardinal tool rule-breaches` (BRE-24): the literal `/tmp/<name>`
//! paths a `Bash` call writes to, not those it reads nor the scratch directory of a session, and no finding in a
//! project that turned off the feature `session-scratch`.

use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "scratch-outside-tmpdir";

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(ok, "git {args:?} failed in {}", dir.display());
}

/// A repository whose one commit holds `config` as `.claude/cardinal.toml`.
fn repo(dir: &Path, config: &str) -> PathBuf {
    std::fs::create_dir_all(dir.join(".claude")).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    std::fs::write(dir.join(".claude/cardinal.toml"), config).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "chore: sync"]);
    dir.to_path_buf()
}

/// Writes under `projects` a session of `repo` running each command of `commands`, the last one failing; returns the
/// 1-based line of each call.
fn session(projects: &Path, repo: &Path, commands: &[&str]) -> Vec<usize> {
    let stamp = |mut line: serde_json::Value| {
        line["cwd"] = json!(repo);
        line["timestamp"] = json!("2026-02-01T10:00:00.000Z");
        line["sessionId"] = json!("s");
        line.to_string()
    };
    let mut lines = vec![stamp(
        json!({"type": "user", "gitBranch": "main", "message": {"role": "user", "content": "go"}}),
    )];
    let mut at = Vec::new();
    for (i, command) in commands.iter().enumerate() {
        let id = format!("c{i}");
        lines.push(stamp(json!({"type": "assistant", "message": {"id": format!("m{i}"), "content": [{"type": "tool_use", "id": id, "name": "Bash", "input": {"command": command}}]}})));
        at.push(lines.len());
        let error = i + 1 == commands.len();
        lines.push(stamp(json!({"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": id, "is_error": error, "content": "SECRET"}]}})));
    }
    let file = projects.join("-proj/s.jsonl");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, lines.join("\n") + "\n").unwrap();
    at
}

fn rule_breaches(projects: &Path, summary: bool) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.args(["tool", "rule-breaches"]).arg(projects);
    if summary {
        cmd.arg("--summary");
    }
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The `(kind, line, detail)` of each finding of the rule, sorted.
fn findings(out: &str) -> Vec<(String, usize, String)> {
    let mut found: Vec<_> = out
        .lines()
        .map(|l| l.split('\t').collect::<Vec<_>>())
        .filter(|c| c.get(3) == Some(&RULE))
        .map(|c| {
            let line = c[5].rsplit(':').next().unwrap().parse().unwrap();
            (c[4].to_string(), line, c[6].to_string())
        })
        .collect();
    found.sort();
    found
}

const CONFIG: &str = "[cardinal]\nsource = \"x\"\nrev = \"r1\"\n";

// spec: BRE-24
#[test]
fn literal_tmp_writes_are_findings() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = repo(&tmp.path().join("proj"), CONFIG);
    let projects = tmp.path().join("projects");
    let at = session(
        &projects,
        &repo,
        &[
            "mkdir -p /tmp/measure/a && cargo build --target-dir /tmp/measure/t",
            "cat /tmp/log && grep x /tmp/log2",
            "ls > /tmp/out.txt 2>/tmp/err",
            "out=$(mktemp -d) && cp x /tmp/claude-scratch-s/y",
            "git commit -m \"write to /tmp/x\" && echo '> /tmp/y'",
            "cp a b /tmp/copy && tee -a /tmp/tee.log < in && touch /tmp/t1",
            "chromium --user-data-dir=/tmp/profile && curl -o /tmp/page x && git clone u /tmp/clone",
            "rsync -a src/ /tmp/sync/ && echo x >> /tmp/append 2>&1",
        ],
    );
    let finding = |i: usize, detail: &str| ("breach".to_string(), at[i], detail.to_string());
    let mut expected = vec![
        finding(0, "/tmp/measure"),
        finding(2, "/tmp/err"),
        finding(2, "/tmp/out.txt"),
        finding(5, "/tmp/copy"),
        finding(5, "/tmp/t1"),
        finding(5, "/tmp/tee.log"),
        finding(6, "/tmp/clone"),
        finding(6, "/tmp/page"),
        finding(6, "/tmp/profile"),
        ("blocked".to_string(), at[7], "/tmp/append".to_string()),
        ("blocked".to_string(), at[7], "/tmp/sync".to_string()),
    ];
    expected.sort();
    assert_eq!(findings(&rule_breaches(&projects, false)), expected);
    let summary = rule_breaches(&projects, true);
    assert!(
        summary
            .lines()
            .any(|l| l.starts_with(&format!("{RULE}\tr1\t1\t1\t9\t2\t"))),
        "{summary}"
    );
}

// spec: BRE-24
#[test]
fn a_project_that_turned_off_session_scratch_is_not_checked() {
    let tmp = tempfile::tempdir().unwrap();
    let config = format!("{CONFIG}disabled = [\"session-scratch\"]\n");
    let repo = repo(&tmp.path().join("proj"), &config);
    let projects = tmp.path().join("projects");
    session(&projects, &repo, &["mkdir /tmp/x", "true"]);
    let (out, summary) = (
        rule_breaches(&projects, false),
        rule_breaches(&projects, true),
    );
    assert!(
        !out.contains(RULE) && !summary.contains(RULE),
        "{out}{summary}"
    );
    // the session is read: the waste rule applies to it
    assert!(summary.contains("oversized-result\t"), "{summary}");
}
