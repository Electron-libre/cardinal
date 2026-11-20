//! Behavior test for `cardinal-ui --end <stem>` (SUA-8, SUA-9), against a throwaway repository whose issue `foo`
//! was closed on the branch `foo` of a linked worktree then merged into `trunk`, a process standing for its session,
//! a fake `zellij` and a fake `cardinal` that log their calls.

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

fn executable(path: &Path, body: &str) {
    std::fs::write(path, body).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A pid no process can have: above the largest `pid_max` of Linux (2^22).
const ENDED_PID: u32 = 99_999_999;

/// The main worktree `repo` on `trunk`, the linked worktree `ws` on `foo` holding the close of `foo`, the fakes and
/// the log of their calls, and the process of the session when one runs.
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

/// The project, `foo` merged into `trunk` when `merged`.
fn project(merged: bool) -> Project {
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
    if merged {
        git(&repo, &["merge", "-q", "--ff-only", "foo"]);
    }
    let log = root.path().join("log");
    std::fs::write(&log, "").unwrap();
    let p = Project {
        repo: std::fs::canonicalize(repo).unwrap(),
        ws: std::fs::canonicalize(ws).unwrap(),
        log,
        root,
        session: None,
    };
    p.fakes(0, "");
    p
}

impl Project {
    fn path(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    /// The fake `zellij`, listing the panes of the file `panes`, a close sending `SIGHUP` to the session; and the
    /// fake `cardinal`, whose `worktree close` exits `code` printing `err`.
    fn fakes(&self, code: i32, err: &str) {
        let log = self.log.display();
        executable(
            &self.path("zellij"),
            &format!(
                "#!/bin/sh\necho \"zellij $*\" >> '{log}'\ncase \"$2\" in\n  list-panes) grep -q FAIL '{panes}' && {{ echo 'no such action' >&2; exit 2; }}; cat '{panes}'; exit 0 ;;\n  close-tab-by-id|close-pane) [ -f '{pid}' ] && kill -HUP $(cat '{pid}'); exit 0 ;;\nesac\nexit 9\n",
                panes = self.path("panes").display(),
                pid = self.path("pid").display(),
            ),
        );
        std::fs::write(self.path("panes"), "[]").unwrap();
        executable(
            &self.path("cardinal"),
            &format!(
                "#!/bin/sh\ncase \"$1 $2 $3\" in\n  'run worktree close') echo \"worktree close $4 in $(pwd)\" >> '{log}'; echo '{err}' >&2; exit {code} ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n"
            ),
        );
    }

    /// Starts the process of the session `s1`, ignoring `SIGHUP` when `deaf`, and writes the lock of `ws` naming it,
    /// with a wait mark naming it when `waiting`.
    fn session(&mut self, waiting: bool, deaf: bool) -> u32 {
        let script = if deaf {
            "trap '' HUP; sleep 60"
        } else {
            "exec sleep 60"
        };
        let mut child: Child = Command::new("sh").args(["-c", script]).spawn().unwrap();
        let pid = child.id();
        // reaped at once, so `kill -0` no longer sees it once it ended
        std::thread::spawn(move || child.wait());
        self.session = Some(pid);
        std::fs::write(self.path("pid"), pid.to_string()).unwrap();
        self.lock(pid, waiting);
        pid
    }

    fn git_path(&self, name: &str) -> PathBuf {
        PathBuf::from(git(
            &self.ws,
            &["rev-parse", "--path-format=absolute", "--git-path", name],
        ))
    }

    fn lock(&self, pid: u32, waiting: bool) {
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
    }

    /// The panes of the tab `foo` (id 4): the session's (`terminal_7`), and a shell when `shell`.
    fn tab(&self, shell: bool) {
        let mut panes = vec![
            r#"{"id":1,"is_plugin":true,"tab_id":4,"tab_name":"foo","terminal_command":null}"#.to_string(),
            r#"{"id":7,"is_plugin":false,"tab_id":4,"tab_name":"foo","terminal_command":"claude /issue-workflow issues/foo.md"}"#.to_string(),
            r#"{"id":3,"is_plugin":false,"tab_id":0,"tab_name":"Tab #1","terminal_command":null}"#.to_string(),
        ];
        if shell {
            panes.push(
                r#"{"id":8,"is_plugin":false,"tab_id":4,"tab_name":"foo","terminal_command":null}"#
                    .to_string(),
            );
        }
        std::fs::write(self.path("panes"), format!("[{}]", panes.join(","))).unwrap();
    }

    /// `cardinal-ui --end <stem>` run in `dir`, in a zellij session when `zellij`: exit code, stdout, stderr.
    fn end(&self, dir: &Path, stem: &str, zellij: bool) -> (Option<i32>, String, String) {
        self.ui(dir, &["--end", stem], zellij)
    }

    /// `cardinal-ui --end-merged` run in `dir`, in a zellij session: exit code, stdout, stderr.
    fn end_merged(&self, dir: &Path) -> (Option<i32>, String, String) {
        self.ui(dir, &["--end-merged"], true)
    }

    /// The linked worktree `ws-<name>` on the branch `<name>`, holding the close of `<name>`, merged into `trunk` when
    /// `merged`.
    fn worktree(&self, name: &str, merged: bool) -> PathBuf {
        let ws = self.path(&format!("ws-{name}"));
        git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                name,
                ws.to_str().unwrap(),
                "trunk",
            ],
        );
        std::fs::create_dir_all(ws.join("issues/done")).unwrap();
        let close = format!("issues/done/{name}.md");
        std::fs::write(ws.join(&close), "closed\n").unwrap();
        git(&ws, &["add", &close]);
        git(&ws, &["commit", "-qm", "close", "--no-verify"]);
        if merged {
            git(&self.repo, &["merge", "-q", "--ff-only", name]);
        }
        std::fs::canonicalize(ws).unwrap()
    }

    /// `cardinal-ui args` run in `dir`, in a zellij session when `zellij`: exit code, stdout, stderr.
    fn ui(&self, dir: &Path, args: &[&str], zellij: bool) -> (Option<i32>, String, String) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"));
        cmd.args(args)
            .current_dir(dir)
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

    /// The `worktree close` lines of the log.
    fn closes(&self) -> String {
        self.log()
            .lines()
            .filter(|l| l.starts_with("worktree close"))
            .flat_map(|l| [l, "\n"])
            .collect()
    }

    fn closed(&self) -> String {
        self.closed_at(&self.ws)
    }

    fn closed_at(&self, ws: &Path) -> String {
        format!(
            "worktree close {} in {}\n",
            ws.display(),
            self.repo.display()
        )
    }
}

/// Waits a little for `pid` to end, as the reaper thread may lag behind the exit of `cardinal-ui`.
fn ended(pid: u32) -> bool {
    let start = Instant::now();
    while alive(pid) && start.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(20));
    }
    !alive(pid)
}

// spec: SUA-H3, SUA-8, SUA-9, STU-3
#[test]
fn a_waiting_session_alone_in_its_tab_ends_with_the_tab_then_the_worktree_closes() {
    let mut p = project(true);
    let pid = p.session(true, false);
    p.tab(false);
    let (code, out, err) = p.end(&p.repo, "foo", true);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("ended foo"), "{out}");
    assert!(ended(pid), "the session ended");
    let log = p.log();
    assert!(
        log.contains("zellij action list-panes --json --all\nzellij action close-tab-by-id 4\n"),
        "{log}"
    );
    assert!(log.ends_with(&p.closed()), "{log}");
}

// spec: SUA-9
#[test]
fn a_tab_holding_another_pane_keeps_it_and_loses_the_session_pane() {
    let mut p = project(true);
    let pid = p.session(true, false);
    p.tab(true);
    let (code, out, err) = p.end(&p.repo, "foo", true);
    assert_eq!(code, Some(0), "{out}{err}");
    let log = p.log();
    assert!(
        log.contains("zellij action close-pane --pane-id terminal_7\n"),
        "{log}"
    );
    assert!(!log.contains("close-tab"), "{log}");
    assert!(ended(pid));
}

// spec: SUA-9
#[test]
fn a_session_with_no_tab_of_the_ui_gets_sighup() {
    let mut p = project(true);
    let pid = p.session(true, false);
    let (code, out, err) = p.end(&p.repo, "foo", false);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(ended(pid));
    assert_eq!(p.log(), p.closed(), "no zellij outside zellij");
}

// spec: SUA-9
#[test]
fn an_ended_session_with_no_tab_only_closes_the_worktree() {
    let p = project(true);
    p.lock(ENDED_PID, false);
    let (code, out, err) = p.end(&p.repo, "foo", true);
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(
        p.log(),
        format!("zellij action list-panes --json --all\n{}", p.closed())
    );
}

// spec: SUA-9
#[test]
fn a_session_deaf_to_sighup_keeps_the_worktree() {
    let mut p = project(true);
    let pid = p.session(true, true);
    let (code, _, err) = p.end(&p.repo, "foo", false);
    assert_eq!(code, Some(1));
    assert!(err.contains("session s1 still runs after 10 s"), "{err}");
    assert!(alive(pid));
    assert_eq!(p.log(), "", "no worktree close");
}

// spec: SUA-9
#[test]
fn a_failed_worktree_close_names_what_was_done() {
    let mut p = project(true);
    p.fakes(1, "the branch foo is not merged into trunk");
    p.session(true, false);
    p.tab(false);
    let (code, _, err) = p.end(&p.repo, "foo", true);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("closed tab foo; ended session s1; worktree close: exit 1: the branch foo is not merged into trunk"),
        "{err}"
    );
}

// spec: SUA-8
#[test]
fn a_session_at_work_is_refused() {
    let mut p = project(true);
    let pid = p.session(false, false);
    p.tab(false);
    let (code, _, err) = p.end(&p.repo, "foo", true);
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!("session s1 is at work in {}", p.ws.display())),
        "{err}"
    );
    assert!(alive(pid));
    assert_eq!(p.log(), "");
}

// spec: SUA-8
#[test]
fn a_trunk_without_the_close_is_refused() {
    let mut p = project(false);
    p.session(true, false);
    let (code, _, err) = p.end(&p.repo, "foo", true);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("trunk does not hold the close of foo: merge it first"),
        "{err}"
    );
    assert_eq!(p.log(), "");
}

// spec: SUA-8
#[test]
fn an_untracked_file_in_the_worktree_is_refused() {
    let mut p = project(true);
    let pid = p.session(true, false);
    std::fs::write(p.ws.join("note"), "x").unwrap();
    let (code, _, err) = p.end(&p.repo, "foo", true);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("has uncommitted changes or untracked files"),
        "{err}"
    );
    assert!(alive(pid));
    assert_eq!(p.log(), "");
}

// spec: SUA-8
#[test]
fn the_ui_run_in_the_worktree_ends_nothing() {
    let mut p = project(true);
    p.session(true, false);
    let (code, _, err) = p.end(&p.ws.join("issues"), "foo", true);
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!("cardinal-ui runs in {}", p.ws.display())),
        "{err}"
    );
    assert_eq!(p.log(), "");
}

// spec: SUA-8
#[test]
fn no_linked_worktree_on_the_branch_or_a_detached_main_is_refused() {
    let p = project(true);
    let (code, _, err) = p.end(&p.repo, "bar", true);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("bar: no linked worktree on branch bar"),
        "{err}"
    );
    let (_, _, err) = p.end(&p.repo, "trunk", true);
    assert!(
        err.contains("trunk: no linked worktree on branch trunk"),
        "{err}"
    );
    git(&p.repo, &["checkout", "-q", "--detach"]);
    let (code, _, err) = p.end(&p.repo, "foo", true);
    assert_eq!(code, Some(1));
    assert!(err.contains("main worktree not on a branch"), "{err}");
    assert_eq!(p.log(), "");
}

// spec: SUA-9
#[test]
fn a_failing_list_panes_or_an_exited_pane_falls_back_to_sighup() {
    for panes in [
        "FAIL".to_string(),
        r#"[{"id":7,"is_plugin":false,"exited":true,"tab_id":4,"tab_name":"foo","terminal_command":"claude /issue-workflow issues/foo.md"}]"#.to_string(),
    ] {
        let mut p = project(true);
        let pid = p.session(true, false);
        std::fs::write(p.path("panes"), &panes).unwrap();
        let (code, out, err) = p.end(&p.repo, "foo", true);
        assert_eq!(code, Some(0), "{panes}: {out}{err}");
        assert!(ended(pid), "{panes}: the session got SIGHUP");
        let log = p.log();
        assert!(!log.contains("close-tab") && !log.contains("close-pane"), "{log}");
        assert!(log.ends_with(&p.closed()), "{log}");
    }
}

// spec: SUA-9
#[test]
fn a_pid_started_after_the_lock_is_another_process_and_gets_no_signal() {
    let mut p = project(true);
    let pid = p.session(true, false);
    let lock = p.git_path("cardinal-session");
    let old = std::time::SystemTime::now() - Duration::from_secs(600);
    std::fs::File::options()
        .write(true)
        .open(&lock)
        .unwrap()
        .set_modified(old)
        .unwrap();
    let (code, out, err) = p.end(&p.repo, "foo", false);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(alive(pid), "the process that took the number lives");
    assert_eq!(p.log(), p.closed());
}

// spec: SUA-H6, SUA-14
#[test]
fn every_merged_worktree_ends_and_an_open_issue_keeps_its_worktree() {
    let p = project(true);
    let bar = p.worktree("bar", true);
    p.worktree("baz", false);
    let (code, out, err) = p.end_merged(&p.repo);
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(out, "ended foo\nended bar\n");
    assert_eq!(p.closes(), format!("{}{}", p.closed(), p.closed_at(&bar)));
}

// spec: SUA-14
#[test]
fn a_refused_worktree_is_kept_with_its_reason_and_the_others_end() {
    let p = project(true);
    let bar = p.worktree("bar", true);
    std::fs::write(p.ws.join("new"), "x").unwrap();
    let (code, out, _) = p.end_merged(&p.repo);
    assert_eq!(code, Some(1));
    assert_eq!(
        out,
        format!(
            "kept foo: {} has uncommitted changes or untracked files\nended bar\n",
            p.ws.display()
        )
    );
    assert_eq!(p.closes(), p.closed_at(&bar));
}

// spec: SUA-14
#[test]
fn the_worktree_the_ui_runs_in_is_kept() {
    let p = project(true);
    let bar = p.worktree("bar", true);
    let (code, out, _) = p.end_merged(&p.ws.join("issues"));
    assert_eq!(code, Some(1));
    assert!(
        out.starts_with(&format!(
            "kept foo: cardinal-ui runs in {}: run it from another worktree\n",
            p.ws.display()
        )),
        "{out}"
    );
    assert!(out.ends_with("ended bar\n"), "{out}");
    assert_eq!(p.closes(), p.closed_at(&bar));
}

// spec: SUA-14
#[test]
fn no_merged_worktree_ends_nothing() {
    let p = project(false);
    let (code, out, err) = p.end_merged(&p.repo);
    assert_eq!(code, Some(0), "{err}");
    assert_eq!(out, "no merged worktree\n");
    assert_eq!(p.log(), "");
}

// spec: SUA-14
#[test]
fn a_worktree_now_serving_an_open_issue_is_left_as_it_is() {
    let p = project(true);
    git(&p.ws, &["checkout", "-q", "-b", "bar"]);
    let (code, out, _) = p.end_merged(&p.repo);
    assert_eq!((code, out.as_str()), (Some(0), "no merged worktree\n"));
    assert_eq!(p.log(), "");
}

// spec: SUA-14
#[test]
fn a_detached_main_worktree_ends_nothing() {
    let p = project(true);
    git(&p.repo, &["checkout", "-q", "--detach"]);
    let (code, out, err) = p.end_merged(&p.repo);
    assert_eq!(code, Some(1), "{out}");
    assert!(err.contains("main worktree not on a branch"), "{err}");
    assert_eq!(p.log(), "");
}
