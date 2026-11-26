//! Behavior test for the commit rules of `cardinal tool rule-breaches` (BRE-9, FEA-11, FEA-22): each commit checked
//! once against the session it is tied to, its attribution and its convention, and none of a project that turned the
//! rule off.

mod rule_breaches_common;

use rule_breaches_common::*;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run git with `args` in `repo` as the user `t`, at the given author and committer dates.
fn git_at(repo: &Path, author_date: &str, committer_date: &str, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .env("GIT_AUTHOR_DATE", author_date)
        .env("GIT_COMMITTER_DATE", committer_date)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed");
}

/// The repository `cm` of the commit checks; returns it with the commits that breach a rule: a long subject, an
/// unknown scope, an attribution trailer, a capitalized subject, a generated mark.
fn commit_repo(tmp: &Path) -> (PathBuf, [String; 5]) {
    let repo = new_repo(&tmp.join("cm"));
    let toml = "[harness]\nsource = \"https://example.org/harness\"\nrev = \"r1\"\n\n[project]\ncommit_scopes = [\"sync\"]\n";
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/harness.toml", toml)],
    );
    commit_at(&repo, "2026-02-01T09:00:00Z", "Not mine.", &[("a", "1")]);
    commit_at(
        &repo,
        "2026-02-01T10:15:00Z",
        "feat(sync): add x",
        &[("a", "2")],
    );
    let long = commit_at(
        &repo,
        "2026-02-01T10:25:00Z",
        &format!("feat: {}", "a".repeat(70)),
        &[("a", "3")],
    );
    let scope = commit_at(&repo, "2026-02-01T10:45:00Z", "fix(nope): y", &[("a", "4")]);
    let trailer = commit_at(
        &repo,
        "2026-02-01T10:50:00Z",
        "docs: z\n\nCo-Authored-By: Claude <noreply@anthropic.com>",
        &[("a", "5")],
    );
    let capital = commit_at(&repo, "2026-02-01T11:30:00Z", "Feat: Big.", &[("a", "6")]);
    sh(&repo, &["checkout", "-q", "-b", "other", "HEAD~3"]);
    let generated = commit_at(
        &repo,
        "2026-02-01T10:20:00Z",
        "chore(harness): w\n\nGenerated with [Claude Code]",
        &[("b", "1")],
    );
    let with_cli = toml.replace("[\"sync\"]", "[\"sync\", \"cli\"]");
    commit_at(
        &repo,
        "2026-02-01T10:21:00Z",
        "chore(sync): add the cli scope",
        &[(".claude/harness.toml", &with_cli)],
    );
    commit_at(
        &repo,
        "2026-02-01T10:22:00Z",
        "feat(cli): add x",
        &[("b", "2")],
    );
    sh(&repo, &["checkout", "-q", "main"]);
    std::fs::write(repo.join("a"), "stashed").unwrap();
    git_at(
        &repo,
        "2026-02-01T10:40:00Z",
        "2026-02-01T10:40:00Z",
        &["stash", "-q"],
    );
    std::fs::write(repo.join("c"), "1").unwrap();
    sh(&repo, &["add", "c"]);
    git_at(
        &repo,
        "2026-01-15T10:00:00Z",
        "2026-02-01T10:41:00Z",
        &["commit", "-q", "-m", "Mine, rebased."],
    );
    commit_at(
        &repo,
        "2026-02-01T11:35:00Z",
        "fix(cardinal): v",
        &[("a", "7")],
    );
    git_at(
        &repo,
        "2026-02-01T11:40:00Z",
        "2026-02-01T11:40:00Z",
        &[
            "merge",
            "-q",
            "--no-ff",
            "-m",
            "Merge branch other.",
            "other",
        ],
    );
    (repo, [long, scope, trailer, capital, generated])
}

// spec: BRE-9
fn commit_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("commits");
    let (repo, [long, scope, trailer, capital, generated]) = commit_repo(&tmp);
    let projects = tmp.join("projects");
    for (id, start, end) in [
        ("s1", "2026-02-01T10:00:00.000Z", "2026-02-01T11:00:00.000Z"),
        ("s2", "2026-02-01T10:30:00.000Z", "2026-02-01T12:00:00.000Z"),
    ] {
        let mut t = Transcript::new(&repo, start);
        t.typed("SECRET start");
        t.ts = end.to_string();
        t.typed("ok");
        t.write(&projects.join("-cm").join(format!("{id}.jsonl")));
    }
    let row = |s: &str, rule: &str, hash: &str| format!("{s}\tcm\tr1\t{rule}\tbreach\t\t{hash}");
    let expected = vec![
        row("s1", "commit-format", &long),
        row("s1", "commit-attribution", &generated),
        row("s2", "commit-format", &scope),
        row("s2", "commit-attribution", &trailer),
        row("s2", "commit-format", &capital),
    ];
    let (ok, out) = run(dir, &projects, false);
    let rows: Vec<String> = out.lines().skip(1).map(str::to_string).collect();
    let (_, summary) = run(dir, &projects, true);
    let summary: Vec<&str> = summary.lines().collect();
    let want_summary = [
        SUMMARY,
        "commit-attribution\tr1\t2\t2\t2\t0\t",
        "commit-format\tr1\t2\t2\t3\t0\t",
    ];
    if sorted(rows.clone()) != sorted(expected.clone()) || summary != want_summary {
        println!("{out}{}", summary.join("\n"));
    }
    vec![
        ("the commit rules succeed".to_string(), ok),
        ("each commit of a session is checked once, against the latest-starting session; merges, stashes and the user's commits are skipped; a scope added on the branch counts".to_string(), sorted(rows) == sorted(expected)),
        ("the commit rules apply to the sessions with a checked commit".to_string(), summary == want_summary),
    ]
}

// spec: BRE-9
fn claim_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("claims");
    let repo = new_repo(&tmp.join("cl"));
    let toml = "[cardinal]\nsource = \"x\"\nrev = \"r1\"\n";
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/cardinal.toml", toml)],
    );
    let trailer = "\n\nCo-Authored-By: Claude <noreply@anthropic.com>";
    let claimed = commit_at(
        &repo,
        "2026-02-01T11:00:00Z",
        &format!("fix: alpha{trailer}"),
        &[("issues/élan.md", "# Élan\n")],
    );
    let other = commit_at(
        &repo,
        "2026-02-01T11:10:00Z",
        &format!("fix: other{trailer}"),
        &[("a", "1")],
    );
    let projects = tmp.join("projects");
    for (id, start, claim) in [
        ("s1", "2026-02-01T10:00:00.000Z", true),
        ("s2", "2026-02-01T10:30:00.000Z", false),
    ] {
        let mut t = Transcript::new(&repo, start);
        t.typed("SECRET start");
        if claim {
            t.run(
                "c1",
                "Bash",
                &bash("cardinal run issue-claim issues/élan.md"),
                None,
            );
        }
        t.ts = "2026-02-01T12:00:00.000Z".to_string();
        t.typed("ok");
        t.write(&projects.join("-cl").join(format!("{id}.jsonl")));
    }
    let (ok, out) = run(dir, &projects, false);
    let row = |s: &str, hash: &str| format!("{s}\tcl\tr1\tcommit-attribution\tbreach\t\t{hash}");
    let rows: Vec<String> = out.lines().skip(1).map(str::to_string).collect();
    let expected = vec![row("s1", &claimed), row("s2", &other)];
    if sorted(rows.clone()) != sorted(expected.clone()) {
        println!("{out}");
    }
    vec![(
        "a commit of a claimed issue is checked against the session that claimed it, the others against the latest-starting session".to_string(),
        ok && sorted(rows) == sorted(expected),
    )]
}

// spec: FEA-11
fn attribution_off_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("attribution-off");
    let repo = new_repo(&tmp.join("off"));
    let config = "[cardinal]\nsource = \"x\"\nrev = \"r1\"\ndisabled = [\"no-ai-attribution\"]\n";
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/cardinal.toml", config)],
    );
    commit_at(
        &repo,
        "2026-02-01T10:10:00Z",
        "docs: z\n\nCo-Authored-By: Claude <noreply@anthropic.com>",
        &[("a", "1")],
    );
    let projects = tmp.join("projects");
    let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
    t.typed("SECRET start");
    t.ts = "2026-02-01T11:00:00.000Z".to_string();
    t.typed("ok");
    t.write(&projects.join("-off").join("s1.jsonl"));
    let (ok, out) = run(dir, &projects, false);
    let (_, summary) = run(dir, &projects, true);
    vec![(
        "a commit of a project that turned off no-ai-attribution is not checked for an attribution"
            .to_string(),
        ok && !out.contains("commit-attribution") && !summary.contains("commit-attribution"),
    )]
}

// spec: FEA-22
fn convention_off_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("convention-off");
    let repo = new_repo(&tmp.join("off"));
    let config =
        "[cardinal]\nsource = \"x\"\nrev = \"r1\"\ndisabled = [\"conventional-commits\"]\n";
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/cardinal.toml", config)],
    );
    commit_at(
        &repo,
        "2026-02-01T10:10:00Z",
        "Added the parser.",
        &[("a", "1")],
    );
    let projects = tmp.join("projects");
    let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
    t.typed("SECRET start");
    t.ts = "2026-02-01T11:00:00.000Z".to_string();
    t.typed("ok");
    t.write(&projects.join("-off").join("s1.jsonl"));
    let (ok, out) = run(dir, &projects, false);
    let (_, summary) = run(dir, &projects, true);
    vec![(
        "a commit of a project that turned off conventional-commits is not checked for its format"
            .to_string(),
        ok && !out.contains("commit-format")
            && !summary.contains("commit-format")
            && summary.contains("commit-attribution"),
    )]
}

#[test]
fn rule_breaches_commits() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    let checks: Vec<_> = commit_checks(&dir, &tmp)
        .into_iter()
        .chain(claim_checks(&dir, &tmp))
        .chain(attribution_off_checks(&dir, &tmp))
        .chain(convention_off_checks(&dir, &tmp))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
