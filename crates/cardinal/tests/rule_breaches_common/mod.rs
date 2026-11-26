//! Helpers of the behavior tests of `cardinal tool rule-breaches`: the fixture repositories, the transcripts under
//! construction, the runs of the tool and the rows it prints.
#![allow(dead_code)]

use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const COLUMNS: &str = "session\tproject\trevision\trule\tkind\tlocation\tdetail";
pub const SUMMARY: &str = "rule\trevision\tapplied\tbreached\tbreaches\tblocked\tbytes";

pub fn sh(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(ok, "git {args:?} failed in {}", dir.display());
}

/// Commits `files` (path, content) in `repo` at `date`, with `message`; returns the abbreviated hash.
pub fn commit_at(repo: &Path, date: &str, message: &str, files: &[(&str, &str)]) -> String {
    for (path, body) in files {
        let file = repo.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, body).unwrap();
    }
    sh(repo, &["add", "-A", "--force"]);
    let out = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            message,
        ])
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "commit failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let head = Command::new("git")
        .args(["log", "-1", "--format=%h"])
        .current_dir(repo)
        .output()
        .unwrap();
    String::from_utf8_lossy(&head.stdout).trim().to_string()
}

pub fn new_repo(dir: &Path) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    sh(dir, &["init", "-q", "-b", "main"]);
    dir.to_path_buf()
}

/// A transcript under construction: each push returns the 1-based line number.
pub struct Transcript {
    pub cwd: String,
    pub ts: String,
    pub lines: Vec<String>,
}

impl Transcript {
    pub fn new(cwd: &Path, ts: &str) -> Self {
        Transcript {
            cwd: cwd.display().to_string(),
            ts: ts.to_string(),
            lines: Vec::new(),
        }
    }

    pub fn push(&mut self, mut line: serde_json::Value) -> usize {
        line["cwd"] = json!(self.cwd);
        line["timestamp"] = json!(self.ts);
        line["sessionId"] = json!("x");
        self.lines.push(line.to_string());
        self.lines.len()
    }

    pub fn typed(&mut self, text: &str) -> usize {
        self.push(json!({"type": "user", "gitBranch": "main", "message": {"role": "user", "content": text}}))
    }

    pub fn call(&mut self, id: &str, name: &str, input: &serde_json::Value) -> usize {
        self.push(json!({"type": "assistant", "message": {"id": format!("m{id}"), "content": [{"type": "tool_use", "id": id, "name": name, "input": input}]}}))
    }

    pub fn text(&mut self, id: &str, text: &str) -> usize {
        self.push(json!({"type": "assistant", "message": {"id": id, "content": [{"type": "text", "text": text}]}}))
    }

    pub fn result(&mut self, id: &str, error: bool, text: &str) -> usize {
        self.push(json!({"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": id, "is_error": error, "content": text}]}}))
    }

    /// A call and its result; returns the line of the call.
    pub fn run(
        &mut self,
        id: &str,
        name: &str,
        input: &serde_json::Value,
        error: Option<&str>,
    ) -> usize {
        let at = self.call(id, name, input);
        self.result(id, error.is_some(), error.unwrap_or("SECRET output"));
        at
    }

    pub fn write(&self, path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            path,
            self.lines
                .iter()
                .map(|l| l.clone() + "\n")
                .collect::<String>(),
        )
        .unwrap();
    }
}

pub fn bash(command: &str) -> serde_json::Value {
    json!({"command": command})
}

pub fn edit(path: &Path) -> serde_json::Value {
    json!({"file_path": path, "old_string": "SECRET old", "new_string": "SECRET new"})
}

pub fn run(dir: &Path, projects: &Path, summary: bool) -> (bool, String) {
    let mut cmd = tool_command(dir.join("rule_breaches.rs"));
    cmd.arg(projects);
    if summary {
        cmd.arg("--summary");
    }
    let out = cmd.output().unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// A finding row of the session `s` at revision `rev`.
pub fn row(s: &str, rev: &str, rule: &str, kind: &str, loc: &str, detail: &str) -> String {
    format!("{s}\tproj\t{rev}\t{rule}\t{kind}\t{loc}\t{detail}")
}

pub fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

pub fn tool_command(script: impl AsRef<std::path::Path>) -> std::process::Command {
    let stem = script
        .as_ref()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .replace('_', "-");
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.arg("tool").arg(stem);
    cmd
}
