//! Behavior test for `cardinal-ui --start <stem>` (STU-9, STU-11, STU-12), against a throwaway git repository, a fake
//! `cardinal` and a fake `zellij` that log their calls.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A repository `repo`, the fakes beside it and the log of their calls.
struct Setup {
    root: tempfile::TempDir,
    repo: PathBuf,
    log: PathBuf,
}

const QUEUE: &str = r#"{"version":1,"next":"b","issues":[{"stem":"b","priority":"high","milestone":null,"state":"ready","taken":null},{"stem":"a","priority":"low","milestone":null,"state":"ready","taken":{"by":"claim","state":"working","worktree":"/x","branch":"x","session_alive":true}},{"stem":"c","priority":"low","milestone":null,"state":"blocked","taken":null}]}"#;

fn setup() -> Setup {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let out = Command::new("git")
        .args(["init", "-q", "-b", "trunk", "."])
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(out.status.success());
    let log = root.path().join("log");
    std::fs::write(&log, "").unwrap();
    Setup { root, repo, log }
}

fn script(path: &Path, body: &str) {
    std::fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

impl Setup {
    /// The fake `cardinal`: the queue of `QUEUE`; `worktree open <stem>` logs its call, then runs `open`.
    fn cardinal(&self, open: &str) -> PathBuf {
        self.cardinal_with(open, QUEUE)
    }

    /// The fake `cardinal` of `cardinal`, its queue `queue`.
    fn cardinal_with(&self, open: &str, queue: &str) -> PathBuf {
        let path = self.root.path().join("cardinal");
        let body = format!(
            "case \"$1 $2\" in\n  'run next-issue') printf '%s\\n' '{queue}'; exit 0 ;;\n  'run worktree') echo \"cardinal $*\" >> '{log}'; {open} ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n",
            log = self.log.display()
        );
        script(&path, &body);
        path
    }

    /// A `worktree open` that prints the path of the new worktree.
    fn opened(&self) -> String {
        let wt = self.root.path().join("repo-b");
        format!(
            "mkdir -p '{w}'; echo building >&2; echo '{w}'; exit 0",
            w = wt.display()
        )
    }

    /// The fake `zellij`: logs each argument on a line, then exits `code` with `err` on stderr.
    fn zellij(&self, code: i32, err: &str) -> PathBuf {
        let path = self.root.path().join("zellij");
        let body = format!(
            "echo zellij >> '{log}'; for a in \"$@\"; do printf '%s\\n' \"$a\" >> '{log}'; done\nprintf '%s' '{err}' >&2; exit {code}\n",
            log = self.log.display()
        );
        script(&path, &body);
        path
    }

    fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap()
    }

    /// `cardinal-ui <args>` in the repository, in a zellij session when `zellij` is given.
    fn ui(
        &self,
        cardinal: &Path,
        zellij: Option<&Path>,
        args: &[&str],
    ) -> (Option<i32>, String, String) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"));
        cmd.args(args)
            .current_dir(&self.repo)
            .env("CARDINAL_UI_CARDINAL", cardinal)
            .env_remove("ZELLIJ")
            .env("CARDINAL_UI_ZELLIJ", "/nonexistent/zellij");
        if let Some(z) = zellij {
            cmd.env("ZELLIJ", "0").env("CARDINAL_UI_ZELLIJ", z);
        }
        let out = cmd.output().unwrap();
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }
}

// spec: STU-H2, STU-9, STU-11, STU-12
#[test]
fn a_ready_issue_starts_in_a_tab_of_its_worktree() {
    let s = setup();
    let cardinal = s.cardinal(&s.opened());
    let zellij = s.zellij(0, "");
    let (code, out, err) = s.ui(&cardinal, Some(&zellij), &["--start", "b"]);
    assert_eq!(code, Some(0), "{out}{err}");
    let wt = s.root.path().join("repo-b");
    let expected = format!(
        "cardinal run worktree open b\nzellij\naction\nnew-tab\n--cwd\n{}\n--name\nb\n--\nclaude\n/issue-workflow issues/b.md\n",
        wt.display()
    );
    assert_eq!(s.log(), expected);
    assert!(
        out.contains('b') && out.contains(&wt.display().to_string()),
        "{out}"
    );
}

// spec: STU-11
#[test]
fn outside_zellij_nothing_starts() {
    let s = setup();
    let cardinal = s.cardinal(&s.opened());
    let (code, _, err) = s.ui(&cardinal, None, &["--start", "b"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("start needs zellij: run cardinal-ui in a zellij session"),
        "{err}"
    );
    assert_eq!(s.log(), "");
}

// spec: STU-9, STU-11
#[test]
fn a_taken_blocked_or_unknown_issue_does_not_start() {
    let s = setup();
    let cardinal = s.cardinal(&s.opened());
    let zellij = s.zellij(0, "");
    for (stem, why) in [("a", "taken"), ("c", "blocked"), ("z", "no open issue")] {
        let (code, _, err) = s.ui(&cardinal, Some(&zellij), &["--start", stem]);
        assert_eq!(code, Some(1), "{stem}");
        assert!(err.contains(stem) && err.contains(why), "{stem}: {err}");
    }
    assert_eq!(s.log(), "");
}

// spec: STU-11
#[test]
fn a_failed_worktree_open_runs_no_zellij() {
    let s = setup();
    let cardinal = s.cardinal("echo 'branch b is already checked out' >&2; exit 1");
    let zellij = s.zellij(0, "");
    let (code, _, err) = s.ui(&cardinal, Some(&zellij), &["--start", "b"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("worktree open b: exit 1: branch b is already checked out"),
        "{err}"
    );
    assert_eq!(s.log(), "cardinal run worktree open b\n");
}

// spec: STU-11
#[test]
fn a_failed_zellij_is_named_and_the_worktree_stays() {
    let s = setup();
    let cardinal = s.cardinal(&s.opened());
    let zellij = s.zellij(2, "no session\nmore");
    let (code, _, err) = s.ui(&cardinal, Some(&zellij), &["--start", "b"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("zellij action new-tab: exit 2: no session"),
        "{err}"
    );
    assert!(s.root.path().join("repo-b").is_dir());
}

// spec: STU-11
#[test]
fn start_without_a_stem_is_refused() {
    let s = setup();
    let cardinal = s.cardinal(&s.opened());
    let (code, _, err) = s.ui(&cardinal, None, &["--start"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("usage"), "{err}");
}

/// A queue of ready issues claimed by sessions that ended: `d` in the linked worktree `<root>/repo-d`, `e` in a
/// worktree removed since, `f` merge pending, and `g` in a directory that is no worktree.
fn ended_queue(s: &Setup) -> (String, PathBuf) {
    let wt = s.root.path().join("repo-d");
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(["-c", "user.email=t@t", "-c", "user.name=t"])
            .args(args)
            .current_dir(&s.repo)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}");
    };
    git(&["commit", "-q", "--allow-empty", "-m", "init", "--no-verify"]);
    git(&["worktree", "add", "-q", "-b", "d", wt.to_str().unwrap()]);
    let gone = s.root.path().join("repo-e");
    let plain = s.root.path().join("repo-g");
    std::fs::create_dir_all(&plain).unwrap();
    let claim = |state: &str, w: &Path| {
        format!(
            r#"{{"by":"claim","state":"{state}","worktree":"{}","branch":"x","session_alive":false}}"#,
            w.display()
        )
    };
    let issue = |stem: &str, taken: String| {
        format!(
            r#"{{"stem":"{stem}","priority":"high","milestone":null,"state":"ready","taken":{taken}}}"#
        )
    };
    let queue = format!(
        r#"{{"version":1,"next":null,"issues":[{},{},{},{}]}}"#,
        issue("d", claim("working", &wt)),
        issue("e", claim("working", &gone)),
        issue("f", claim("merge pending", &wt)),
        issue("g", claim("working", &plain)),
    );
    (queue, wt)
}

// spec: STU-9, STU-11
#[test]
fn an_issue_whose_session_ended_resumes_in_the_worktree_of_its_claim() {
    let s = setup();
    let (queue, wt) = ended_queue(&s);
    let cardinal = s.cardinal_with(&s.opened(), &queue);
    let zellij = s.zellij(0, "");
    let (code, out, err) = s.ui(&cardinal, Some(&zellij), &["--start", "d"]);
    assert_eq!(code, Some(0), "{out}{err}");
    let expected = format!(
        "zellij\naction\nnew-tab\n--cwd\n{}\n--name\nd\n--\nclaude\n/issue-workflow issues/d.md\n",
        wt.display()
    );
    assert_eq!(s.log(), expected, "no worktree open");
    assert!(
        out.contains(&format!("resumed d in {}", wt.display())),
        "{out}"
    );
}

// spec: STU-11
#[test]
fn a_claim_whose_worktree_is_gone_or_merge_pending_does_not_resume() {
    let s = setup();
    let (queue, _) = ended_queue(&s);
    let cardinal = s.cardinal_with(&s.opened(), &queue);
    let zellij = s.zellij(0, "");
    let gone = s.root.path().join("repo-e");
    let (code, _, err) = s.ui(&cardinal, Some(&zellij), &["--start", "e"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!(
            "e: worktree {} gone: cardinal run issue-claim --release e",
            gone.display()
        )),
        "{err}"
    );
    let plain = s.root.path().join("repo-g");
    let (code, _, err) = s.ui(&cardinal, Some(&zellij), &["--start", "g"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!("g: worktree {} gone", plain.display())),
        "{err}"
    );
    let (code, _, err) = s.ui(&cardinal, Some(&zellij), &["--start", "f"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("f: taken"), "{err}");
    assert_eq!(s.log(), "");
}

// spec: STU-11
#[test]
fn a_claim_whose_worktree_a_live_session_holds_does_not_resume() {
    let s = setup();
    let (queue, wt) = ended_queue(&s);
    let out = Command::new("git")
        .args(["rev-parse", "--git-path", "cardinal-session"])
        .current_dir(&wt)
        .output()
        .unwrap();
    let lock = wt.join(String::from_utf8_lossy(&out.stdout).trim());
    std::fs::write(&lock, format!("session s9\npid {}\n", std::process::id())).unwrap();
    let cardinal = s.cardinal_with(&s.opened(), &queue);
    let zellij = s.zellij(0, "");
    let (code, _, err) = s.ui(&cardinal, Some(&zellij), &["--start", "d"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!("d: session s9 works in {}", wt.display())),
        "{err}"
    );
    assert_eq!(s.log(), "");
}

// spec: STU-11
#[test]
fn a_claim_that_says_nothing_of_its_session_does_not_resume() {
    let s = setup();
    let (queue, _) = ended_queue(&s);
    let queue = queue.replacen(r#","session_alive":false"#, "", 1);
    let cardinal = s.cardinal_with(&s.opened(), &queue);
    let zellij = s.zellij(0, "");
    let (code, _, err) = s.ui(&cardinal, Some(&zellij), &["--start", "d"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("d: taken"), "{err}");
}
