//! Helpers of the tests of the hooks `cardinal tool workspace-lock` (`workspace_lock.rs`, `workspace_lock_handover.rs`):
//! a throwaway repository with a linked worktree, the hook runs, the lock, the claims and the processes they name.
#![allow(dead_code)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

pub fn git(dir: &Path, args: &[&str]) -> String {
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
pub struct Repo {
    pub _root: tempfile::TempDir,
    pub main: PathBuf,
    pub ws: PathBuf,
}

pub fn repo() -> Repo {
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
pub type Out = (Option<i32>, String, String);

pub fn spawn(
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

pub fn finish(child: std::process::Child) -> Out {
    let out = child.wait_with_output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

pub fn hook(mode: &str, cwd: &Path, session: &str, pid: &str) -> Out {
    finish(spawn(mode, cwd, session, pid, None))
}

/// The lock of the worktree of `dir`, empty when there is none.
pub fn lock_path(dir: &Path) -> PathBuf {
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

pub fn lock(dir: &Path) -> String {
    std::fs::read_to_string(lock_path(dir)).unwrap_or_default()
}

/// Set the time the lock of `dir` was last written.
pub fn written_at(dir: &Path, time: SystemTime) {
    std::fs::File::options()
        .write(true)
        .open(lock_path(dir))
        .unwrap()
        .set_modified(time)
        .unwrap();
}

/// Leave work in progress in `dir`: an untracked file.
pub fn dirty(dir: &Path) {
    std::fs::write(dir.join("wip"), "wip").unwrap();
}

/// Claim the issue `stem` in the state `working` for `session` of process `pid` in the worktree `dir` (CLM-1).
pub fn claim(dir: &Path, stem: &str, session: &str, pid: &str) {
    claim_as(dir, stem, session, pid, dir, "working");
}

/// Claim the issue `stem` from the repository of `dir` with the worktree `worktree` and the state `state`.
pub fn claim_as(dir: &Path, stem: &str, session: &str, pid: &str, worktree: &Path, state: &str) {
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

/// Claim `stem` like `claim`, written at `since` (Unix seconds, CLM-11): before any process of the test started.
pub fn claim_since(dir: &Path, stem: &str, session: &str, pid: &str, since: u64) {
    claim(dir, stem, session, pid);
    let blob = dir.join(".git").join("claim-blob");
    let text = std::fs::read_to_string(&blob).unwrap();
    std::fs::write(&blob, format!("{text}since {since}\n")).unwrap();
    let oid = git(dir, &["hash-object", "-w", blob.to_str().unwrap()]);
    git(
        dir,
        &["update-ref", &format!("refs/cardinal/claims/{stem}"), &oid],
    );
}

pub fn live() -> String {
    std::process::id().to_string()
}

/// A second live process, killed when dropped.
pub struct Other(pub std::process::Child);

impl Other {
    pub fn new() -> Self {
        Other(Command::new("sleep").arg("30").spawn().unwrap())
    }
    pub fn pid(&self) -> String {
        self.0.id().to_string()
    }
}

impl Drop for Other {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub fn dead() -> String {
    let mut child = Command::new("true").spawn().unwrap();
    let pid = child.id().to_string();
    child.wait().unwrap();
    pid
}
