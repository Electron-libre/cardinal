//! Behavior test for the hooks `cardinal tool workspace-lock check | start | release` (`specs/workspace-lock.md`):
//! hook events on stdin, the process of the session in `CLAUDE_PID`, against a throwaway repository with a linked
//! worktree.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

/// A repository `main` and its linked worktree `ws`.
struct Repo {
    _root: tempfile::TempDir,
    main: PathBuf,
    ws: PathBuf,
}

fn repo() -> Repo {
    let root = tempfile::tempdir().unwrap();
    let main = root.path().join("main");
    std::fs::create_dir_all(&main).unwrap();
    git(&main, &["init", "-q", "-b", "trunk", "."]);
    git(&main, &["commit", "-q", "--allow-empty", "-m", "init"]);
    let ws = root.path().join("ws");
    git(
        &main,
        &["worktree", "add", "-q", "-b", "ws", ws.to_str().unwrap()],
    );
    Repo {
        _root: root,
        main,
        ws,
    }
}

/// What one hook run gave: (exit code, stdout, stderr).
type Out = (Option<i32>, String, String);

fn spawn(
    mode: &str,
    cwd: &Path,
    session: &str,
    pid: &str,
    input: Option<&str>,
) -> std::process::Child {
    let event =
        serde_json::json!({"session_id": session, "cwd": cwd, "hook_event_name": mode}).to_string();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.args(["tool", "workspace-lock", mode])
        .current_dir(cwd)
        .env_remove("CLAUDE_PID")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if !pid.is_empty() {
        cmd.env("CLAUDE_PID", pid);
    }
    let mut child = cmd.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.unwrap_or(&event).as_bytes())
        .unwrap();
    child
}

fn finish(child: std::process::Child) -> Out {
    let out = child.wait_with_output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn hook(mode: &str, cwd: &Path, session: &str, pid: &str) -> Out {
    finish(spawn(mode, cwd, session, pid, None))
}

/// The lock of the worktree of `dir`, empty when there is none.
fn lock_path(dir: &Path) -> PathBuf {
    PathBuf::from(git(
        dir,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "cardinal-session",
        ],
    ))
}

fn lock(dir: &Path) -> String {
    std::fs::read_to_string(lock_path(dir)).unwrap_or_default()
}

/// Set the time the lock of `dir` was last written.
fn written_at(dir: &Path, time: SystemTime) {
    std::fs::File::options()
        .write(true)
        .open(lock_path(dir))
        .unwrap()
        .set_modified(time)
        .unwrap();
}

/// Leave work in progress in `dir`: an untracked file.
fn dirty(dir: &Path) {
    std::fs::write(dir.join("wip"), "wip").unwrap();
}

/// Claim the issue `stem` in the state `working` for `session` of process `pid` in the worktree `dir` (CLM-1).
fn claim(dir: &Path, stem: &str, session: &str, pid: &str) {
    claim_as(dir, stem, session, pid, dir, "working");
}

/// Claim the issue `stem` from the repository of `dir` with the worktree `worktree` and the state `state`.
fn claim_as(dir: &Path, stem: &str, session: &str, pid: &str, worktree: &Path, state: &str) {
    let blob = dir.join(".git").join("claim-blob");
    std::fs::write(
        &blob,
        format!(
            "session {session}\npid {pid}\nworktree {}\nbranch trunk\nstate {state}\n",
            worktree.display()
        ),
    )
    .unwrap();
    let oid = git(dir, &["hash-object", "-w", blob.to_str().unwrap()]);
    git(
        dir,
        &["update-ref", &format!("refs/cardinal/claims/{stem}"), &oid],
    );
}

fn live() -> String {
    std::process::id().to_string()
}

/// A second live process, killed when dropped.
struct Other(std::process::Child);

impl Other {
    fn new() -> Self {
        Other(Command::new("sleep").arg("30").spawn().unwrap())
    }
    fn pid(&self) -> String {
        self.0.id().to_string()
    }
}

impl Drop for Other {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn dead() -> String {
    let mut child = Command::new("true").spawn().unwrap();
    let pid = child.id().to_string();
    child.wait().unwrap();
    pid
}

// spec: WSL-1
#[test]
fn check_takes_a_free_or_stale_lock_and_lets_the_call_through() {
    let r = repo();
    let (code, _, err) = hook("check", &r.main, "a", &dead());
    assert_eq!(code, Some(0), "{err}");
    assert!(lock(&r.main).contains("session a\n"));
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "a dead holder frees the lock: {err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
}

// spec: WSL-1
#[test]
fn check_takes_a_lock_whose_pid_a_later_process_took() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    dirty(&r.main);
    // long before `other` started: the session that wrote it ended and `other` took its number
    written_at(&r.main, UNIX_EPOCH + Duration::from_hours(24));
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "a stale lock is free: {err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
}

// spec: WSL-1, WSL-3
#[test]
fn check_refuses_while_the_holder_started_before_the_last_write() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    dirty(&r.main);
    written_at(&r.main, SystemTime::now() + Duration::from_hours(1));
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "{err}");
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-2
#[test]
fn check_lets_the_holder_and_its_process_through() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    let (code, _, _) = hook("check", &r.main, "a", &live());
    assert_eq!(code, Some(0), "the same session in a new process");
    assert_eq!(lock(&r.main), format!("session a\npid {}\n", live()));
    let (code, _, _) = hook("check", &r.main, "c", &live());
    assert_eq!(code, Some(0), "a cleared session of the same process");
    assert!(lock(&r.main).contains("session c\n"));
}

// spec: WSL-3
#[test]
fn check_refuses_another_session_while_the_holder_runs() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    dirty(&r.main);
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "{err}");
    assert!(
        err.contains("another live session") && err.contains("(a)") && err.contains("worktree"),
        "{err}"
    );
    assert!(
        err.contains(&format!("process {}", other.pid()))
            && err.contains("    ?? wip")
            && err.contains("git stash -u"),
        "the refusal names the process and the paths that keep the lock: {err}"
    );
    assert!(
        err.contains(&format!("this session (b) of process {}", live())),
        "the refusal names the calling session and process: {err}"
    );
    for i in 0..12 {
        std::fs::write(r.main.join(format!("more{i}")), "").unwrap();
    }
    let (_, _, err) = hook("check", &r.main, "b", &live());
    assert!(
        err.contains("and 3 more"),
        "a long list says what it leaves out: {err}"
    );
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-4
#[test]
fn two_sessions_taking_the_lock_at_once_leave_one_holder() {
    for _ in 0..10 {
        let r = repo();
        dirty(&r.main);
        let other = Other::new();
        let a = spawn("check", &r.main, "a", &live(), None);
        let b = spawn("check", &r.main, "b", &other.pid(), None);
        let (a, b) = (finish(a).0, finish(b).0);
        assert!(
            (a == Some(0)) ^ (b == Some(0)),
            "exactly one holds it: {a:?} {b:?}"
        );
        let winner = if a == Some(0) { "a" } else { "b" };
        assert!(lock(&r.main).contains(&format!("session {winner}\n")));
    }
}

// spec: WSL-5
#[test]
fn start_takes_the_lock_or_tells_the_agent_the_worktree_is_busy() {
    let r = repo();
    let other = Other::new();
    let (code, out, _) = hook("start", &r.main, "a", &other.pid());
    assert_eq!(code, Some(0));
    assert!(out.is_empty(), "{out}");
    assert!(lock(&r.main).contains("session a\n"));
    dirty(&r.main);
    let (code, out, _) = hook("start", &r.main, "b", &live());
    assert_eq!(code, Some(0), "SessionStart cannot block");
    assert!(out.contains("busy") && out.contains("refused"), "{out}");
    assert!(
        out.contains(&format!("process {}", other.pid())) && out.contains("    ?? wip"),
        "{out}"
    );
    assert!(
        out.contains(&format!("this session (b) of process {}", live())),
        "{out}"
    );
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-10
#[test]
fn a_holder_with_nothing_in_progress_hands_the_lock_over() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "a clean worktree and no claim: {err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
    dirty(&r.main);
    let (code, _, err) = hook("check", &r.main, "a", &other.pid());
    assert_eq!(
        code,
        Some(2),
        "the old holder is refused once b has work in progress: {err}"
    );
    assert!(lock(&r.main).contains("session b\n"));
}

// spec: WSL-10
#[test]
fn start_takes_over_the_lock_of_a_holder_with_nothing_in_progress() {
    let r = repo();
    let other = Other::new();
    hook("start", &r.main, "a", &other.pid());
    let (code, out, _) = hook("start", &r.main, "b", &live());
    assert_eq!(code, Some(0));
    assert!(out.is_empty(), "no busy warning: {out}");
    assert!(lock(&r.main).contains("session b\n"));
}

// spec: WSL-10
#[test]
fn a_holder_with_a_working_claim_keeps_the_lock_in_a_clean_worktree() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    claim(&r.main, "foo", "a", &other.pid());
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "claimed by its session: {err}");
    assert!(
        err.contains("the issue foo it claims") && !err.contains("git stash"),
        "the refusal names the claim, not changes to stash: {err}"
    );
    claim(&r.main, "foo", "cleared", &other.pid());
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "claimed by its process: {err}");
    claim(&r.main, "foo", "z", &dead());
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "a claim of another session: {err}");
    assert!(lock(&r.main).contains("session b\n"));
}

// spec: WSL-H1
#[test]
fn a_busy_worktree_refuses_another_session_until_its_holder_ends() {
    let r = repo();
    let other = Other::new();
    let (code, _, _) = hook("start", &r.main, "a", &other.pid());
    assert_eq!(code, Some(0));
    dirty(&r.main);
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "{err}");
    assert!(err.contains("(a)"), "{err}");
    let (code, out, err) = hook("release", &r.main, "a", &other.pid());
    assert_eq!((code, out.as_str(), err.as_str()), (Some(0), "", ""));
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(0), "{err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
}

// spec: WSL-6
#[test]
fn release_deletes_only_its_own_lock() {
    let r = repo();
    hook("check", &r.main, "a", &live());
    let (code, _, _) = hook("release", &r.main, "b", &live());
    assert_eq!(code, Some(0));
    assert!(lock(&r.main).contains("session a\n"));
    hook("release", &r.main, "a", &live());
    assert_eq!(lock(&r.main), "");
}

// spec: WSL-7
#[test]
fn a_broken_input_or_no_repository_lets_the_call_through() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    let (code, _, _) = finish(spawn("check", &r.main, "b", &live(), Some("not json")));
    assert_eq!(code, Some(0), "unreadable input");
    let (code, _, _) = hook("check", &r.main, "b", "");
    assert_eq!(code, Some(0), "no CLAUDE_PID");
    let outside = tempfile::tempdir().unwrap();
    let (code, _, _) = hook("check", outside.path(), "b", &live());
    assert_eq!(code, Some(0), "no repository");
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-9
#[test]
fn a_linked_worktree_has_its_own_lock() {
    let r = repo();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    let (code, _, err) = hook("check", &r.ws, "b", &live());
    assert_eq!(code, Some(0), "{err}");
    assert!(lock(&r.ws).contains("session b\n"));
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-8
#[test]
fn the_shipped_settings_entries_declare_the_three_hooks() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../entries/.claude/settings.json"),
    )
    .unwrap();
    let entries: serde_json::Value = serde_json::from_str(&text).unwrap();
    let declared = |event: &str, matcher: Option<&str>, command: &str| {
        entries["hooks"][event].as_array().is_some_and(|groups| {
            groups.iter().any(|g| {
                matcher.is_none_or(|m| g["matcher"] == m)
                    && g["hooks"]
                        .as_array()
                        .is_some_and(|h| h.iter().any(|h| h["command"] == command))
            })
        })
    };
    assert!(declared(
        "PreToolUse",
        Some("Bash|Edit|Write|NotebookEdit"),
        "cardinal run workspace-lock check"
    ));
    assert!(declared(
        "SessionStart",
        None,
        "cardinal run workspace-lock start"
    ));
    assert!(declared(
        "SessionEnd",
        None,
        "cardinal run workspace-lock release"
    ));
}

/// Run `mode` for `session` of process `pid` with a `transcript_path` in `transcripts`.
fn hook_with_transcript(
    mode: &str,
    cwd: &Path,
    session: &str,
    pid: &str,
    transcripts: &Path,
) -> Out {
    let event = serde_json::json!({
        "session_id": session,
        "cwd": cwd,
        "hook_event_name": mode,
        "transcript_path": transcripts.join(format!("{session}.jsonl")),
    })
    .to_string();
    finish(spawn(mode, cwd, session, pid, Some(&event)))
}

/// Write the transcript of `session` in `transcripts`, ending with a `continued-in` record naming `next` when given.
fn transcript(transcripts: &Path, session: &str, next: Option<&str>) {
    let mut records =
        vec![serde_json::json!({"type": "mode", "mode": "normal", "sessionId": session})];
    if let Some(next) = next {
        records.push(
            serde_json::json!({"type": "continued-in", "sessionId": session, "continuedInSessionId": next}),
        );
    }
    records.push(serde_json::json!({"type": "last-prompt", "lastPrompt": "ok"}));
    let text = records
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(transcripts.join(format!("{session}.jsonl")), text).unwrap();
}

// spec: WSL-11
#[test]
fn a_session_continued_in_another_hands_it_the_lock() {
    let r = repo();
    let transcripts = tempfile::tempdir().unwrap();
    let other = Other::new();
    hook("check", &r.ws, "a", &other.pid());
    dirty(&r.ws);
    transcript(transcripts.path(), "a", Some("c"));
    let (code, _, err) = hook_with_transcript("check", &r.ws, "b", &live(), transcripts.path());
    assert_eq!(
        code,
        Some(2),
        "a continuation of another session is refused: {err}"
    );
    let (code, out, _) = hook_with_transcript("start", &r.ws, "c", &live(), transcripts.path());
    assert_eq!(code, Some(0));
    assert!(out.is_empty(), "the fork takes the lock silently: {out}");
    assert_eq!(lock(&r.ws), format!("session c\npid {}\n", live()));
}

// spec: WSL-11
#[test]
fn check_takes_the_lock_of_the_session_it_continues_and_refuses_without_a_transcript() {
    let r = repo();
    let transcripts = tempfile::tempdir().unwrap();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    dirty(&r.main);
    let (code, _, err) = hook_with_transcript("check", &r.main, "b", &live(), transcripts.path());
    assert_eq!(code, Some(2), "no transcript of the holder: {err}");
    let (code, _, err) = hook("check", &r.main, "b", &live());
    assert_eq!(code, Some(2), "no transcript_path: {err}");
    transcript(transcripts.path(), "a", None);
    let (code, _, _) = hook_with_transcript("check", &r.main, "b", &live(), transcripts.path());
    assert_eq!(code, Some(2), "a transcript with no continued-in record");
    transcript(transcripts.path(), "a", Some("b"));
    let (code, _, err) = hook_with_transcript("check", &r.main, "b", &live(), transcripts.path());
    assert_eq!(code, Some(0), "{err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
}

/// Append `line` to the transcript of `session` in `transcripts`.
fn append(transcripts: &Path, session: &str, line: &[u8]) {
    let path = transcripts.join(format!("{session}.jsonl"));
    let mut text = std::fs::read(&path).unwrap_or_default();
    text.extend_from_slice(line);
    text.push(b'\n');
    std::fs::write(path, text).unwrap();
}

// spec: WSL-11
#[test]
fn only_a_last_continued_in_record_hands_the_lock_over() {
    let r = repo();
    let transcripts = tempfile::tempdir().unwrap();
    let other = Other::new();
    hook("check", &r.main, "a", &other.pid());
    dirty(&r.main);
    let t = transcripts.path();
    append(t, "a", b"\xff\xfe not utf-8");
    append(t, "a", br#"{"type":"continued-in","timestamp":"2026-11-22T20:09:24.722Z","continuedInSessionId":"b"}"#);
    append(
        t,
        "a",
        br#"{"type":"user","timestamp":"2026-11-22T20:09:24.646Z"}"#,
    );
    append(
        t,
        "a",
        br#"{"type":"assistant","timestamp":"2026-11-22T20:30:00.000Z"}"#,
    );
    let (code, _, err) = hook_with_transcript("check", &r.main, "b", &live(), t);
    assert_eq!(
        code,
        Some(2),
        "the user went back to a after the fork: {err}"
    );
    append(t, "a", br#"{"type":"continued-in","timestamp":"2026-11-22T20:31:00.000Z","continuedInSessionId":"b"}"#);
    append(
        t,
        "a",
        br#"{"type":"user","timestamp":"2026-11-22T20:30:59.000Z"}"#,
    );
    let (code, _, err) = hook_with_transcript("check", &r.main, "b", &live(), t);
    assert_eq!(
        code,
        Some(0),
        "a later fork, past a line that is not UTF-8: {err}"
    );
    assert!(lock(&r.main).contains("session b\n"));
}

// spec: WSL-11
#[test]
fn a_holder_session_that_is_no_file_name_has_no_transcript() {
    let r = repo();
    let root = tempfile::tempdir().unwrap();
    let transcripts = root.path().join("project");
    std::fs::create_dir_all(root.path().join("elsewhere")).unwrap();
    std::fs::create_dir_all(&transcripts).unwrap();
    let other = Other::new();
    hook("check", &r.main, "../elsewhere/a", &other.pid());
    dirty(&r.main);
    transcript(&root.path().join("elsewhere"), "a", Some("b"));
    let (code, _, err) = hook_with_transcript("check", &r.main, "b", &live(), &transcripts);
    assert_eq!(code, Some(2), "{err}");
}

/// The blob of the claim of `stem`, empty when there is none.
fn claim_of(dir: &Path, stem: &str) -> String {
    let oid = Command::new("git")
        .args(["rev-parse", "--verify", "-q"])
        .arg(format!("refs/cardinal/claims/{stem}"))
        .current_dir(dir)
        .output()
        .unwrap();
    let oid = String::from_utf8_lossy(&oid.stdout).trim().to_string();
    if oid.is_empty() {
        return String::new();
    }
    git(dir, &["cat-file", "blob", &oid])
}

/// Run `issue-claim issues/<stem>.md` in `dir` as `session` of process `pid`.
fn issue_claim(dir: &Path, stem: &str, session: &str, pid: &str) -> Out {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "issue-claim", &format!("issues/{stem}.md")])
        .current_dir(dir)
        .env("CLAUDE_CODE_SESSION_ID", session)
        .env("CLAUDE_PID", pid)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// spec: WSL-12
#[test]
fn a_session_continued_in_another_hands_it_its_working_claims() {
    let r = repo();
    let transcripts = tempfile::tempdir().unwrap();
    let other = Other::new();
    std::fs::create_dir_all(r.main.join("issues")).unwrap();
    std::fs::write(r.main.join("issues/foo.md"), "# Foo\n").unwrap();
    git(&r.main, &["add", "issues"]);
    git(&r.main, &["commit", "-q", "-m", "foo"]);
    hook("check", &r.main, "a", &other.pid());
    claim(&r.main, "foo", "a", &other.pid());
    transcript(transcripts.path(), "a", Some("b"));
    let (code, _, err) = hook_with_transcript("check", &r.main, "b", &live(), transcripts.path());
    assert_eq!(code, Some(0), "{err}");
    assert_eq!(
        claim_of(&r.main, "foo"),
        format!(
            "session b\npid {}\nworktree {}\nbranch trunk\nstate working",
            live(),
            r.main.display()
        )
    );
    let (code, out, err) = issue_claim(&r.main, "foo", "b", &live());
    assert_eq!(code, Some(0), "the fork claims its issue again: {out}{err}");
    let third = Other::new();
    let (code, _, err) = hook("check", &r.main, "c", &third.pid());
    assert_eq!(code, Some(2), "the moved claim keeps the lock: {err}");
    assert!(err.contains("the issue foo"), "{err}");
}

// spec: WSL-12
#[test]
fn a_handover_moves_only_the_working_claims_of_the_holder_in_its_worktree() {
    let r = repo();
    let transcripts = tempfile::tempdir().unwrap();
    let (holder, stranger) = (Other::new(), Other::new());
    hook("check", &r.main, "a", &holder.pid());
    dirty(&r.main);
    claim(&r.main, "cleared", "s1", &holder.pid());
    claim(&r.main, "theirs", "z", &stranger.pid());
    claim_as(
        &r.main,
        "pending",
        "a",
        &holder.pid(),
        &r.main,
        "merge pending",
    );
    claim_as(&r.main, "elsewhere", "a", &holder.pid(), &r.ws, "working");
    let unchanged = ["theirs", "pending", "elsewhere"].map(|s| claim_of(&r.main, s));
    transcript(transcripts.path(), "a", Some("b"));
    let (code, _, err) = hook_with_transcript("check", &r.main, "b", &live(), transcripts.path());
    assert_eq!(code, Some(0), "{err}");
    assert_eq!(lock(&r.main), format!("session b\npid {}\n", live()));
    assert!(
        claim_of(&r.main, "cleared").starts_with(&format!("session b\npid {}\n", live())),
        "a claim of the holder's process moves too"
    );
    assert_eq!(
        ["theirs", "pending", "elsewhere"].map(|s| claim_of(&r.main, s)),
        unchanged
    );
}

// spec: WSL-13
// spec: WSL-12
#[test]
fn the_session_a_user_returns_to_takes_the_lock_and_its_claims_back_from_the_fork() {
    let r = repo();
    let transcripts = tempfile::tempdir().unwrap();
    let fork = Other::new();
    hook("check", &r.main, "b", &fork.pid());
    claim(&r.main, "foo", "b", &fork.pid());
    let t = transcripts.path();
    append(t, "a", br#"{"type":"continued-in","timestamp":"2026-11-22T20:09:24.722Z","continuedInSessionId":"b"}"#);
    append(
        t,
        "a",
        br#"{"type":"user","timestamp":"2099-01-01T00:00:00.000Z"}"#,
    );
    let (code, _, err) = hook_with_transcript("check", &r.main, "a", &live(), t);
    assert_eq!(code, Some(0), "the user went back to a: {err}");
    assert_eq!(lock(&r.main), format!("session a\npid {}\n", live()));
    assert!(
        claim_of(&r.main, "foo").starts_with(&format!("session a\npid {}\n", live())),
        "the claim comes back to a"
    );
    let (code, _, err) = hook_with_transcript("check", &r.main, "b", &fork.pid(), t);
    assert_eq!(code, Some(2), "the fork left is refused: {err}");
    assert!(err.contains("the issue foo"), "{err}");
}

// spec: WSL-13
#[test]
fn only_a_record_after_the_last_continued_in_gives_the_lock_back() {
    let r = repo();
    let transcripts = tempfile::tempdir().unwrap();
    let fork = Other::new();
    hook("check", &r.main, "b", &fork.pid());
    dirty(&r.main);
    let t = transcripts.path();
    append(t, "a", br#"{"type":"continued-in","timestamp":"2026-11-22T20:09:24.722Z","continuedInSessionId":"b"}"#);
    append(
        t,
        "a",
        br#"{"type":"user","timestamp":"2026-11-22T20:09:24.646Z"}"#,
    );
    let (code, _, err) = hook_with_transcript("check", &r.main, "a", &live(), t);
    assert_eq!(code, Some(2), "no record stamped after the fork: {err}");
    let (code, _, err) = hook("check", &r.main, "a", &live());
    assert_eq!(code, Some(2), "no transcript_path: {err}");
    append(t, "c", br#"{"type":"continued-in","timestamp":"2026-11-22T20:09:24.722Z","continuedInSessionId":"d"}"#);
    append(
        t,
        "c",
        br#"{"type":"user","timestamp":"2099-01-01T00:00:00.000Z"}"#,
    );
    let (code, _, err) = hook_with_transcript("check", &r.main, "c", &live(), t);
    assert_eq!(code, Some(2), "no record naming the holder: {err}");
    append(t, "a", br#"{"type":"continued-in","timestamp":"2026-11-22T20:20:00.000Z","continuedInSessionId":"e"}"#);
    append(
        t,
        "a",
        br#"{"type":"assistant","timestamp":"2099-01-01T00:00:00.000Z"}"#,
    );
    let (code, out, err) = hook_with_transcript("start", &r.main, "a", &live(), t);
    assert_eq!(code, Some(0), "a later fork the user left too: {err}");
    assert!(
        out.is_empty(),
        "the session returned to takes the lock silently: {out}"
    );
    assert!(lock(&r.main).contains("session a\n"));
}

// spec: WSL-13
#[test]
fn a_fork_that_took_the_lock_after_the_return_keeps_it() {
    let r = repo();
    let transcripts = tempfile::tempdir().unwrap();
    let fork = Other::new();
    let t = transcripts.path();
    append(t, "a", br#"{"type":"continued-in","timestamp":"2000-01-01T00:00:00.000Z","continuedInSessionId":"b"}"#);
    append(
        t,
        "a",
        br#"{"type":"user","timestamp":"2000-01-01T00:01:00.000Z"}"#,
    );
    hook("check", &r.main, "b", &fork.pid());
    dirty(&r.main);
    append(
        t,
        "a",
        br#"{"type":"user","timestamp":"2099-01-01T00:00:00.000Z"}"#,
    );
    let (code, _, err) = hook_with_transcript("check", &r.main, "a", &live(), t);
    assert_eq!(
        code,
        Some(2),
        "b took the lock once a had ended, after the return: {err}"
    );
}
