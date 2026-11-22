//! Behavior test for `cardinal-ui --merge <stem>` (SUA-2, SUA-7), against a throwaway repository whose issue `foo`
//! was closed on the branch `foo` of a linked worktree, and a fake `cardinal` that logs its calls.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args([
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8_lossy(&out.stdout).trim_end().to_string()
}

const BAR: &str =
    r#"{"stem":"bar","priority":"high","milestone":null,"state":"ready","taken":null}"#;
const FOO: &str = r#"{"stem":"foo","priority":"high","milestone":null,"state":"ready","taken":{"by":"claim","state":"merge pending","worktree":"WS","branch":"foo","session_alive":false}}"#;

/// The main worktree `repo` on `trunk`, the linked worktree `ws` on `foo` one commit ahead, the fake `cardinal` and
/// the log of its calls.
struct Project {
    root: tempfile::TempDir,
    repo: PathBuf,
    ws: PathBuf,
    log: PathBuf,
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
    let ws = root.path().join("ws");
    git(
        &repo,
        &["worktree", "add", "-q", "-b", "foo", ws.to_str().unwrap()],
    );
    std::fs::create_dir_all(ws.join("issues/done")).unwrap();
    std::fs::write(ws.join("issues/done/foo.md"), "closed\n").unwrap();
    git(&ws, &["add", "issues/done/foo.md"]);
    git(&ws, &["commit", "-qm", "close foo", "--no-verify"]);
    let log = root.path().join("log");
    std::fs::write(&log, "").unwrap();
    let repo = std::fs::canonicalize(repo).unwrap();
    let ws = std::fs::canonicalize(ws).unwrap();
    Project {
        root,
        repo,
        ws,
        log,
    }
}

impl Project {
    /// The fake `cardinal`: a queue of `bar` and, while the trunk does not hold its close, `foo` `merge pending`, as
    /// CLM-6 prunes it; `check` logs its directory, then runs `check`; `roadmap` lists no milestone.
    fn cardinal(&self, check: &str) -> PathBuf {
        let path = self.root.path().join("cardinal");
        let body = format!(
            "#!/bin/sh\ncase \"$1 $2\" in\n  'run next-issue') if git cat-file -e trunk:issues/done/foo.md 2>/dev/null; then printf '%s\\n' '{{\"version\":1,\"next\":\"bar\",\"issues\":[{BAR}]}}'; else printf '%s\\n' '{{\"version\":1,\"next\":\"bar\",\"issues\":[{BAR},{FOO}]}}'; fi; exit 0 ;;\n  'run roadmap') echo '{{\"version\":1,\"milestones\":[]}}'; exit 0 ;;\n  'run session-metrics'|'run delivered-quality') exit 3 ;;\n  'check ') echo \"check in $(pwd)\" >> '{log}'; {check} ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n",
            log = self.log.display()
        );
        std::fs::write(&path, body).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// `cardinal-ui --merge <stem>` run in the linked worktree: exit code, stdout, stderr.
    fn merge(&self, cardinal: &Path, stem: &str) -> (Option<i32>, String, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
            .args(["--merge", stem])
            .current_dir(&self.ws)
            .env("CARDINAL_UI_CARDINAL", cardinal)
            .env_remove("ZELLIJ")
            .output()
            .unwrap();
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    /// The screen of `cardinal-ui --print` run in the main worktree.
    fn print(&self, cardinal: &Path) -> String {
        let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
            .arg("--print")
            .current_dir(&self.repo)
            .env("CARDINAL_UI_CARDINAL", cardinal)
            .env("XDG_CACHE_HOME", self.root.path().join("cache"))
            // the theme of the user is never read (STU-32)
            .env("XDG_CONFIG_HOME", "/nonexistent/config")
            .env_remove("CARDINAL_UI_THEME")
            .env_remove("ZELLIJ")
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    fn trunk(&self) -> String {
        git(&self.repo, &["rev-parse", "trunk"])
    }

    fn foo(&self) -> String {
        git(&self.repo, &["rev-parse", "foo"])
    }

    fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap()
    }

    /// Writes the lock of the main worktree, naming the session `s1` and the process `pid`.
    fn lock(&self, pid: u32) {
        let path = git(
            &self.repo,
            &[
                "rev-parse",
                "--path-format=absolute",
                "--git-path",
                "cardinal-session",
            ],
        );
        std::fs::write(path, format!("session s1\npid {pid}\n")).unwrap();
    }
}

/// A pid no process can have: above the largest `pid_max` of Linux (2^22).
const ENDED_PID: u32 = 99_999_999;

// spec: SUA-H2, SUA-2, SUA-7, STU-3
#[test]
fn a_merge_pending_issue_fast_forwards_the_trunk_then_checks() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    let ws_head = git(&p.ws, &["rev-parse", "HEAD"]);
    let refs = || git(&p.repo, &["for-each-ref", "--format=%(refname)", "refs/"]);
    let before = refs();
    let (code, out, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("merged foo (foo) into trunk"), "{out}");
    assert_eq!(p.trunk(), p.foo(), "the trunk moved to the branch");
    assert!(
        p.repo.join("issues/done/foo.md").is_file(),
        "the main worktree too"
    );
    assert_eq!(refs(), before, "no ref of its own");
    assert_eq!(p.log(), format!("check in {}\n", p.repo.display()));
    assert_eq!(git(&p.ws, &["rev-parse", "HEAD"]), ws_head);
    assert_eq!(git(&p.repo, &["status", "--porcelain"]), "");
    let shown = p.print(&cardinal);
    assert!(
        shown.contains("> bar") && !shown.contains("foo  high"),
        "{shown}"
    );
}

// spec: SUA-7
#[test]
fn a_branch_without_its_close_committed_is_refused() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    git(&p.ws, &["reset", "-q", "--soft", "HEAD~1"]);
    let before = p.trunk();
    let (code, _, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(1));
    assert!(
        err.contains("foo does not hold the close of foo: commit it first"),
        "{err}"
    );
    assert_eq!(p.trunk(), before);
    assert_eq!(p.log(), "");
}

// spec: SUA-2
#[test]
fn a_tag_named_as_the_branch_is_not_merged() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    git(&p.ws, &["tag", "foo", "HEAD"]);
    git(
        &p.ws,
        &["commit", "-qm", "more", "--allow-empty", "--no-verify"],
    );
    let (code, out, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(p.trunk(), git(&p.repo, &["rev-parse", "refs/heads/foo"]));
}

// spec: SUA-7
#[test]
fn an_issue_not_merge_pending_merges_nothing() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    let before = p.trunk();
    for stem in ["bar", "nope"] {
        let (code, _, err) = p.merge(&cardinal, stem);
        assert_eq!(code, Some(1), "{stem}");
        assert!(err.contains(&format!("{stem}: not merge pending")), "{err}");
    }
    assert_eq!(p.trunk(), before);
    assert_eq!(p.log(), "");
}

// spec: SUA-7
#[test]
fn a_dirty_main_worktree_is_refused() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    let before = p.trunk();
    std::fs::write(p.repo.join("stray"), "x").unwrap();
    let (code, _, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(1));
    assert!(
        err.contains("main worktree has uncommitted changes or untracked files"),
        "{err}"
    );
    std::fs::remove_file(p.repo.join("stray")).unwrap();
    std::fs::write(p.repo.join("tracked"), "x").unwrap();
    git(&p.repo, &["add", "tracked"]);
    let (code, _, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(1));
    assert!(err.contains("uncommitted changes"), "{err}");
    assert_eq!(p.trunk(), before);
    assert_eq!(p.log(), "");
}

// spec: SUA-7
#[test]
fn a_detached_main_worktree_is_refused() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    git(&p.repo, &["checkout", "-q", "--detach"]);
    let (code, _, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(1));
    assert!(err.contains("main worktree not on a branch"), "{err}");
    assert_eq!(p.log(), "");
}

// spec: SUA-7
#[test]
fn a_live_session_in_the_main_worktree_is_refused_an_ended_one_is_not() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    let before = p.trunk();
    p.lock(std::process::id());
    let (code, _, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(1));
    assert!(
        err.contains("session s1 works in the main worktree"),
        "{err}"
    );
    assert_eq!(p.trunk(), before);
    p.lock(ENDED_PID);
    let (code, out, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(p.trunk(), p.foo());
}

// spec: SUA-7
#[test]
fn a_diverged_branch_is_refused() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    git(
        &p.repo,
        &["commit", "-qm", "other", "--allow-empty", "--no-verify"],
    );
    let before = p.trunk();
    let (code, _, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(1));
    assert!(
        err.contains("foo diverged from trunk: rebase it in its worktree"),
        "{err}"
    );
    assert_eq!(p.trunk(), before);
    assert_eq!(p.log(), "");
}

// spec: SUA-7
#[test]
fn a_failing_check_keeps_the_merge_and_names_the_install() {
    let p = project();
    let line = "the installed cardinal cannot run the hook `x`: install it with `cargo install y`";
    let cardinal = p.cardinal(&format!("echo '{line}' >&2; echo more >&2; exit 1"));
    let (code, _, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!(
            "merged foo (foo) into trunk; cardinal check: exit 1: {line}"
        )),
        "{err}"
    );
    assert_eq!(p.trunk(), p.foo(), "the merge is kept");
}

// spec: SUA-7
#[test]
fn merge_without_a_stem_is_refused() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
        .arg("--merge")
        .current_dir(&p.ws)
        .env("CARDINAL_UI_CARDINAL", cardinal)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

// spec: SUA-7
#[test]
fn a_bare_repository_has_no_main_worktree_to_merge_in() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    let bare = p.root.path().join("bare.git");
    let w = p.root.path().join("w");
    git(
        p.root.path(),
        &[
            "clone",
            "-q",
            "--bare",
            p.repo.to_str().unwrap(),
            bare.to_str().unwrap(),
        ],
    );
    git(
        &bare,
        &["worktree", "add", "-q", w.to_str().unwrap(), "trunk"],
    );
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
        .args(["--merge", "foo"])
        .current_dir(&w)
        .env("CARDINAL_UI_CARDINAL", cardinal)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    assert!(
        err.contains("no main worktree: the repository is bare"),
        "{err}"
    );
    assert_eq!(p.log(), "");
}

// spec: SUE-H1, STU-23
#[test]
fn a_linked_worktree_whose_close_is_on_the_trunk_reads_merged() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    let ws = p.ws.display().to_string();
    let row = |shown: String| {
        shown
            .lines()
            .find(|l| l.contains(&ws))
            .unwrap_or_default()
            .to_string()
    };
    let before = row(p.print(&cardinal));
    assert!(
        before.contains("foo") && !before.contains("merged"),
        "{before}"
    );
    git(&p.repo, &["merge", "-q", "--ff-only", "foo"]);
    let after = row(p.print(&cardinal));
    assert!(after.contains("foo  merged"), "{after}");
    let main = p.print(&cardinal);
    assert!(!main.lines().any(|l| l.contains("trunk  merged")), "{main}");
}
