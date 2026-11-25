//! Helpers of the behavior tests of `cardinal tool delivered-quality`: the fixture repository, the runs of the tool
//! and the reading of its rows.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
pub const COLUMNS: &str = "issue\trevision\tsetup\tmilestone\tclosed\tdeliverables\tcommits\tfixes_in_issue\tfixes_after\treverts_after\treviews\tcounted_reviews\treview_findings\ttraced\ttraced_without_red\tspec_lines\tdoc_lines\tspecs\tspec_bytes\tspec_requirements\tspec_intent_chars\tspec_happy_paths\tsessions\tfailed_checks\tvalidations\tother_messages\tinput_tokens\tcache_read_tokens\toutput_tokens";

pub fn sh(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(ok, "git {args:?} failed in {}", dir.display());
}

/// Writes `files` (path, content; `None` removes it) in `repo` and commits them at `date` with `message`; returns the
/// full hash.
pub fn commit_at(repo: &Path, date: &str, message: &str, files: &[(&str, Option<&str>)]) -> String {
    for (path, body) in files {
        let file = repo.join(path);
        match body {
            Some(body) => {
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                std::fs::write(file, body).unwrap();
            }
            None => std::fs::remove_file(file).unwrap(),
        }
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
            "--allow-empty",
            "-m",
            message,
        ])
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "commit failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .unwrap();
    String::from_utf8_lossy(&head.stdout).trim().to_string()
}

pub fn toml(rev: &str) -> String {
    format!("[harness]\nsource = \"https://example.org/harness\"\nrev = \"{rev}\"\n")
}

const ALPHA: &str = "# SECRET alpha\n\nRun the code review.\n\n# Tracking\n\n## Plan\n1. [x] SECRET one\n2. [x] two\n   3. [ ] not a plan line\n\n## Decisions\n1. [x] not in the plan\n\n## Deliverable 1\n```sh\n# a comment, not a heading\n```\n| code review | low |\n- Code review: 6 findings, 1 finding dismissed.\n- code review (low): 1 finding; second review: no findings\n- Code review: no findings.\n\n# Notes\n- Code review: 4 findings, after the Tracking section.\n";

/// The commits after `alpha` is closed: fixes in and out of the window, reverts of `gap` and of `alpha`, a fix on a
/// side branch, each changing its own line of `src/a.rs`, which `alpha` wrote.
fn commits_after_close(repo: &Path, gap: &str) {
    commit_at(
        repo,
        "2026-01-10T00:00:00Z",
        "fix(files): later",
        &[("src/a.rs", Some("b1\nc2\na3\na4\na5\na6\na7\n"))],
    );
    commit_at(
        repo,
        "2026-01-10T01:00:00Z",
        "fix: docs and lock only",
        &[
            ("docs/arch.md", Some("b")),
            (".claude/harness.lock", Some("b")),
            (".claude/cardinal.lock", Some("b")),
        ],
    );
    commit_at(
        repo,
        "2026-01-11T00:00:00Z",
        "chore: undo",
        &[("src/b.rs", Some("x"))],
    );
    commit_at(
        repo,
        "2026-01-11T01:00:00Z",
        &format!("chore: undo the gap\n\nThis reverts commit {gap}."),
        &[("src/b.rs", Some("y"))],
    );
    commit_at(
        repo,
        "2026-01-12T00:00:00Z",
        "Revert \"feat(files): a\"",
        &[("src/a.rs", Some("b1\nc2\nd3\na4\na5\na6\na7\n"))],
    );
    commit_at(
        repo,
        "2026-01-13T00:00:00Z",
        "revert(files): a",
        &[("src/a.rs", Some("b1\nc2\nd3\ne4\na5\na6\na7\n"))],
    );
    commit_at(
        repo,
        "2026-02-20T00:00:00Z",
        "fix(files): too late",
        &[("src/a.rs", Some("b1\nc2\nd3\ne4\nf5\na6\na7\n"))],
    );
    commit_at(
        repo,
        "2026-01-04T23:00:00Z",
        "fix(files): rebased after the close",
        &[("src/a.rs", Some("b1\nc2\nd3\ne4\nf5\nh6\na7\n"))],
    );
    sh(repo, &["checkout", "-q", "-b", "side"]);
    commit_at(
        repo,
        "2026-01-09T00:00:00Z",
        "fix(files): on a side branch",
        &[("src/a.rs", Some("b1\nc2\nd3\ne4\nf5\nh6\ng7\n"))],
    );
    sh(repo, &["checkout", "-q", "main"]);
}

/// The fixture repository: `alpha` (three commits, one fix inside, fixes and reverts after), `beta` (closed in one
/// commit that also rewrites a citation in `alpha`), `gamma` (done in the work tree only).
pub fn fixture(tmp: &Path) -> PathBuf {
    let repo = tmp.join("proj");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/harness.toml", Some(&toml("r1")))],
    );
    commit_at(
        &repo,
        "2026-01-02T00:00:00Z",
        "docs: open the issues",
        &[
            ("issues/alpha.md", Some("# SECRET alpha\n")),
            ("issues/beta.md", Some("# beta\n")),
        ],
    );
    commit_at(
        &repo,
        "2026-01-03T00:00:00Z",
        "feat(files): SECRET a",
        &[
            ("src/a.rs", Some("a1\na2\na3\na4\na5\na6\na7\n")),
            ("docs/arch.md", Some("a")),
            (".claude/harness.lock", Some("a")),
            (".claude/cardinal.lock", Some("a")),
            ("issues/alpha.md", Some(ALPHA)),
        ],
    );
    let gap = commit_at(
        &repo,
        "2026-01-04T00:00:00Z",
        "fix(files): gap",
        &[
            ("src/a.rs", Some("b1\na2\na3\na4\na5\na6\na7\n")),
            ("issues/alpha.md", Some(&format!("{ALPHA}x\n"))),
        ],
    );
    commit_at(
        &repo,
        "2026-01-04T06:00:00Z",
        "chore: bump",
        &[(".claude/harness.toml", Some(&toml("r2")))],
    );
    commit_at(&repo, "2026-01-04T12:00:00Z", "docs: close beta", &[
        ("issues/beta.md", None),
        ("issues/done/beta.md", Some("# beta\n\n# Tracking\n\n1. [x] one\n2. [x] two\n\n## Decisions\n1. [x] not in the plan\n* Code review: 2 findings.\n* Code review: fixed the doc.\n")),
        ("issues/alpha.md", Some(&format!("{ALPHA}y\n"))),
        (".claude/harness.toml", Some(&toml("r3"))),
    ]);
    commit_at(
        &repo,
        "2026-01-05T00:00:00Z",
        "docs: close alpha",
        &[
            ("issues/alpha.md", None),
            ("issues/done/alpha.md", Some(ALPHA)),
        ],
    );
    commits_after_close(&repo, &gap);
    std::fs::write(repo.join("issues/done/gamma.md"), "# gamma\n\n# Tracking\n").unwrap();
    repo
}

/// Runs the script on `repo`; with no `--projects` in `args`, on a projects directory that does not exist, so the
/// real transcripts are never read.
pub fn run(dir: &Path, repo: &Path, args: &[&str]) -> (bool, String) {
    let mut cmd = tool_command(dir.join("delivered_quality.rs"));
    cmd.arg(repo).args(args);
    if !args.contains(&"--projects") {
        cmd.arg("--projects").arg(repo.join("no-projects"));
    }
    let out = cmd.output().unwrap();
    if !out.status.success() {
        println!("{}", String::from_utf8_lossy(&out.stderr));
    }
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// The rows of `out` by issue, each a map from column to value.
pub fn rows(out: &str) -> HashMap<String, HashMap<String, String>> {
    let columns: Vec<&str> = COLUMNS.split('\t').collect();
    out.lines()
        .skip(1)
        .map(|l| {
            let row: HashMap<String, String> = columns
                .iter()
                .zip(l.split('\t'))
                .map(|(c, v)| (c.to_string(), v.to_string()))
                .collect();
            (row["issue"].clone(), row)
        })
        .collect()
}

pub fn has(
    rows: &HashMap<String, HashMap<String, String>>,
    issue: &str,
    column: &str,
    value: &str,
) -> bool {
    let got = rows
        .get(issue)
        .and_then(|r| r.get(column))
        .map(String::as_str);
    if got != Some(value) {
        println!("{issue}.{column}: expected {value:?}, got {got:?}");
    }
    got == Some(value)
}

/// `cardinal tool <name>` for the script path the checks were written against (`next_issue.rs` is `next-issue`).
pub fn tool_command(script: impl AsRef<std::path::Path>) -> std::process::Command {
    let stem = script
        .as_ref()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .replace('_', "-");
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.arg("tool").arg(stem);
    cmd
}
