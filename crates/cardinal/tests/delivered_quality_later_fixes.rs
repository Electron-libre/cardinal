//! Behavior test for the later fixes of `cardinal tool delivered-quality` (QUA-4): the fixes of the lines a done issue
//! wrote, followed across renames and quoted names, and of a later issue only the fix that starts it.

mod delivered_quality_common;

use delivered_quality_common::*;
use std::path::{Path, PathBuf};

/// Two issues sharing `src/a.rs`, renamed `src/b.rs` after both closed, and the fixes of the lines of each: `alpha`
/// wrote `x1` to `x3`, `beta` `y4` and `src/c "q".rs` (a name git quotes); the line `-- x0`, written before them,
/// reads `--- x0` in the diff of the fix that removes it.
fn blame_fixture(tmp: &Path) -> PathBuf {
    let repo = tmp.join("blame");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    let (alpha, beta) = ("# alpha\n\n# Tracking\n", "# beta\n\n# Tracking\n");
    let steps = [
        (
            "2026-02-01T00:00:00Z",
            "docs: open the issues",
            vec![
                ("issues/alpha.md", Some("# alpha\n")),
                ("issues/beta.md", Some("# beta\n")),
                ("src/a.rs", Some("-- x0\n")),
            ],
        ),
        (
            "2026-02-02T00:00:00Z",
            "feat: alpha",
            vec![
                ("src/a.rs", Some("-- x0\nx1\nx2\nx3\n")),
                ("issues/alpha.md", Some(alpha)),
            ],
        ),
        (
            "2026-02-03T00:00:00Z",
            "docs: close alpha",
            vec![
                ("issues/alpha.md", None),
                ("issues/done/alpha.md", Some(alpha)),
            ],
        ),
        (
            "2026-02-04T00:00:00Z",
            "feat: beta",
            vec![
                ("src/a.rs", Some("-- x0\nx1\nx2\nx3\ny4\n")),
                ("src/c \"q\".rs", Some("c1\nc2\n")),
                ("issues/beta.md", Some(beta)),
            ],
        ),
        (
            "2026-02-05T00:00:00Z",
            "docs: close beta",
            vec![
                ("issues/beta.md", None),
                ("issues/done/beta.md", Some(beta)),
            ],
        ),
        (
            "2026-02-05T12:00:00Z",
            "fix: a line of alpha in a file of beta",
            vec![("src/a.rs", Some("x1\nw2\nx3\ny4\n"))],
        ),
        (
            "2026-02-06T00:00:00Z",
            "refactor: rename",
            vec![("src/a.rs", None), ("src/b.rs", Some("x1\nw2\nx3\ny4\n"))],
        ),
        (
            "2026-02-07T00:00:00Z",
            "fix: a line of alpha",
            vec![("src/b.rs", Some("z1\nw2\nx3\ny4\n"))],
        ),
        (
            "2026-02-08T00:00:00Z",
            "fix: insert below a line of beta",
            vec![("src/c \"q\".rs", Some("c1\nnew\nc2\n"))],
        ),
        (
            "2026-02-09T00:00:00Z",
            "fix: insert at the top",
            vec![("src/c \"q\".rs", Some("top\nc1\nnew\nc2\n"))],
        ),
    ];
    for (date, message, files) in steps {
        commit_at(&repo, date, message, &files);
    }
    repo
}

// spec: QUA-4
fn blame_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = blame_fixture(tmp);
    let counted = |out: &str| {
        let r = rows(out);
        [
            has(&r, "alpha", "fixes_after", "2"),
            has(&r, "beta", "fixes_after", "1"),
        ] == [true; 2]
    };
    let (_, out) = run(dir, &repo, &[]);
    // a user's configuration that changes the diff headers and makes blame fail in a repository without the file
    let config = tmp.join("gitconfig");
    std::fs::write(
        &config,
        "[diff]\n\tnoprefix = true\n[blame]\n\tignoreRevsFile = .no-such-file\n",
    )
    .unwrap();
    let mut cmd = tool_command(dir.join("delivered_quality.rs"));
    cmd.arg(&repo)
        .arg("--projects")
        .arg(repo.join("no-projects"))
        .env("GIT_CONFIG_GLOBAL", &config);
    let configured = String::from_utf8_lossy(&cmd.output().unwrap().stdout).into_owned();
    vec![
        (
            "a fix counts only against the issue that wrote its lines, across a rename, an insertion by the line above it"
                .to_string(),
            counted(&out),
        ),
        (
            "the user's git configuration leaves the count unchanged".to_string(),
            counted(&configured),
        ),
    ]
}

/// An issue `alpha` that wrote `src/a.rs`, then the fixes of two later issues on its lines: the review fix of `beta`,
/// started by a `feat`, and the fix that starts `gamma`.
fn later_issue_fixture(tmp: &Path) -> PathBuf {
    let repo = tmp.join("later");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    let tracked = |name: &str, note: &str| format!("# {name}\n\n# Tracking\n{note}");
    let alpha = tracked("alpha", "");
    let steps = [
        (
            "2026-02-28T00:00:00Z",
            "docs: close zeta",
            vec![("issues/done/zeta.md", Some(tracked("zeta", "")))],
        ),
        (
            "2026-03-01T00:00:00Z",
            "feat: alpha",
            vec![
                ("src/a.rs", Some("x1\nx2\nx3\nx4\n".to_string())),
                ("issues/alpha.md", Some(alpha.clone())),
            ],
        ),
        (
            "2026-03-02T00:00:00Z",
            "docs: close alpha",
            vec![
                ("issues/alpha.md", None),
                ("issues/done/alpha.md", Some(alpha)),
            ],
        ),
        (
            "2026-03-03T00:00:00Z",
            "docs: open beta and gamma",
            vec![
                ("issues/beta.md", Some("# beta\n".to_string())),
                ("issues/gamma.md", Some("# gamma\n".to_string())),
            ],
        ),
        (
            "2026-03-04T00:00:00Z",
            "feat: beta",
            vec![
                ("src/b.rs", Some("b1\n".to_string())),
                ("issues/beta.md", Some(tracked("beta", "1. [x] b\n"))),
            ],
        ),
        (
            "2026-03-05T00:00:00Z",
            "fix: the review of beta",
            vec![
                ("src/a.rs", Some("w1\nx2\nx3\nx4\n".to_string())),
                ("issues/beta.md", Some(tracked("beta", "2. [x] review\n"))),
            ],
        ),
        (
            "2026-03-06T00:00:00Z",
            "fix: gamma",
            vec![
                ("src/a.rs", Some("w1\nw2\nx3\nx4\n".to_string())),
                ("issues/gamma.md", Some(tracked("gamma", "1. [x] g\n"))),
            ],
        ),
        (
            "2026-03-07T00:00:00Z",
            "fix: alpha, filing delta",
            vec![
                ("src/a.rs", Some("w1\nw2\nw3\nx4\n".to_string())),
                ("issues/delta.md", Some("# delta\n".to_string())),
            ],
        ),
        (
            "2026-03-08T00:00:00Z",
            "fix: alpha, citing the closed zeta",
            vec![
                ("src/a.rs", Some("w1\nw2\nw3\nw4\n".to_string())),
                ("issues/done/zeta.md", Some(tracked("zeta", "see w4\n"))),
            ],
        ),
    ];
    for (date, message, files) in steps {
        let files: Vec<(&str, Option<&str>)> =
            files.iter().map(|(p, b)| (*p, b.as_deref())).collect();
        commit_at(&repo, date, message, &files);
    }
    repo
}

// spec: QUA-4
fn later_issue_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = later_issue_fixture(tmp);
    let (_, out) = run(dir, &repo, &[]);
    vec![(
        "a fix of a later issue counts only when it starts that issue; one filing or citing an issue not in progress counts"
            .to_string(),
        has(&rows(&out), "alpha", "fixes_after", "3"),
    )]
}

#[test]
fn delivered_quality_later_fixes() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    let checks: Vec<_> = blame_checks(&dir, &tmp)
        .into_iter()
        .chain(later_issue_checks(&dir, &tmp))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
