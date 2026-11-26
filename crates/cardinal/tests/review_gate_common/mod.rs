//! Helpers of the behavior tests of `cardinal tool review-gate`: the runs of the hook, the git calls of the
//! fixtures, the reviewed and unreviewed repositories and the run of the named checks.
#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Exit code and stderr of the hook in `mode` for `event`, with `project` as the project directory.
pub fn run(hook: &Path, mode: &str, project: &Path, event: &serde_json::Value) -> (i32, String) {
    let mut child = Command::new(hook)
        .args(["tool", "review-gate", mode])
        .env("CLAUDE_PROJECT_DIR", project)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(event.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

pub fn review_done(hook: &Path, project: &Path, skill: &str) {
    let event = serde_json::json!({
        "tool_name": "Skill", "cwd": project, "tool_input": {"skill": skill}, "tool_response": {"success": true}
    });
    assert_eq!(run(hook, "record", project, &event).0, 0);
}

pub fn commit_attempt(hook: &Path, project: &Path, command: &str) -> (i32, String) {
    let event = serde_json::json!({"tool_name": "Bash", "cwd": project, "tool_input": {"command": command}});
    run(hook, "check", project, &event)
}

pub fn sh(project: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.email=t@t", "-c", "user.name=t"])
        .args(args)
        .current_dir(project)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?} failed");
}

/// The absolute git directory of the worktree `dir` is in.
pub fn git_dir_of(dir: &Path) -> std::path::PathBuf {
    let out = Command::new("git")
        .args(["rev-parse", "--absolute-git-dir"])
        .current_dir(dir)
        .output()
        .unwrap();
    Path::new(String::from_utf8_lossy(&out.stdout).trim()).to_path_buf()
}

/// Two repositories side by side: `current` reviewed and unchanged, `other` with a change nobody reviewed.
/// Returns the shell directory (`current`).
pub fn target_fixture(hook: &Path, _name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().unwrap().keep();
    let _ = fs::remove_dir_all(&root);
    let (current, other) = (root.join("current"), root.join("other"));
    for repo in [&current, &other] {
        fs::create_dir_all(repo).unwrap();
        sh(repo, &["init", "-q", "."]);
        fs::write(repo.join("a.txt"), "one\n").unwrap();
        sh(repo, &["add", "-A"]);
        sh(repo, &["commit", "-qm", "init"]);
        fs::write(repo.join("a.txt"), "two\n").unwrap();
    }
    review_done(hook, &current, "code-review");
    (current, other)
}

/// Exit code and stderr of `native` run in `cwd` with `env` added, as git runs a `pre-commit` hook: no event on stdin.
pub fn native_run(hook: &Path, cwd: &Path, env: &[(&str, &Path)]) -> (i32, String) {
    let out = Command::new(hook)
        .args(["tool", "review-gate", "native"])
        .current_dir(cwd)
        .envs(env.iter().map(|(k, v)| (*k, *v)))
        .env_remove("CLAUDE_PROJECT_DIR")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Prints each named check, `ok` or `FAIL`, and fails once all ran when one of them failed.
pub fn assert_checks(checks: impl IntoIterator<Item = (&'static str, bool)>) {
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
