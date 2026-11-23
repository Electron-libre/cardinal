//! Behavior test for the end of a waiting session before the rebase of `cardinal-ui --merge <stem>` (SUM-4), against
//! a throwaway repository whose issue `foo` was closed on the branch `foo` of a linked worktree while the trunk moved
//! on, a process standing for its session, a fake `zellij` and a fake `cardinal` whose `rebase-issue` refuses a live
//! lock as RBI-4 does.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

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

fn alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap()
        .success()
}

/// Waits a little for `pid` to end, as the reaper thread may lag behind the exit of `cardinal-ui`.
fn ended(pid: u32) -> bool {
    let start = Instant::now();
    while alive(pid) && start.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(20));
    }
    !alive(pid)
}

fn executable(path: &Path, body: &str) {
    std::fs::write(path, body).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

const BAR: &str =
    r#"{"stem":"bar","priority":"high","milestone":null,"state":"ready","taken":null}"#;
const FOO: &str = r#"{"stem":"foo","priority":"high","milestone":null,"state":"ready","taken":{"by":"claim","state":"merge pending","worktree":"WS","branch":"foo","session_alive":true}}"#;

/// The main worktree `repo` on `trunk` one commit ahead of the base of `foo`, the linked worktree `ws` on `foo`
/// holding its close, the fakes and the log of their calls, and the process of the session when one runs.
struct Project {
    root: tempfile::TempDir,
    repo: PathBuf,
    ws: PathBuf,
    log: PathBuf,
    session: Option<u32>,
}

impl Drop for Project {
    fn drop(&mut self) {
        if let Some(pid) = self.session {
            let _ = Command::new("kill")
                .args(["-9", &pid.to_string()])
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
}

/// The project, its fake `rebase-issue` rebasing `foo` onto `trunk`, or failing on a conflict when `conflict`.
fn project(conflict: bool) -> Project {
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
    std::fs::write(repo.join("README.md"), "trunk\n").unwrap();
    git(&repo, &["add", "README.md"]);
    git(&repo, &["commit", "-qm", "readme", "--no-verify"]);
    let log = root.path().join("log");
    std::fs::write(&log, "").unwrap();
    let p = Project {
        repo: std::fs::canonicalize(repo).unwrap(),
        ws: std::fs::canonicalize(ws).unwrap(),
        log,
        root,
        session: None,
    };
    p.fakes(conflict);
    p
}

impl Project {
    fn path(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    /// The fake `zellij`, listing the panes of the file `panes`, a close sending `SIGHUP` to the session; the fake
    /// `cardinal`, whose `rebase-issue` refuses a dirty worktree or a live lock (RBI-4), then rebases or meets a
    /// conflict, and whose `project-check`, `check` and `worktree close` pass.
    fn fakes(&self, conflict: bool) {
        let log = self.log.display();
        executable(
            &self.path("zellij"),
            &format!(
                "#!/bin/sh\necho \"zellij $*\" >> '{log}'\ncase \"$2\" in\n  list-panes) cat '{panes}'; exit 0 ;;\n  close-tab-by-id|close-pane) echo '[]' > '{panes}'; [ -f '{pid}' ] && kill -HUP $(cat '{pid}'); exit 0 ;;\nesac\nexit 9\n",
                panes = self.path("panes").display(),
                pid = self.path("pid").display(),
            ),
        );
        std::fs::write(self.path("panes"), "[]").unwrap();
        let ws = self.ws.display();
        let lock = self.git_path("cardinal-session");
        let rebase = if conflict {
            "echo 'foo: conflict in README.md: rebase aborted, settle it in ws' >&2; exit 1"
                .to_string()
        } else {
            format!("git -c user.email=t@t -c user.name=t -c commit.gpgsign=false -C '{ws}' rebase -q trunk; exit $?")
        };
        executable(
            &self.path("cardinal"),
            &format!(
                "#!/bin/sh\ncase \"$1 $2\" in\n  'run rebase-issue') echo \"rebase-issue $3\" >> '{log}'\n    [ -n \"$(git -C '{ws}' status --porcelain)\" ] && {{ echo 'worktree {ws} has uncommitted changes or untracked files' >&2; exit 1; }}\n    pid=$(sed -n 's/^pid //p' '{lock}' 2>/dev/null); [ -n \"$pid\" ] && kill -0 \"$pid\" 2>/dev/null && {{ echo 'session s1 works in {ws}' >&2; exit 1; }}\n    {rebase} ;;\n  'run project-check') exit 0 ;;\n  'run worktree') echo \"worktree close $4\" >> '{log}'; exit 0 ;;\n  'run next-issue') if git cat-file -e trunk:issues/done/foo.md 2>/dev/null; then printf '%s\\n' '{{\"version\":1,\"next\":\"bar\",\"issues\":[{BAR}]}}'; else printf '%s\\n' '{{\"version\":1,\"next\":\"bar\",\"issues\":[{BAR},{FOO}]}}'; fi; exit 0 ;;\n  'run roadmap') echo '{{\"version\":1,\"milestones\":[]}}'; exit 0 ;;\n  'check ') echo check >> '{log}'; [ -f '{failing}' ] && {{ echo 'check failed' >&2; exit 1; }}; exit 0 ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n",
                lock = lock.display(),
                failing = self.path("check-fails").display(),
            ),
        );
    }

    fn git_path(&self, name: &str) -> PathBuf {
        PathBuf::from(git(
            &self.ws,
            &["rev-parse", "--path-format=absolute", "--git-path", name],
        ))
    }

    /// Starts the process of the session `s1`, ignoring `SIGHUP` when `deaf`, and writes the lock of `ws` naming it,
    /// with a wait mark naming it when `waiting`.
    fn session(&mut self, waiting: bool, deaf: bool) -> u32 {
        let script = if deaf {
            // an ignored signal stays ignored across `exec`: no orphan `sleep` outlives the test
            "trap '' HUP; exec sleep 60"
        } else {
            "exec sleep 60"
        };
        let mut child: Child = Command::new("sh").args(["-c", script]).spawn().unwrap();
        let pid = child.id();
        // reaped at once, so `kill -0` no longer sees it once it ended
        std::thread::spawn(move || child.wait());
        self.session = Some(pid);
        std::fs::write(self.path("pid"), pid.to_string()).unwrap();
        std::fs::write(
            self.git_path("cardinal-session"),
            format!("session s1\npid {pid}\n"),
        )
        .unwrap();
        if waiting {
            std::fs::write(
                self.git_path("cardinal-waiting"),
                "session s1\nsince 1\nreason stop\n",
            )
            .unwrap();
        }
        pid
    }

    /// The tab `foo` (id 4) holding the session's pane alone.
    fn tab(&self) {
        std::fs::write(
            self.path("panes"),
            r#"[{"id":7,"is_plugin":false,"tab_id":4,"tab_name":"foo","terminal_command":"claude /issue-workflow issues/foo.md"}]"#,
        )
        .unwrap();
    }

    /// `cardinal-ui args` run in the main worktree, in a zellij session when `zellij`: exit code, stdout, stderr.
    fn ui(&self, args: &[&str], zellij: bool) -> (Option<i32>, String, String) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"));
        cmd.args(args)
            .current_dir(&self.repo)
            .env("CARDINAL_UI_CARDINAL", self.path("cardinal"))
            .env("CARDINAL_UI_ZELLIJ", self.path("zellij"));
        if zellij {
            cmd.env("ZELLIJ", "0");
        } else {
            cmd.env_remove("ZELLIJ");
        }
        let out = cmd.output().unwrap();
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap()
    }

    fn trunk(&self) -> String {
        git(&self.repo, &["rev-parse", "trunk"])
    }

    fn foo(&self) -> String {
        git(&self.repo, &["rev-parse", "foo"])
    }
}

// spec: SUM-H3, SUM-4
#[test]
fn a_waiting_session_ends_with_its_tab_before_the_rebase_then_the_end_closes_the_worktree() {
    let mut p = project(false);
    let pid = p.session(true, false);
    p.tab();
    let (code, out, err) = p.ui(&["--merge", "foo"], true);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(
        out.contains("rebased and merged foo (foo) into trunk"),
        "{out}"
    );
    assert!(ended(pid), "the session ended");
    assert_eq!(p.trunk(), p.foo());
    let log = p.log();
    assert!(
        log.starts_with(
            "zellij action list-panes --json --all\nzellij action close-tab-by-id 4\nrebase-issue foo\n"
        ),
        "{log}"
    );
    // the end the key `f` chains (SUE-1): the tab gone, the session ended, only the worktree is left to close
    let (code, out, err) = p.ui(&["--end", "foo"], true);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("ended foo"), "{out}");
    assert!(
        p.log()
            .ends_with(&format!("worktree close {}\n", p.ws.display())),
        "{}",
        p.log()
    );
}

// spec: SUM-4
#[test]
fn a_session_at_work_keeps_running_and_the_rebase_is_refused() {
    let mut p = project(false);
    let pid = p.session(false, false);
    p.tab();
    let before = p.trunk();
    let (code, _, err) = p.ui(&["--merge", "foo"], true);
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!("session s1 works in {}", p.ws.display())),
        "{err}"
    );
    assert!(alive(pid), "the session still runs");
    assert!(!p.log().contains("zellij"), "{}", p.log());
    assert_eq!(p.trunk(), before);
}

// spec: SUM-4
#[test]
fn a_waiting_session_in_a_dirty_worktree_keeps_running() {
    let mut p = project(false);
    let pid = p.session(true, false);
    p.tab();
    std::fs::write(p.ws.join("notes.txt"), "draft\n").unwrap();
    let (code, _, err) = p.ui(&["--merge", "foo"], true);
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!(
            "worktree {} has uncommitted changes or untracked files",
            p.ws.display()
        )),
        "{err}"
    );
    assert!(alive(pid), "the session still runs");
    assert!(!p.log().contains("zellij"), "{}", p.log());
}

// spec: SUM-4
#[test]
fn a_conflict_after_the_end_says_the_session_ended() {
    let mut p = project(true);
    let pid = p.session(true, false);
    let before = p.trunk();
    let (code, _, err) = p.ui(&["--merge", "foo"], false);
    assert_eq!(code, Some(1));
    assert!(
        err.contains(
            "ended session s1; foo: conflict in README.md: rebase aborted, settle it in ws"
        ),
        "{err}"
    );
    assert!(ended(pid), "the session got SIGHUP");
    assert_eq!(p.trunk(), before);
}

// spec: SUM-4
#[test]
fn a_session_deaf_to_sighup_runs_no_rebase() {
    let mut p = project(false);
    p.session(true, true);
    let before = p.trunk();
    let (code, _, err) = p.ui(&["--merge", "foo"], false);
    assert_eq!(code, Some(1));
    assert!(err.contains("session s1 still runs after 10 s"), "{err}");
    assert!(!p.log().contains("rebase-issue"), "{}", p.log());
    assert_eq!(p.trunk(), before);
}

// spec: SUM-4
#[test]
fn a_check_that_fails_after_the_end_says_the_session_ended() {
    let mut p = project(false);
    let pid = p.session(true, false);
    std::fs::write(p.path("check-fails"), "").unwrap();
    let (code, _, err) = p.ui(&["--merge", "foo"], false);
    assert_eq!(code, Some(1));
    assert!(
        err.contains(
            "ended session s1; rebased and merged foo (foo) into trunk; cardinal check: exit 1: check failed"
        ),
        "{err}"
    );
    assert!(ended(pid), "the session got SIGHUP");
    assert_eq!(p.trunk(), p.foo());
}
