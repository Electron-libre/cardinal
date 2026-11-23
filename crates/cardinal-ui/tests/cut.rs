//! Behavior test for `cardinal-ui --cut <milestone>` (SUA-19, SUA-20, SUA-21), against a throwaway repository whose
//! main worktree `repo` is on `trunk` with a linked worktree `ws`, a fake `cardinal` and a fake `zellij` that log
//! their calls.

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

/// A document of `roadmap --open --json` (RMP-8): `One` in issues, `Two` an idea, `Three` shaped.
const ROADMAP: &str = r#"{"version":1,"milestones":[{"name":"One","state":"in issues","on_hold":null,"document":"ROADMAP.md","issues":[{"path":"issues/a.md","stem":"a","open":true}]},{"name":"Two","state":"idea","on_hold":null,"document":"ROADMAP.md","issues":[]},{"name":"Three","state":"shaped","on_hold":null,"document":"ROADMAP.md","issues":[]}]}"#;

/// The first message of the session of the milestone `Two` (SUA-21).
const TWO: &str = "/create-issue cut the milestone Two into issues";

struct Project {
    root: tempfile::TempDir,
    repo: PathBuf,
    ws: PathBuf,
    log: PathBuf,
}

fn project() -> Project {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("repo");
    let skill = repo.join(".claude/skills/create-issue");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "create-issue\n").unwrap();
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
    /// The fake `cardinal`: `run roadmap --open --json` logs its directory, then runs `roadmap`.
    fn cardinal(&self, roadmap: &str) -> PathBuf {
        let path = self.root.path().join("cardinal");
        let body = format!(
            "case \"$1 $2 $3 $4\" in\n  'run roadmap --open --json') echo \"roadmap in $(pwd)\" >> '{log}'; {roadmap} ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n",
            log = self.log.display()
        );
        script(&path, &body);
        path
    }

    /// The fake `zellij`: `list-panes` prints the panes `panes` (JSON); `new-tab` logs each argument on a line, then
    /// exits `code` with `err` on stderr.
    fn zellij(&self, panes: &str, code: i32, err: &str) -> PathBuf {
        let path = self.root.path().join("zellij");
        let body = format!(
            "if [ \"$2\" = list-panes ]; then printf '%s\\n' '{panes}'; exit 0; fi\necho zellij >> '{log}'; for a in \"$@\"; do printf '%s\\n' \"$a\" >> '{log}'; done\nprintf '%s' '{err}' >&2; exit {code}\n",
            log = self.log.display()
        );
        script(&path, &body);
        path
    }

    /// `cardinal-ui --cut <milestone>` run in the linked worktree, in a zellij session when `zellij` is given.
    fn cut(
        &self,
        milestone: &str,
        cardinal: &Path,
        zellij: Option<&Path>,
    ) -> (Option<i32>, String, String) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"));
        cmd.args(["--cut", milestone])
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

    /// The log of the tab of the milestone `name` opened in the main worktree, after the `roadmap` run there.
    fn opened(&self, name: &str) -> String {
        format!(
            "roadmap in {r}\nzellij\naction\nnew-tab\n--cwd\n{r}\n--name\n{name}\n--\nclaude\n/create-issue cut the milestone {name} into issues\n",
            r = self.repo.display()
        )
    }
}

/// A pane of the tab `tab` running `command`, exited or not, as `list-panes --json --all` prints it.
fn pane(tab: &str, command: &str, exited: bool) -> String {
    format!(
        r#"{{"id":1,"is_plugin":false,"tab_id":3,"tab_name":"{tab}","terminal_command":"{command}","exited":{exited}}}"#
    )
}

fn roadmap() -> String {
    format!("printf '%s\\n' '{ROADMAP}'; exit 0")
}

// spec: SUA-H9, SUA-19, SUA-20, SUA-21, STU-3
#[test]
fn an_idea_milestone_opens_its_tab_in_the_main_worktree() {
    let p = project();
    let cardinal = p.cardinal(&roadmap());
    let zellij = p.zellij(&format!("[{}]", pane("x", "claude", false)), 0, "");
    let ws_head = git(&p.ws, &["rev-parse", "HEAD"]);
    let (code, out, err) = p.cut("Two", &cardinal, Some(&zellij));
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(p.log(), p.opened("Two"));
    assert_eq!(
        out.trim(),
        format!("started the cut of Two in {}", p.repo.display())
    );
    assert_eq!(git(&p.ws, &["rev-parse", "HEAD"]), ws_head);
    assert_eq!(git(&p.ws, &["status", "--porcelain"]), "");
}

// spec: SUA-19, SUA-20
#[test]
fn a_shaped_milestone_opens_its_tab() {
    let p = project();
    let cardinal = p.cardinal(&roadmap());
    let zellij = p.zellij("[]", 0, "");
    let (code, out, err) = p.cut("Three", &cardinal, Some(&zellij));
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(p.log(), p.opened("Three"));
}

// spec: SUA-20
#[test]
fn outside_zellij_no_cut_starts() {
    let p = project();
    let cardinal = p.cardinal(&roadmap());
    let (code, _, err) = p.cut("Two", &cardinal, None);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("cut needs zellij: run cardinal-ui in a zellij session"),
        "{err}"
    );
    assert_eq!(p.log(), "");
}

// spec: SUA-20
#[test]
fn a_milestone_in_issues_or_not_listed_opens_no_tab() {
    let p = project();
    let cardinal = p.cardinal(&roadmap());
    let zellij = p.zellij("[]", 0, "");
    let (code, _, err) = p.cut("One", &cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(err.contains("One: in issues, not shaped nor idea"), "{err}");
    let (code, _, err) = p.cut("Ghost", &cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(
        err.contains("Ghost: no open milestone of that name"),
        "{err}"
    );
    assert!(!p.log().contains("zellij"), "{}", p.log());
}

// spec: SUA-20
#[test]
fn a_roadmap_that_cannot_be_read_opens_no_tab() {
    let p = project();
    let cardinal = p.cardinal("echo 'cannot read ROADMAP.md' >&2; exit 3");
    let zellij = p.zellij("[]", 0, "");
    let (code, _, err) = p.cut("Two", &cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(
        err.contains("roadmap: exit 3: cannot read ROADMAP.md"),
        "{err}"
    );
    assert!(!p.log().contains("zellij"), "{}", p.log());
}

// spec: SUA-20
#[test]
fn a_project_without_the_skill_opens_no_tab() {
    let p = project();
    git(
        &p.repo,
        &["rm", "-q", ".claude/skills/create-issue/SKILL.md"],
    );
    git(&p.repo, &["commit", "-qm", "off", "--no-verify"]);
    let cardinal = p.cardinal(&roadmap());
    let zellij = p.zellij("[]", 0, "");
    let (code, _, err) = p.cut("Two", &cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(
        err.contains("the skill create-issue is turned off"),
        "{err}"
    );
    assert!(!p.log().contains("zellij"), "{}", p.log());
}

// spec: SUA-20
#[test]
fn a_dirty_main_worktree_opens_no_tab() {
    let p = project();
    std::fs::write(p.repo.join("notes.txt"), "x\n").unwrap();
    let cardinal = p.cardinal(&roadmap());
    let zellij = p.zellij("[]", 0, "");
    let (code, _, err) = p.cut("Two", &cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(
        err.contains("main worktree has uncommitted changes or untracked files"),
        "{err}"
    );
    assert_eq!(p.log(), "");
}

// spec: SUA-20
#[test]
fn a_tab_of_the_milestone_still_running_opens_no_other() {
    let p = project();
    let cardinal = p.cardinal(&roadmap());
    let running = p.zellij(
        &format!("[{}]", pane("Two", &format!("claude {TWO}"), false)),
        0,
        "",
    );
    let (code, _, err) = p.cut("Two", &cardinal, Some(&running));
    assert_eq!(code, Some(1));
    assert!(err.contains("a tab Two is already open"), "{err}");
    assert!(!p.log().contains("zellij"), "{}", p.log());
    let exited = p.zellij(
        &format!("[{}]", pane("Two", &format!("claude {TWO}"), true)),
        0,
        "",
    );
    let (code, out, err) = p.cut("Two", &cardinal, Some(&exited));
    assert_eq!(code, Some(0), "{out}{err}");
}

// spec: SUA-20
#[test]
fn a_zellij_that_fails_is_named() {
    let p = project();
    let cardinal = p.cardinal(&roadmap());
    let zellij = p.zellij("[]", 2, "no session");
    let (code, _, err) = p.cut("Two", &cardinal, Some(&zellij));
    assert_eq!(code, Some(1));
    assert!(err.contains("zellij action new-tab: exit 2"), "{err}");
}
