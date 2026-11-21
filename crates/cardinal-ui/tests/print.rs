//! Behavior test for `cardinal-ui --print`, against throwaway git repositories and a fake `cardinal` (STU-5) whose
//! answers each test writes.

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

/// A repository `repo` on `trunk` and a linked worktree `ws` on branch `ws`, next to a fake `cardinal`.
struct Project {
    root: tempfile::TempDir,
    repo: PathBuf,
    ws: PathBuf,
}

/// The answer of the fake `cardinal` to one tool: its stdout, its stderr and its exit code.
struct Answer<'a> {
    out: &'a str,
    err: &'a str,
    code: i32,
}

const fn ok(out: &str) -> Answer<'_> {
    Answer {
        out,
        err: "",
        code: 0,
    }
}

const QUEUE: &str = r#"{"version":1,"next":"b","issues":[
{"stem":"b","title":"Title b","priority":"high","why":"","milestone":"One","depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":null,"stale":false},
{"stem":"a","title":"Title a","priority":"low","why":"","milestone":"One","depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":{"by":"claim","state":"working","worktree":"WS","branch":"ws","session_alive":true},"stale":false}]}"#;
const ROADMAP: &str = r#"{"version":1,"milestones":[{"name":"One","state":"in issues","on_hold":null,"released":null,"document":"ROADMAP.md","issues":[{"path":"issues/a.md","stem":"a","open":true},{"path":"issues/b.md","stem":"b","open":true}]}]}"#;

/// The path `name` of the git directory of the worktree `dir`.
fn git_path(dir: &Path, name: &str) -> String {
    git(
        dir,
        &["rev-parse", "--path-format=absolute", "--git-path", name],
    )
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
        &["worktree", "add", "-q", "-b", "ws", ws.to_str().unwrap()],
    );
    let ws = std::fs::canonicalize(ws).unwrap();
    let repo = std::fs::canonicalize(repo).unwrap();
    Project { root, repo, ws }
}

impl Project {
    /// Writes the fake `cardinal`: `run next-issue --json` and `run roadmap --json` give their answer, the steps of a
    /// measure an empty one (STU-17); a call of `run release --json` fails the test.
    fn cardinal(&self, queue: &Answer, roadmap: &Answer) -> PathBuf {
        self.cardinal_releasing(queue, roadmap, None)
    }

    /// The fake `cardinal` of [`Project::cardinal`], whose `run release --json` gives `release` when given and
    /// leaves the file `release-ran` next to it.
    fn cardinal_releasing(
        &self,
        queue: &Answer,
        roadmap: &Answer,
        release: Option<&Answer>,
    ) -> PathBuf {
        let path = self.root.path().join("cardinal");
        let arm = |a: &Answer| {
            format!(
                "printf '%s\\n' '{}'; printf '%s' '{}' >&2; exit {}",
                a.out.replace('\'', "'\\''").replace('\n', ""),
                a.err.replace('\'', "'\\''"),
                a.code
            )
        };
        let script = format!(
            "#!/bin/sh\n# the tools run at the root of the work tree (STU-5)\n[ -e .git ] || {{ echo \"not at the root: $PWD\" >&2; exit 9; }}\ncase \"$*\" in\n  'run next-issue --json') {} ;;\n  'run roadmap --open --json') {} ;;\n{}  'run session-metrics' | 'run delivered-quality '*) exit 0 ;;\n  'run cardinal-report '*) echo '{{\"version\":1,\"tables\":[]}}'; exit 0 ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n",
            arm(queue),
            arm(roadmap),
            release.map_or_else(String::new, |r| format!(
                "  'run release --json') touch '{}'; {} ;;\n",
                self.root.path().join("release-ran").display(),
                arm(r)
            ))
        );
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// Makes the session `id` of the process `pid` hold the lock of the worktree `dir` (WSL-9).
    fn lock(dir: &Path, id: &str, pid: &str) {
        std::fs::write(
            git_path(dir, "cardinal-session"),
            format!("session {id}\npid {pid}\n"),
        )
        .unwrap();
    }

    /// Marks the worktree `dir` as waiting for the user in the session `id` since `since` (SWT-3).
    fn waiting(dir: &Path, id: &str, since: u64, reason: &str) {
        std::fs::write(
            git_path(dir, "cardinal-waiting"),
            format!("session {id}\nsince {since}\nreason {reason}\n"),
        )
        .unwrap();
    }

    /// Writes a fake `zellij` whose `action list-tabs --json` prints `tabs` and exits `code`.
    fn zellij(&self, tabs: &str, code: i32) -> PathBuf {
        let path = self.root.path().join("zellij");
        let script = format!(
            "#!/bin/sh\ncase \"$*\" in\n  'action list-tabs --json') printf '%s\\n' '{tabs}'; exit {code} ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n"
        );
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }
}

/// `cardinal-ui <args>` in `dir` with the fake `cardinal`, outside zellij: exit code, stdout, stderr.
fn ui(dir: &Path, cardinal: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    ui_in(dir, cardinal, None, args)
}

/// `cardinal-ui <args>` in `dir` with the fake `cardinal`, in a zellij session whose `zellij` is `zellij` when given
/// (SUA-10); the time zone is UTC.
fn ui_in(
    dir: &Path,
    cardinal: &Path,
    zellij: Option<&Path>,
    args: &[&str],
) -> (Option<i32>, String, String) {
    let mut cmd = command(dir, cardinal, args);
    if let Some(zellij) = zellij {
        cmd.env("ZELLIJ", "0").env("CARDINAL_UI_ZELLIJ", zellij);
    }
    output(&mut cmd)
}

/// `cardinal-ui <args>` of [`ui`] with the environment variables `envs` set, `None` for those removed.
fn ui_env(
    dir: &Path,
    cardinal: &Path,
    envs: &[(&str, Option<&Path>)],
    args: &[&str],
) -> (Option<i32>, String, String) {
    let mut cmd = command(dir, cardinal, args);
    for (name, value) in envs {
        match value {
            Some(v) => cmd.env(name, v),
            None => cmd.env_remove(name),
        };
    }
    output(&mut cmd)
}

/// The command `cardinal-ui <args>` in `dir` with the fake `cardinal`, outside zellij, in UTC, with no theme file.
fn command(dir: &Path, cardinal: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"));
    cmd.args(args)
        .current_dir(dir)
        .env("CARDINAL_UI_CARDINAL", cardinal)
        .env("TZ", "UTC")
        .env_remove("ZELLIJ")
        // outside zellij, a call of `zellij` fails the test
        .env("CARDINAL_UI_ZELLIJ", "/nonexistent/zellij")
        // the measures go next to the fake `cardinal`, never to the cache of the user (STU-18)
        .env("XDG_CACHE_HOME", cardinal.parent().unwrap().join("cache"))
        // the theme of the user is never read (STU-32)
        .env("XDG_CONFIG_HOME", "/nonexistent/config")
        .env_remove("CARDINAL_UI_THEME");
    cmd
}

/// Runs `cmd`: exit code, stdout, stderr.
fn output(cmd: &mut Command) -> (Option<i32>, String, String) {
    let out = cmd.output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The line of `out` that holds `needle`.
fn line<'a>(out: &'a str, needle: &str) -> &'a str {
    out.lines()
        .find(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("no line with {needle} in\n{out}"))
}

// spec: STU-H1, STU-1, STU-3, STU-4, STU-5, STU-7
#[test]
fn the_user_reads_the_milestones_the_queue_and_the_worktrees() {
    let p = project();
    let queue = QUEUE.replace("WS", p.ws.to_str().unwrap());
    let cardinal = p.cardinal(&ok(&queue), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", &std::process::id().to_string());
    let (code, out, err) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    let one = line(&out, "One  ");
    assert!(
        one.contains("in issues") && one.contains("2/2 open"),
        "{one}"
    );
    let b = line(&out, "> b");
    assert!(b.contains("high") && b.contains("ready"), "{b}");
    let a = line(&out, "  ⠿ a  ");
    assert!(
        a.contains(&format!("claimed in {}", p.ws.display())) && !a.contains("ended"),
        "{a}"
    );
    let ws = line(&out, &format!("{}  ", p.ws.display()));
    assert!(ws.contains("ws") && ws.contains("session s1"), "{ws}");
    assert!(!ws.contains("ended"), "{ws}");
    assert!(
        line(&out, &format!("{}  ", p.repo.display())).contains("free"),
        "{out}"
    );
    assert!(
        !p.root.path().join("repo/.git/cardinal-session").exists(),
        "STU-3: nothing written"
    );
}

// spec: STU-1, STU-7
#[test]
fn an_ended_session_is_shown_ended() {
    let p = project();
    let queue = QUEUE
        .replace("WS", p.ws.to_str().unwrap())
        .replace("\"session_alive\":true", "\"session_alive\":false");
    let cardinal = p.cardinal(&ok(&queue), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", "2147483646");
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(line(&out, "  a  ").contains("session ended"), "{out}");
    assert!(
        line(&out, &format!("{}  ", p.ws.display())).contains("session s1 ended"),
        "{out}"
    );
}

// spec: STU-1
#[test]
fn a_milestone_on_hold_says_so() {
    let p = project();
    let roadmap = ROADMAP.replace("\"on_hold\":null", "\"on_hold\":\"later\"");
    let cardinal = p.cardinal(&ok(QUEUE), &ok(&roadmap));
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(line(&out, "One  ").contains("on hold"), "{out}");
}

/// A roadmap of the milestone `New`, `done` in `ROADMAP.md`, not released.
const DONE: &str = r#"{"version":1,"milestones":[{"name":"New","state":"done","on_hold":null,"released":null,"document":"ROADMAP.md","issues":[{"path":"issues/done/a.md","stem":"a","open":false}]}]}"#;

/// The document of `release --json` (REL-15) proposing `proposed` (JSON) and listing `New` of `document`.
fn release(proposed: &str, document: &str) -> String {
    format!(
        r#"{{"version":1,"proposed":{proposed},"prepared":false,"last":"0.2.0","commits":2,"milestones":[{{"name":"New","document":"{document}"}}]}}"#
    )
}

// spec: STU-1, STU-3
#[test]
fn a_done_milestone_that_release_lists_awaits_its_release() {
    let p = project();
    let doc = release(r#""0.3.0""#, "ROADMAP.md");
    let cardinal = p.cardinal_releasing(&ok(QUEUE), &ok(DONE), Some(&ok(&doc)));
    let (code, out, err) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    let new = line(&out, "New  ");
    assert!(
        new.contains("done") && new.contains("awaits release v0.3.0"),
        "{new}"
    );
}

// spec: STU-1
#[test]
fn with_no_proposed_version_the_mark_has_no_version() {
    let p = project();
    let doc = release("null", "ROADMAP.md");
    let cardinal = p.cardinal_releasing(&ok(QUEUE), &ok(DONE), Some(&ok(&doc)));
    let (code, out, err) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(p.root.path().join("release-ran").exists(), "{out}");
    let new = line(&out, "New  ");
    assert!(new.contains("awaits release"), "{new}");
    assert!(!new.contains("awaits release v"), "{new}");
}

// spec: STU-1
#[test]
fn a_milestone_of_another_document_is_not_marked() {
    let p = project();
    let doc = release(r#""0.3.0""#, "docs/OTHER.md");
    let cardinal = p.cardinal_releasing(&ok(QUEUE), &ok(DONE), Some(&ok(&doc)));
    let (code, out, err) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(p.root.path().join("release-ran").exists(), "{out}");
    assert!(!line(&out, "New  ").contains("awaits release"), "{out}");
}

// spec: STU-3
#[test]
fn with_no_done_milestone_release_is_not_run() {
    let p = project();
    let doc = release(r#""0.3.0""#, "ROADMAP.md");
    let cardinal = p.cardinal_releasing(&ok(QUEUE), &ok(ROADMAP), Some(&ok(&doc)));
    let (code, out, err) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(!p.root.path().join("release-ran").exists(), "{out}");
    assert!(!out.contains("awaits release"), "{out}");
}

// spec: STU-6, STU-8, SUQ-1
#[test]
fn a_release_that_fails_heads_the_queue_with_its_error() {
    let p = project();
    let failing = Answer {
        out: "",
        err: "cannot read the tags\nmore",
        code: 3,
    };
    let cardinal = p.cardinal_releasing(&ok(QUEUE), &ok(DONE), Some(&failing));
    let (code, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(p.root.path().join("release-ran").exists(), "{out}");
    let row = out.lines().position(|l| l.contains("New  "));
    let error = out
        .lines()
        .position(|l| l.contains("release: exit 3: cannot read the tags"));
    assert!(
        matches!((row, error), (Some(r), Some(e)) if e < r),
        "the error heads the queue: {out}"
    );
    assert!(!line(&out, "New  ").contains("awaits release"), "{out}");
    assert_eq!(code, Some(1));
}

// spec: STU-6
#[test]
fn a_release_of_an_unsupported_version_is_an_error_above_the_milestones() {
    let p = project();
    let cardinal = p.cardinal_releasing(&ok(QUEUE), &ok(DONE), Some(&ok(r#"{"version":2}"#)));
    let (code, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let row = out.lines().position(|l| l.contains("New  "));
    let error = out
        .lines()
        .position(|l| l.contains("release: version 2 unsupported"));
    assert!(
        matches!((row, error), (Some(r), Some(e)) if e < r),
        "the error heads the queue: {out}"
    );
    assert_eq!(code, Some(1));
}

// spec: STU-6, STU-8
#[test]
fn a_tool_that_fails_shows_its_error_in_its_panel() {
    let p = project();
    let failing = Answer {
        out: "",
        err: "cannot run the tool\nmore",
        code: 3,
    };
    let cardinal = p.cardinal(&ok(QUEUE), &failing);
    let (code, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(
        out.contains("roadmap: exit 3: cannot run the tool"),
        "{out}"
    );
    assert!(out.contains("> b"), "the queue is still shown: {out}");
    assert_eq!(code, Some(1));
}

// spec: STU-6
#[test]
fn an_unsupported_version_or_text_is_an_error() {
    let p = project();
    let cardinal = p.cardinal(
        &ok("Next: issues/b.md"),
        &ok(r#"{"version":2,"milestones":[]}"#),
    );
    let (code, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(out.contains("roadmap: version 2 unsupported"), "{out}");
    assert!(out.contains("next-issue: not JSON"), "{out}");
    assert_eq!(code, Some(1));
}

// spec: STU-6
#[test]
fn the_problems_of_the_issue_files_are_shown_under_the_queue() {
    let p = project();
    let problems = Answer {
        out: QUEUE,
        err: "issue without a priority: c\n",
        code: 1,
    };
    let cardinal = p.cardinal(&problems, &ok(ROADMAP));
    let (code, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(out.contains("issue without a priority: c"), "{out}");
    assert!(out.contains("> b"), "{out}");
    assert_eq!(code, Some(0));
}

// spec: STU-8
#[test]
fn outside_a_repository_the_worktrees_panel_shows_an_error() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let outside = p.root.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
        .arg("--print")
        .current_dir(&outside)
        .env("CARDINAL_UI_CARDINAL", &cardinal)
        .env("GIT_CEILING_DIRECTORIES", p.root.path())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("git worktree list: exit"), "{text}");
    assert_eq!(out.status.code(), Some(1));
}

// spec: STU-8
#[test]
fn an_unknown_argument_is_refused() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let (code, out, err) = ui(&p.repo, &cardinal, &["--prnt"]);
    assert_eq!((code, out.as_str()), (Some(2), ""));
    assert!(err.contains("usage"), "{err}");
}

// spec: STU-6
#[test]
fn a_queue_that_fails_with_exit_1_and_no_document_shows_its_stderr() {
    let p = project();
    let failing = Answer {
        out: "",
        err: "no issues directory",
        code: 1,
    };
    let cardinal = p.cardinal(&failing, &ok(ROADMAP));
    let (code, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(
        out.contains("next-issue: exit 1: no issues directory"),
        "{out}"
    );
    assert_eq!(code, Some(1));
}

// spec: STU-1, STU-4, STU-16
#[test]
fn the_screen_holds_three_titled_panels_at_least_100_columns_wide() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let titles: Vec<&str> = out
        .lines()
        .filter_map(|l| l.strip_prefix("┌ ")?.split(' ').next())
        .collect();
    assert_eq!(titles, ["Queue", "Worktrees", "Measures"], "{out}");
    let width = out.lines().next().unwrap().chars().count();
    assert!(width >= 100, "{width}: {out}");
    assert!(out.lines().all(|l| l.chars().count() <= width), "{out}");
}

// spec: STU-4
#[test]
fn a_long_queue_is_printed_whole() {
    let p = project();
    let issues: Vec<String> = (0..700)
        .map(|i| format!(r#"{{"stem":"issue{i}","title":"t","priority":"low","why":"","milestone":null,"depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":null,"stale":false}}"#))
        .collect();
    let queue = format!(
        r#"{{"version":1,"next":"issue0","issues":[{}]}}"#,
        issues.join(",")
    );
    let cardinal = p.cardinal(&ok(&queue), &ok(ROADMAP));
    let (code, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0));
    assert!(line(&out, "> issue0 ").contains("  low"), "the first issue");
    assert!(
        line(&out, "  issue699 ").contains("  low"),
        "the last issue"
    );
    assert!(
        out.lines().last().unwrap().contains("q quit"),
        "the help line last"
    );
}

// spec: STU-4
#[test]
fn a_line_of_wide_characters_is_printed_whole() {
    let p = project();
    let name = "里程碑".repeat(30);
    let roadmap = ROADMAP.replace("\"name\":\"One\"", &format!("\"name\":\"{name}\""));
    let cardinal = p.cardinal(&ok(QUEUE), &ok(&roadmap));
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(line(&out, &name).contains("2/2 open"), "{out}");
}

// spec: STU-5
#[test]
fn from_a_subdirectory_the_tools_run_at_the_root() {
    let p = project();
    let sub = p.repo.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let (code, out, err) = ui(&sub, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("> b"), "{out}");
    let missing = p.root.path().join("them.toml");
    let (code, _, err) = ui_env(
        &p.repo,
        &cardinal,
        &[("CARDINAL_UI_THEME", Some(&missing))],
        &["--print"],
    );
    assert_eq!(code, Some(0), "{err}");
    assert!(
        err.starts_with(&format!("theme: {}: ", missing.display())),
        "a file the variable names is never missing in silence: {err}"
    );
}

// spec: STU-7
#[test]
fn a_lock_without_a_pid_is_free() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let path = git(
        &p.ws,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "cardinal-session",
        ],
    );
    std::fs::write(path, "session s1\n").unwrap();
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(
        line(&out, &format!("{}  ", p.ws.display())).contains("free"),
        "{out}"
    );
}

// spec: STU-8
#[test]
fn a_closed_output_is_no_panic() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let mut child = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
        .arg("--print")
        .current_dir(&p.repo)
        .env("CARDINAL_UI_CARDINAL", &cardinal)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let out = child.wait_with_output().unwrap();
    assert!(matches!(out.status.code(), Some(0 | 1)), "{out:?}");
}

/// 2025-01-02 09:05:00 UTC.
const NINE_O_FIVE: u64 = 1_735_808_700;

// spec: STU-H3, STU-13
#[test]
fn a_session_that_waits_for_the_user_is_flagged() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", &std::process::id().to_string());
    Project::waiting(&p.ws, "s1", NINE_O_FIVE, "stop");
    let (code, out, err) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    let ws = line(&out, &format!("{}  ", p.ws.display()));
    assert!(
        ws.contains("session s1  waiting since 2025-01-02 09:05: stop"),
        "a day other than today shows its date: {ws}"
    );
    assert!(!line(&out, &format!("{}  ", p.repo.display())).contains("waiting"));
}

// spec: STU-13
#[test]
fn a_session_waiting_since_today_shows_the_time_alone() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", &std::process::id().to_string());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Project::waiting(&p.ws, "s1", now, "stop");
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let ws = line(&out, &format!("{}  ", p.ws.display()));
    let at = ws.find("waiting since ").expect(ws) + "waiting since ".len();
    assert_eq!(&ws[at + 2..at + 3], ":", "HH:MM alone: {ws}");
    assert_eq!(&ws[at + 5..at + 7], ": ", "{ws}");
}

// spec: STU-13
#[test]
fn a_mark_of_an_ended_or_another_session_is_not_shown() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    Project::lock(&p.ws, "s1", "2147483646");
    Project::waiting(&p.ws, "s1", NINE_O_FIVE, "stop");
    Project::lock(&p.repo, "s2", &std::process::id().to_string());
    Project::waiting(&p.repo, "s3", NINE_O_FIVE, "stop");
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let ws = line(&out, &format!("{}  ", p.ws.display()));
    assert!(ws.contains("ended") && !ws.contains("waiting"), "{ws}");
    assert!(
        !line(&out, &format!("{}  ", p.repo.display())).contains("waiting"),
        "{out}"
    );
}

const TABS: &str =
    r#"[{"position":0,"name":"Tab #1","tab_id":0},{"position":1,"name":"a","tab_id":3}]"#;

// spec: STU-10, STU-3
#[test]
fn an_issue_with_a_tab_of_its_name_shows_it() {
    let p = project();
    let queue = QUEUE.replace("WS", p.ws.to_str().unwrap());
    let cardinal = p.cardinal(&ok(&queue), &ok(ROADMAP));
    let zellij = p.zellij(TABS, 0);
    let (code, out, err) = ui_in(&p.repo, &cardinal, Some(&zellij), &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(
        line(&out, "  ⠿ a  ")
            .trim_end_matches(['│', ' '])
            .ends_with("  tab"),
        "{out}"
    );
    assert!(!line(&out, "> b").contains("tab"), "{out}");
}

// spec: SUA-10
#[test]
fn outside_zellij_or_when_list_tabs_fails_no_tab_and_no_error() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let (code, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}");
    assert!(!line(&out, "  ⠿ a  ").contains("tab"), "{out}");
    let failing = p.zellij("", 1);
    let (code, out, _) = ui_in(&p.repo, &cardinal, Some(&failing), &["--print"]);
    assert_eq!(code, Some(0), "{out}");
    assert!(!line(&out, "  ⠿ a  ").contains("tab"), "{out}");
    let text = p.zellij("not json", 0);
    let (code, out, _) = ui_in(&p.repo, &cardinal, Some(&text), &["--print"]);
    assert_eq!(code, Some(0), "{out}");
}

// spec: STU-32, STU-33
#[test]
fn the_ignored_lines_of_the_theme_are_named_on_stderr_and_print_stays_plain() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let theme = p.root.path().join("theme.toml");
    std::fs::write(
        &theme,
        "# a theme\nborder = \"#112233\"  # the frame\naccent = \"blue\"\n\nshadow = \"#000000\"\ngarbage\n",
    )
    .unwrap();
    let (code, out, err) = ui_env(
        &p.repo,
        &cardinal,
        &[("CARDINAL_UI_THEME", Some(&theme))],
        &["--print"],
    );
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(
        err.lines().collect::<Vec<_>>(),
        [
            "theme: line 3 ignored: invalid color \"blue\"",
            "theme: line 5 ignored: unknown role shadow",
            "theme: line 6 ignored: not a role = \"#rrggbb\" line",
        ],
        "{err}"
    );
    assert!(!out.contains('\x1b'), "{out}");
}

// spec: STU-33
#[test]
fn a_theme_that_cannot_be_read_is_named_and_the_screen_shown() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let dir = p.root.path().join("theme-dir");
    std::fs::create_dir(&dir).unwrap();
    let (code, out, err) = ui_env(
        &p.repo,
        &cardinal,
        &[("CARDINAL_UI_THEME", Some(&dir))],
        &["--print"],
    );
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(
        err.starts_with(&format!("theme: {}: ", dir.display())),
        "{err}"
    );
    assert!(out.contains("> b"), "{out}");
}

// spec: STU-32
#[test]
fn the_theme_is_read_from_the_config_directory_and_a_missing_one_says_nothing() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let home = p.root.path().join("home");
    let config = p.root.path().join("config");
    for dir in [home.join(".config/cardinal-ui"), config.join("cardinal-ui")] {
        std::fs::create_dir_all(&dir).unwrap();
    }
    std::fs::write(home.join(".config/cardinal-ui/theme.toml"), "home\n").unwrap();
    std::fs::write(config.join("cardinal-ui/theme.toml"), "\nconfig\n").unwrap();
    let run = |xdg: &Path| {
        ui_env(
            &p.repo,
            &cardinal,
            &[
                ("CARDINAL_UI_THEME", None),
                ("HOME", Some(&home)),
                ("XDG_CONFIG_HOME", Some(xdg)),
            ],
            &["--print"],
        )
    };
    let (code, _, err) = run(&config);
    assert_eq!(code, Some(0), "{err}");
    assert!(err.starts_with("theme: line 2 ignored"), "{err}");
    let (_, _, err) = run(Path::new("config"));
    assert!(
        err.starts_with("theme: line 1 ignored"),
        "relative: $HOME/.config: {err}"
    );
    let (_, _, err) = run(&p.root.path().join("nothing"));
    assert_eq!(err, "", "a missing file says nothing");
}

/// An issue of `next-issue --json` (NXT-19), open and free.
fn open_issue(stem: &str, priority: &str, milestone: &str, depends_on: &str) -> String {
    format!(
        r#"{{"stem":"{stem}","title":"t","priority":"{priority}","why":"","milestone":{milestone},"depends_on":{depends_on},"state":"ready","blocked_by":[],"hold":null,"taken":null,"stale":false}}"#
    )
}

// spec: SUQ-H1, SUQ-1, SUQ-2, SUQ-3, STU-1
#[test]
fn the_user_reads_each_milestone_as_a_tree_of_its_issues() {
    let p = project();
    let one = r#""One""#;
    let issues = [
        open_issue("a", "high", one, "[]"),
        open_issue("b", "low", one, r#"["a"]"#),
        open_issue("c", "medium", one, r#"["a","b"]"#),
        open_issue("d", "low", "null", "[]"),
    ];
    let queue = format!(
        r#"{{"version":1,"next":"a","issues":[{}]}}"#,
        issues.join(",")
    );
    let roadmap = ROADMAP.replace(
        r#"{"path":"issues/b.md","stem":"b","open":true}"#,
        r#"{"path":"issues/b.md","stem":"b","open":true},{"path":"issues/c.md","stem":"c","open":true}"#,
    );
    let cardinal = p.cardinal(&ok(&queue), &ok(&roadmap));
    let (code, out, err) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0), "{out}{err}");
    let rows: Vec<&str> = ["One  ", "> a", "├─ b", "└─ c", "No milestone", "  d  "]
        .iter()
        .map(|needle| line(&out, needle))
        .collect();
    let at = |row: &str| out.lines().position(|l| l == row).unwrap();
    assert!(rows.windows(2).all(|w| at(w[0]) + 1 == at(w[1])), "{out}");
    assert!(rows[0].contains("in issues  3/3 open"), "{out}");
    assert!(
        rows[3]
            .trim_end_matches([' ', '│'])
            .ends_with("also after b"),
        "{out}"
    );
    assert!(!rows[2].contains("also after"), "{out}");
    let column = |row: &str, word: &str| row.find(word).map(|b| row[..b].chars().count());
    let priorities: Vec<_> = [(1, "high"), (2, "low"), (3, "medium"), (5, "low")]
        .iter()
        .map(|(n, word)| column(rows[*n], &format!("  {word}  ")))
        .collect();
    assert!(
        priorities
            .iter()
            .all(|c| c.is_some() && *c == priorities[0]),
        "{out}"
    );
}
