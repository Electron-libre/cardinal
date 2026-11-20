//! Behavior test for `cardinal-ui --release` (STU-26, STU-27, STU-28), against a throwaway repository whose main
//! worktree `repo` is on `trunk` with a linked worktree `ws`, a fake `cardinal` and a fake `zellij` that log their
//! calls.

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

fn script(path: &Path, body: &str) {
    std::fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A document of `release --json` (REL-15) listing the milestone `New`.
const AWAITS: &str = r#"{"version":1,"proposed":"0.3.0","last":"v0.2.0","commits":2,"prepared":false,"milestones":[{"name":"New","document":"ROADMAP.md"}]}"#;

/// A document of `release --json` listing no milestone.
const NOTHING: &str =
    r#"{"version":1,"proposed":null,"last":"v0.2.0","commits":0,"prepared":false,"milestones":[]}"#;

/// A pid no process can have: above the largest `pid_max` of Linux (2^22).
const ENDED_PID: u32 = 99_999_999;

struct Project {
    root: tempfile::TempDir,
    repo: PathBuf,
    ws: PathBuf,
    log: PathBuf,
}

fn project() -> Project {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("repo");
    std::fs::create_dir_all(repo.join(".claude/skills/release")).unwrap();
    std::fs::write(repo.join(".claude/skills/release/SKILL.md"), "release\n").unwrap();
    git(&repo, &["init", "-q", "-b", "trunk", "."]);
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-qm", "init", "--no-verify"]);
    let ws = root.path().join("ws");
    git(
        &repo,
        &["worktree", "add", "-q", "-b", "x", ws.to_str().unwrap()],
    );
    let log = root.path().join("log");
    std::fs::write(&log, "").unwrap();
    Project {
        repo: std::fs::canonicalize(repo).unwrap(),
        ws: std::fs::canonicalize(ws).unwrap(),
        root,
        log,
    }
}

impl Project {
    /// The fake `cardinal`: `run release --json` logs its directory, then runs `release`.
    fn cardinal(&self, release: &str) -> PathBuf {
        let path = self.root.path().join("cardinal");
        let body = format!(
            "case \"$1 $2 $3\" in\n  'run release --json') echo \"release in $(pwd)\" >> '{log}'; {release} ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n",
            log = self.log.display()
        );
        script(&path, &body);
        path
    }

    /// The fake `zellij`: `list-tabs` prints the tabs `tabs` (JSON); `new-tab` logs each argument on a line, then
    /// exits `code` with `err` on stderr.
    fn zellij(&self, tabs: &str, code: i32, err: &str) -> PathBuf {
        let path = self.root.path().join("zellij");
        let body = format!(
            "if [ \"$2\" = list-tabs ]; then printf '%s\\n' '{tabs}'; exit 0; fi\necho zellij >> '{log}'; for a in \"$@\"; do printf '%s\\n' \"$a\" >> '{log}'; done\nprintf '%s' '{err}' >&2; exit {code}\n",
            log = self.log.display()
        );
        script(&path, &body);
        path
    }

    /// `cardinal-ui --release` run in the linked worktree, in a zellij session when `zellij` is given.
    fn release(&self, cardinal: &Path, zellij: Option<&Path>) -> (Option<i32>, String, String) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"));
        cmd.arg("--release")
            .current_dir(&self.ws)
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

    /// The log of a release tab opened in the main worktree, after the `release --json` run there.
    fn opened(&self) -> String {
        format!(
            "release in {r}\nzellij\naction\nnew-tab\n--cwd\n{r}\n--name\nrelease\n--\nclaude\n/release\n",
            r = self.repo.display()
        )
    }
}

fn awaits() -> String {
    format!("printf '%s\\n' '{AWAITS}'; exit 0")
}

// spec: STU-H8, STU-26, STU-27, STU-28
#[test]
fn a_milestone_awaiting_its_release_opens_a_release_tab_in_the_main_worktree() {
    let p = project();
    let cardinal = p.cardinal(&awaits());
    let zellij = p.zellij(r#"[{"name":"x"}]"#, 0, "");
    let ws_head = git(&p.ws, &["rev-parse", "HEAD"]);
    let (code, out, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(p.log(), p.opened());
    assert!(out.contains("release"), "{out}");
    assert_eq!(git(&p.ws, &["rev-parse", "HEAD"]), ws_head);
    assert_eq!(git(&p.ws, &["status", "--porcelain"]), "");
}

// spec: STU-27
#[test]
fn outside_zellij_no_release_starts() {
    let p = project();
    let cardinal = p.cardinal(&awaits());
    let (code, _, err) = p.release(&cardinal, None);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("release needs zellij: run cardinal-ui in a zellij session"),
        "{err}"
    );
    assert_eq!(p.log(), "");
}

// spec: STU-27
#[test]
fn no_milestone_awaiting_its_release_opens_no_tab() {
    let p = project();
    let cardinal = p.cardinal(&format!("printf '%s\\n' '{NOTHING}'; exit 0"));
    let zellij = p.zellij("[]", 0, "");
    let (code, _, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(err.contains("no milestone awaits its release"), "{err}");
    assert!(!p.log().contains("zellij"), "{}", p.log());
}

// spec: STU-27
#[test]
fn a_release_that_cannot_be_read_is_named() {
    let p = project();
    let cardinal = p.cardinal("echo 'cannot read the tags' >&2; exit 3");
    let zellij = p.zellij("[]", 0, "");
    let (code, _, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(
        err.contains("release: exit 3: cannot read the tags"),
        "{err}"
    );
    assert!(!p.log().contains("zellij"), "{}", p.log());
}

// spec: STU-27
#[test]
fn a_dirty_or_detached_main_worktree_opens_no_tab() {
    let p = project();
    let cardinal = p.cardinal(&awaits());
    let zellij = p.zellij("[]", 0, "");
    std::fs::write(p.repo.join("new.txt"), "x").unwrap();
    let (code, _, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(
        err.contains("main worktree has uncommitted changes or untracked files"),
        "{err}"
    );
    std::fs::remove_file(p.repo.join("new.txt")).unwrap();
    git(&p.repo, &["checkout", "-q", "--detach"]);
    let (code, _, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(err.contains("main worktree not on a branch"), "{err}");
    assert!(!p.log().contains("zellij"), "{}", p.log());
}

// spec: STU-27
#[test]
fn a_live_session_in_the_main_worktree_holds_it() {
    let p = project();
    let cardinal = p.cardinal(&awaits());
    let zellij = p.zellij("[]", 0, "");
    p.lock(std::process::id());
    let (code, _, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(
        err.contains("session s1 works in the main worktree"),
        "{err}"
    );
    assert!(!p.log().contains("zellij"), "{}", p.log());
    p.lock(ENDED_PID);
    let (code, out, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(p.log().ends_with("claude\n/release\n"), "{}", p.log());
}

// spec: STU-27
#[test]
fn an_open_release_tab_is_not_opened_twice() {
    let p = project();
    let cardinal = p.cardinal(&awaits());
    let zellij = p.zellij(r#"[{"name":"x"},{"name":"release"}]"#, 0, "");
    let (code, _, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(err.contains("a release tab is already open"), "{err}");
    assert!(!p.log().contains("zellij"), "{}", p.log());
}

// spec: STU-27
#[test]
fn a_project_that_turned_the_skill_off_opens_no_tab() {
    let p = project();
    git(&p.repo, &["rm", "-q", ".claude/skills/release/SKILL.md"]);
    git(&p.repo, &["commit", "-qm", "off", "--no-verify"]);
    let cardinal = p.cardinal(&awaits());
    let zellij = p.zellij("[]", 0, "");
    let (code, _, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(
        err.contains("the skill release is turned off: run the steps cardinal run release prints"),
        "{err}"
    );
    assert!(!p.log().contains("zellij"), "{}", p.log());
}

// spec: STU-27
#[test]
fn a_failed_zellij_is_named() {
    let p = project();
    let cardinal = p.cardinal(&awaits());
    let zellij = p.zellij("[]", 2, "no session\nmore");
    let (code, _, err) = p.release(&cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(
        err.contains("zellij action new-tab: exit 2: no session"),
        "{err}"
    );
}
