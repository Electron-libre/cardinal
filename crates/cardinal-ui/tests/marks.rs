//! Behavior test for the marks of a session at work and of a waiting one in the queue of `cardinal-ui --print`
//! (SUQ-4), against a throwaway repository with two linked worktrees and a fake `cardinal` (STU-5).

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "user.email=t@t", "-c", "user.name=t"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8_lossy(&out.stdout).trim_end().to_string()
}

/// A repository `repo` on `trunk` and the linked worktrees `ws` and `wt` on the branches of their name.
struct Project {
    root: tempfile::TempDir,
    repo: PathBuf,
    ws: PathBuf,
    wt: PathBuf,
}

fn project() -> Project {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", "trunk", "."]);
    git(
        &repo,
        &["commit", "-qm", "init", "--allow-empty", "--no-verify"],
    );
    let [ws, wt] = ["ws", "wt"].map(|name| {
        let dir = root.path().join(name);
        git(
            &repo,
            &["worktree", "add", "-q", "-b", name, dir.to_str().unwrap()],
        );
        std::fs::canonicalize(dir).unwrap()
    });
    let repo = std::fs::canonicalize(repo).unwrap();
    Project { root, repo, ws, wt }
}

/// The file `name` of the git directory of the worktree `dir`, written with `text`.
fn git_file(dir: &Path, name: &str, text: &str) {
    let path = git(
        dir,
        &["rev-parse", "--path-format=absolute", "--git-path", name],
    );
    std::fs::write(path, text).unwrap();
}

/// The session `id` of this test's process holds the lock of `dir` (WSL-9).
fn lock(dir: &Path, id: &str) {
    git_file(
        dir,
        "cardinal-session",
        &format!("session {id}\npid {}\n", std::process::id()),
    );
}

/// The worktree `dir` waits for the user in the session `id` (SWT-3).
fn waiting(dir: &Path, id: &str) {
    git_file(
        dir,
        "cardinal-waiting",
        &format!("session {id}\nsince 1735808700\nreason stop\n"),
    );
}

/// An open issue of milestone `One` taken as `taken` (a JSON value).
fn issue(stem: &str, taken: &str) -> String {
    format!(
        r#"{{"stem":"{stem}","title":"t","priority":"low","why":"","milestone":"One","depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":{taken},"stale":false}}"#
    )
}

/// A claim of the worktree `dir` on the branch `branch`, in the state `state`, its session alive or not.
fn claim(dir: &Path, branch: &str, state: &str, alive: bool) -> String {
    format!(
        r#"{{"by":"claim","state":"{state}","worktree":"{}","branch":"{branch}","session_alive":{alive}}}"#,
        dir.display()
    )
}

/// `cardinal-ui --print` in `p` with a fake `cardinal` whose queue holds `issues`: its stdout.
fn print(p: &Project, issues: &[String]) -> String {
    let queue = format!(
        r#"{{"version":1,"next":null,"issues":[{}]}}"#,
        issues.join(",")
    );
    let roadmap = r#"{"version":1,"milestones":[]}"#;
    let cardinal = p.root.path().join("cardinal");
    let script = format!(
        "#!/bin/sh\ncase \"$*\" in\n  'run next-issue --json') printf '%s\\n' '{queue}' ;;\n  'run roadmap --open --json') printf '%s\\n' '{roadmap}' ;;\n  'run session-metrics') ;;\n  'run delivered-quality '*) for rows; do :; done; : > \"$rows\" ;;\n  'run cardinal-report '*) echo '{{\"version\":1,\"tables\":[]}}' ;;\n  *) echo \"bad call: $*\" >&2; exit 9 ;;\nesac\n"
    );
    std::fs::write(&cardinal, script).unwrap();
    std::fs::set_permissions(&cardinal, std::fs::Permissions::from_mode(0o755)).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
        .arg("--print")
        .current_dir(&p.repo)
        .env("CARDINAL_UI_CARDINAL", &cardinal)
        .env("TZ", "UTC")
        .env_remove("ZELLIJ")
        .env("CARDINAL_UI_ZELLIJ", "/nonexistent/zellij")
        .env("XDG_CACHE_HOME", p.root.path().join("cache"))
        .env("XDG_CONFIG_HOME", "/nonexistent/config")
        .env_remove("CARDINAL_UI_THEME")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout
}

/// The issue row of `stem` in `out`: the line whose stem cell, after the next mark, the tree and a mark, is `stem`.
fn row<'a>(out: &'a str, stem: &str) -> &'a str {
    out.lines()
        .find(|l| {
            let cell = l.trim_start_matches(['│', ' ']);
            cell.split("  ").next().is_some_and(|c| c.ends_with(stem))
        })
        .unwrap_or_else(|| panic!("no row of {stem} in\n{out}"))
}

// spec: SUQ-H2, SUQ-4, SUQ-3
#[test]
fn an_issue_at_work_and_a_waiting_one_carry_their_mark() {
    let p = project();
    lock(&p.ws, "s1");
    lock(&p.wt, "s2");
    waiting(&p.wt, "s2");
    let out = print(
        &p,
        &[
            issue("a", &claim(&p.ws, "ws", "working", true)),
            issue("b", &claim(&p.wt, "wt", "working", true)),
            issue("c", "null"),
        ],
    );
    let (a, b, c) = (row(&out, "a"), row(&out, "b"), row(&out, "c"));
    assert!(a.contains("  ⠿ a  "), "{a}");
    assert!(b.contains("  ‖ b  "), "{b}");
    assert!(c.contains("  c  "), "{c}");
    let column = |r: &str, word: &str| r.find(word).map(|at| r[..at].chars().count());
    assert_eq!(column(a, "low"), column(c, "low"), "SUQ-3:\n{out}");
    assert_eq!(column(b, "low"), column(c, "low"), "SUQ-3:\n{out}");
}

// spec: SUQ-4
#[test]
fn an_ended_session_a_pending_merge_and_a_branch_carry_no_mark() {
    let p = project();
    lock(&p.ws, "s1");
    let branch = format!(
        r#"{{"by":"branch","branch":"wt","worktree":"{}"}}"#,
        p.wt.display()
    );
    let out = print(
        &p,
        &[
            issue("a", &claim(&p.ws, "ws", "working", false)),
            issue("b", &claim(&p.ws, "ws", "merge pending", true)),
            issue("c", &branch),
        ],
    );
    for stem in ["a", "b", "c"] {
        let r = row(&out, stem);
        assert!(!r.contains('⠿') && !r.contains('‖'), "{r}");
    }
}

// spec: SUQ-4
#[test]
fn a_wait_of_another_session_or_an_unlisted_worktree_reads_at_work() {
    let p = project();
    lock(&p.ws, "s1");
    waiting(&p.ws, "s9");
    let gone = p.root.path().join("gone");
    let out = print(
        &p,
        &[
            issue("a", &claim(&p.ws, "ws", "working", true)),
            issue("b", &claim(&gone, "gone", "working", true)),
        ],
    );
    assert!(row(&out, "a").contains("⠿ a"), "{out}");
    assert!(row(&out, "b").contains("⠿ b"), "{out}");
}
