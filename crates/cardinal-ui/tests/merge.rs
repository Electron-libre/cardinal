//! Behavior test for `cardinal-ui --merge <stem>` (STU-20, STU-21), against a throwaway repository whose issue `foo`
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

const QUEUE: &str = r#"{"version":1,"next":"bar","issues":[{"stem":"bar","priority":"high","milestone":null,"state":"ready","taken":null},{"stem":"foo","priority":"high","milestone":null,"state":"ready","taken":{"by":"claim","state":"merge pending","worktree":"WS","branch":"foo","session_alive":false}}]}"#;

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
    std::fs::write(ws.join("done.md"), "closed\n").unwrap();
    git(&ws, &["add", "done.md"]);
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
    /// The fake `cardinal`: the queue of `QUEUE`; `check` logs its directory, then runs `check`.
    fn cardinal(&self, check: &str) -> PathBuf {
        let path = self.root.path().join("cardinal");
        let body = format!(
            "#!/bin/sh\ncase \"$1 $2\" in\n  'run next-issue') printf '%s\\n' '{QUEUE}'; exit 0 ;;\n  'check ') echo \"check in $(pwd)\" >> '{log}'; {check} ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n",
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

/// The pid of a process that ended.
fn ended_pid() -> u32 {
    let mut child = Command::new("true").spawn().unwrap();
    let pid = child.id();
    child.wait().unwrap();
    pid
}

// spec: STU-H5, STU-20, STU-21, STU-3
#[test]
fn a_merge_pending_issue_fast_forwards_the_trunk_then_checks() {
    let p = project();
    let cardinal = p.cardinal("exit 0");
    let ws_head = git(&p.ws, &["rev-parse", "HEAD"]);
    let (code, out, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("merged foo (foo) into trunk"), "{out}");
    assert_eq!(p.trunk(), p.foo(), "the trunk moved to the branch");
    assert!(p.repo.join("done.md").is_file(), "the main worktree too");
    assert_eq!(p.log(), format!("check in {}\n", p.repo.display()));
    assert_eq!(git(&p.ws, &["rev-parse", "HEAD"]), ws_head);
    assert_eq!(git(&p.repo, &["status", "--porcelain"]), "");
}

// spec: STU-21
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

// spec: STU-21
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

// spec: STU-21
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

// spec: STU-21
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
    p.lock(ended_pid());
    let (code, out, err) = p.merge(&cardinal, "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(p.trunk(), p.foo());
}

// spec: STU-21
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

// spec: STU-21
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

// spec: STU-21
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
