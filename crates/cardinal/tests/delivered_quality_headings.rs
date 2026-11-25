//! Behavior test for the headings of an issue file as `cardinal tool delivered-quality` reads them (QUA-12): a
//! heading names its section as `check-issue` names one (CKI-6), spaced or followed by a note, and every heading it
//! reads, a bare `#` included, ends `# Tracking` (QUA-6, QUA-11).

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

/// Writes `files` (path, content; `None` removes it) in `repo` and commits them with `message`.
fn commit(repo: &Path, message: &str, files: &[(&str, Option<&str>)]) {
    for (rel, text) in files {
        let path = repo.join(rel);
        match text {
            Some(text) => {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, text).unwrap();
            }
            None => std::fs::remove_file(path).unwrap(),
        }
    }
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-qm", message, "--no-verify"]);
}

/// The cells of the row of `issue` printed by `delivered-quality <repo>`, keyed by their column.
fn row(repo: &Path, issue: &str) -> Vec<(String, String)> {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "delivered-quality"])
        .arg(repo)
        .arg("--projects")
        .arg(repo.join("no-projects"))
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut lines = out.lines();
    let header: Vec<&str> = lines.next().unwrap().split('\t').collect();
    let cells: Vec<&str> = lines
        .map(|l| l.split('\t').collect::<Vec<_>>())
        .find(|cells| cells[0] == issue)
        .unwrap_or_else(|| panic!("no row for {issue}: {out}"));
    header
        .iter()
        .zip(cells)
        .map(|(h, c)| (h.to_string(), c.to_string()))
        .collect()
}

fn cell<'a>(row: &'a [(String, String)], column: &str) -> &'a str {
    &row.iter().find(|(h, _)| h == column).unwrap().1
}

const GOAL: &str = "# A\n\n# Goal\n\nDo it.\n";
const DONE: &str = "# A\n\n# Goal\n\nDo it.\n\n#  Tracking\n\n## Plan (two lines)\n1. [x] one\n2. [x] two\n\nCode review: 1 finding\n";

// spec: QUA-12
/// `#  Tracking` and `## Plan (two lines)` are the tracking and the plan of the issue, and the commit that adds the
/// spaced heading starts its work (QUA-2); `## Planning`, `## Plan of attack` and a title `# Tracking down a leak` are
/// none.
#[test]
fn a_spaced_or_noted_heading_names_its_section() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    git(repo, &["init", "-q", "-b", "main", "."]);
    commit(repo, "docs: open a", &[("issues/a.md", Some(GOAL))]);
    commit(
        repo,
        "docs: plan a",
        &[("issues/a.md", Some(&format!("{GOAL}\n#  Tracking\n")))],
    );
    commit(
        repo,
        "feat: a",
        &[("src/a.rs", Some("x\n")), ("issues/a.md", Some(DONE))],
    );
    commit(
        repo,
        "docs: close a",
        &[("issues/a.md", None), ("issues/done/a.md", Some(DONE))],
    );
    commit(
        repo,
        "docs: close b",
        &[(
            "issues/done/b.md",
            Some("# B\n\n# Tracking\n\n## Planning\n1. [x] one\n"),
        )],
    );
    commit(
        repo,
        "docs: close c",
        &[(
            "issues/done/c.md",
            Some("# Tracking down a leak\n\n# Approach\n\n## Plan of attack\n1. [ ] look\n2. [ ] fix\n\n# Tracking\n\n## Plan\n1. [x] one\n\nCode review: 2 findings\n"),
        )],
    );
    let c = row(repo, "c");
    assert_eq!(
        ["deliverables", "reviews", "review_findings"].map(|col| cell(&c, col)),
        ["1", "1", "2"],
        "{c:?}"
    );
    let a = row(repo, "a");
    assert_eq!(
        ["deliverables", "commits", "reviews", "review_findings"].map(|c| cell(&a, c)),
        ["2", "3", "1", "1"],
        "{a:?}"
    );
    assert_eq!(cell(&row(repo, "b"), "deliverables"), "0");
}

const BARE: &str = "# D\n\n# Goal\n\nDo it.\n\n# Tracking\n\n```sh\n#\n```\nCode review: 1 finding\n\n#  \n\nCode review: 2 findings\nred: ABC-1: assertion failed\n";

// spec: QUA-6, QUA-11
/// A bare `#` line, trailing spaces allowed, ends the `# Tracking` section, as `check-issue` reads it (CKI-6), and one
/// inside a fence does not (QUA-12): the review and the `red:` item after it are not the issue's.
#[test]
fn a_bare_hash_line_ends_the_tracking() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    git(repo, &["init", "-q", "-b", "main", "."]);
    commit(
        repo,
        "feat: d",
        &[
            ("issues/d.md", Some(BARE)),
            ("tests/d.rs", Some("// spec: ABC-1\nfn d() {}\n")),
        ],
    );
    commit(
        repo,
        "docs: close d",
        &[("issues/d.md", None), ("issues/done/d.md", Some(BARE))],
    );
    let d = row(repo, "d");
    assert_eq!(
        ["reviews", "review_findings", "traced", "traced_without_red"].map(|c| cell(&d, c)),
        ["1", "1", "1", "1"],
        "{d:?}"
    );
}
