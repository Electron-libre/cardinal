//! Fixtures of the behavior tests of `cardinal-ui --print`: throwaway git repositories and a fake `cardinal` (STU-5)
//! whose answers each test writes. Each test file uses a part of them.
#![allow(dead_code)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// A repository `repo` on `trunk` and a linked worktree `ws` on branch `ws`, next to a fake `cardinal`.
pub struct Project {
    pub root: tempfile::TempDir,
    pub repo: PathBuf,
    pub ws: PathBuf,
}

/// The answer of the fake `cardinal` to one tool: its stdout, its stderr and its exit code.
pub struct Answer<'a> {
    pub out: &'a str,
    pub err: &'a str,
    pub code: i32,
}

pub const fn ok(out: &str) -> Answer<'_> {
    Answer {
        out,
        err: "",
        code: 0,
    }
}

pub const QUEUE: &str = r#"{"version":1,"next":"b","issues":[
{"stem":"b","title":"Title b","priority":"high","why":"","milestone":"One","depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":null,"stale":false},
{"stem":"a","title":"Title a","priority":"low","why":"","milestone":"One","depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":{"by":"claim","state":"working","worktree":"WS","branch":"ws","session_alive":true},"stale":false}]}"#;
pub const ROADMAP: &str = r#"{"version":1,"milestones":[{"name":"One","state":"in issues","on_hold":null,"released":null,"document":"ROADMAP.md","issues":[{"path":"issues/a.md","stem":"a","open":true},{"path":"issues/b.md","stem":"b","open":true}]}]}"#;

/// The path `name` of the git directory of the worktree `dir`.
pub fn git_path(dir: &Path, name: &str) -> String {
    git(
        dir,
        &["rev-parse", "--path-format=absolute", "--git-path", name],
    )
}

pub fn project() -> Project {
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
    pub fn cardinal(&self, queue: &Answer, roadmap: &Answer) -> PathBuf {
        self.cardinal_releasing(queue, roadmap, None)
    }

    /// The fake `cardinal` of [`Project::cardinal`], whose `run release --json` gives `release` when given and
    /// leaves the file `release-ran` next to it.
    pub fn cardinal_releasing(
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
    pub fn lock(dir: &Path, id: &str, pid: &str) {
        std::fs::write(
            git_path(dir, "cardinal-session"),
            format!("session {id}\npid {pid}\n"),
        )
        .unwrap();
    }

    /// Marks the worktree `dir` as waiting for the user in the session `id` since `since` (SWT-3).
    pub fn waiting(dir: &Path, id: &str, since: u64, reason: &str) {
        std::fs::write(
            git_path(dir, "cardinal-waiting"),
            format!("session {id}\nsince {since}\nreason {reason}\n"),
        )
        .unwrap();
    }

    /// Writes a fake `zellij` whose `action list-tabs --json` prints `tabs` and exits `code`.
    pub fn zellij(&self, tabs: &str, code: i32) -> PathBuf {
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
pub fn ui(dir: &Path, cardinal: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    ui_in(dir, cardinal, None, args)
}

/// `cardinal-ui <args>` in `dir` with the fake `cardinal`, in a zellij session whose `zellij` is `zellij` when given
/// (SUA-10); the time zone is UTC.
pub fn ui_in(
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
pub fn ui_env(
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
pub fn command(dir: &Path, cardinal: &Path, args: &[&str]) -> Command {
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
        // the theme of the user is never read (SUS-4)
        .env("XDG_CONFIG_HOME", "/nonexistent/config")
        .env_remove("CARDINAL_UI_THEME");
    cmd
}

/// Runs `cmd`: exit code, stdout, stderr.
pub fn output(cmd: &mut Command) -> (Option<i32>, String, String) {
    let out = cmd.output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The line of `out` that holds `needle`.
pub fn line<'a>(out: &'a str, needle: &str) -> &'a str {
    out.lines()
        .find(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("no line with {needle} in\n{out}"))
}
