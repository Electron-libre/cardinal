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
    /// Writes the fake `cardinal`: `run next-issue --json` and `run roadmap --json` give their answer.
    fn cardinal(&self, queue: &Answer, roadmap: &Answer) -> PathBuf {
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
            "#!/bin/sh\n# the tools run at the root of the work tree (STU-5)\n[ -e .git ] || {{ echo \"not at the root: $PWD\" >&2; exit 9; }}\ncase \"$*\" in\n  'run next-issue --json') {} ;;\n  'run roadmap --open --json') {} ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n",
            arm(queue),
            arm(roadmap)
        );
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// Makes the session `id` of the process `pid` hold the lock of the worktree `dir` (WSL-9).
    fn lock(dir: &Path, id: &str, pid: &str) {
        let path = git(
            dir,
            &[
                "rev-parse",
                "--path-format=absolute",
                "--git-path",
                "cardinal-session",
            ],
        );
        std::fs::write(path, format!("session {id}\npid {pid}\n")).unwrap();
    }
}

/// `cardinal-ui <args>` in `dir` with the fake `cardinal`: exit code, stdout, stderr.
fn ui(dir: &Path, cardinal: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
        .args(args)
        .current_dir(dir)
        .env("CARDINAL_UI_CARDINAL", cardinal)
        .output()
        .unwrap();
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
    let a = line(&out, "  a  ");
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

// spec: STU-1, STU-4
#[test]
fn the_screen_holds_three_titled_panels_at_least_100_columns_wide() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let titles: Vec<&str> = out
        .lines()
        .filter_map(|l| l.strip_prefix("┌ ")?.split(' ').next())
        .collect();
    assert_eq!(titles, ["Milestones", "Queue", "Worktrees"], "{out}");
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
    assert!(out.contains("> issue0  low"), "the first issue");
    assert!(out.contains("  issue699  low"), "the last issue");
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
