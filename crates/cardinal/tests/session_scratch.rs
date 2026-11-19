//! Behavior test for the hooks `cardinal tool session-scratch start | end` (`specs/session-scratch.md`): hook events on
//! stdin, the temporary directory in `TMPDIR`, the env file of Claude Code in `CLAUDE_ENV_FILE` and the process of the
//! session in `CLAUDE_PID`, all in a throwaway directory.

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A temporary directory standing for `/tmp`, and the env file of a session.
struct Machine {
    _root: tempfile::TempDir,
    tmp: PathBuf,
    env_file: PathBuf,
}

fn machine() -> Machine {
    let root = tempfile::tempdir().unwrap();
    let tmp = root.path().join("tmp");
    std::fs::create_dir(&tmp).unwrap();
    let env_file = root.path().join("env.sh");
    Machine {
        tmp,
        env_file,
        _root: root,
    }
}

/// A process id that runs: the test's own.
fn live() -> String {
    std::process::id().to_string()
}

/// A process id that runs no process.
const DEAD: &str = "999999999";

/// Runs `cardinal tool session-scratch <mode>` with `input` on stdin and the given environment; returns (exit code,
/// stdout, stderr).
fn run(
    mode: &str,
    input: &str,
    tmpdir: &Path,
    env_file: Option<&Path>,
    pid: &str,
) -> (Option<i32>, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.args(["tool", "session-scratch", mode])
        .env("TMPDIR", tmpdir)
        .env("CLAUDE_PID", pid)
        .env_remove("CLAUDE_ENV_FILE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(file) = env_file {
        cmd.env("CLAUDE_ENV_FILE", file);
    }
    let mut child = cmd.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn event(session: &str) -> String {
    serde_json::json!({"session_id": session, "hook_event_name": "SessionStart", "source": "startup"}).to_string()
}

impl Machine {
    fn start(&self, session: &str, pid: &str) -> (Option<i32>, String, String) {
        run(
            "start",
            &event(session),
            &self.tmp,
            Some(&self.env_file),
            pid,
        )
    }

    fn end(&self, session: &str) -> (Option<i32>, String, String) {
        run(
            "end",
            &event(session),
            &self.tmp,
            Some(&self.env_file),
            &live(),
        )
    }

    fn scratch(&self, session: &str) -> PathBuf {
        self.tmp.join(format!("claude-scratch-{session}"))
    }

    /// A scratch directory of `session` left by process `pid` (none: no record), its content and itself last changed
    /// `hours` ago.
    fn left(&self, session: &str, pid: Option<&str>, hours: u64, keep: bool) -> PathBuf {
        let dir = self.scratch(session);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub/out.log"), "x").unwrap();
        if let Some(pid) = pid {
            std::fs::write(dir.join(".cardinal-session"), format!("{pid}\n")).unwrap();
        }
        if keep {
            std::fs::write(dir.join("cardinal-keep"), "").unwrap();
        }
        age(&dir, hours);
        dir
    }
}

/// Sets the modification time of `dir` and of everything under it to `hours` ago.
fn age(dir: &Path, hours: u64) {
    let ok = Command::new("find")
        .arg(dir)
        .args([
            "-exec",
            "touch",
            "-h",
            "-d",
            &format!("{hours} hours ago"),
            "{}",
            "+",
        ])
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "touch {}", dir.display());
}

fn env_lines(m: &Machine) -> String {
    std::fs::read_to_string(&m.env_file).unwrap_or_default()
}

// spec: SCR-H1, SCR-1, SCR-2, SCR-7
#[test]
fn a_session_gets_a_scratch_directory_removed_at_its_end() {
    let m = machine();
    std::fs::write(&m.env_file, "export A=1\n").unwrap();
    let (code, out, err) = m.start("s1", &live());
    assert_eq!((code, out.as_str(), err.as_str()), (Some(0), "", ""));
    let dir = m.scratch("s1");
    assert!(dir.is_dir());
    let mode = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
    assert_eq!(
        env_lines(&m),
        format!("export A=1\nexport TMPDIR='{}'\n", dir.display())
    );
    assert_eq!(
        std::fs::read_to_string(dir.join(".cardinal-session")).unwrap(),
        format!("{}\n", live())
    );
    std::fs::create_dir(dir.join("measure")).unwrap();
    std::fs::write(dir.join("measure/out.log"), "x").unwrap();
    let (code, out, err) = m.end("s1");
    assert_eq!((code, out.as_str(), err.as_str()), (Some(0), "", ""));
    assert!(!dir.exists());
}

// spec: SCR-H2, SCR-5
#[test]
fn a_kept_directory_survives_the_end_of_its_session() {
    let m = machine();
    m.start("s1", &live());
    let dir = m.scratch("s1");
    std::fs::write(dir.join("report.md"), "x").unwrap();
    std::fs::write(dir.join("cardinal-keep"), "").unwrap();
    assert_eq!(m.end("s1").0, Some(0));
    assert!(dir.join("report.md").is_file());
}

// spec: SCR-3, SCR-5
#[test]
fn the_start_sweeps_the_directories_of_dead_sessions_left_for_a_day() {
    let m = machine();
    let dead_old = m.left("a", Some(DEAD), 48, false);
    let live_old = m.left("b", Some(&live()), 48, false);
    let dead_recent = m.left("c", Some(DEAD), 1, false);
    let kept = m.left("d", Some(DEAD), 48, true);
    let unrecorded = m.left("e", None, 48, false);
    // a file changed an hour ago in a directory unchanged for two days keeps it
    let touched = m.left("f", Some(DEAD), 48, false);
    age(&touched.join("sub/out.log"), 1);
    let file = m.tmp.join("claude-scratch-x");
    std::fs::write(&file, "x").unwrap();
    age(&file, 48);
    let other = m.tmp.join("other");
    std::fs::create_dir(&other).unwrap();
    age(&other, 48);
    assert_eq!(m.start("s", &live()).0, Some(0));
    assert!(!dead_old.exists());
    assert!(!unrecorded.exists());
    for kept in [&live_old, &dead_recent, &kept, &touched, &file, &other] {
        assert!(kept.exists(), "{}", kept.display());
    }
    assert!(m.scratch("s").is_dir());
}

// spec: SCR-4
#[test]
fn a_broken_input_or_environment_changes_nothing() {
    let m = machine();
    let outside = m.tmp.join("x");
    std::fs::create_dir(&outside).unwrap();
    for input in [
        event("../x"),
        event("a/b"),
        event(""),
        "not json".to_string(),
        "{}".to_string(),
    ] {
        for mode in ["start", "end"] {
            let out = run(
                mode,
                &input,
                &m.tmp.join("claude-scratch-y"),
                Some(&m.env_file),
                &live(),
            );
            assert_eq!(
                out,
                (Some(0), String::new(), String::new()),
                "{mode} {input}"
            );
        }
    }
    assert!(outside.is_dir());
    assert_eq!(std::fs::read_dir(&m.tmp).unwrap().count(), 1);
    assert_eq!(env_lines(&m), "");
    // no env file: nothing would use the directory
    let out = run("start", &event("s"), &m.tmp, None, &live());
    assert_eq!(out, (Some(0), String::new(), String::new()));
    assert!(!m.scratch("s").exists());
    // no process: the sweep of another session could not tell this one alive
    let out = run("start", &event("s"), &m.tmp, Some(&m.env_file), "");
    assert_eq!(out, (Some(0), String::new(), String::new()));
    assert!(!m.scratch("s").exists());
    // a temporary directory that does not exist: nothing created, nothing exported
    let missing = m.tmp.join("missing");
    let out = run("start", &event("s"), &missing, Some(&m.env_file), &live());
    assert_eq!(out, (Some(0), String::new(), String::new()));
    assert!(!missing.exists());
    assert_eq!(env_lines(&m), "");
}

// spec: SCR-6
#[test]
fn a_session_started_from_another_gets_its_directory_beside_it() {
    let m = machine();
    let parent = m.scratch("p");
    std::fs::create_dir(&parent).unwrap();
    let out = run("start", &event("s"), &parent, Some(&m.env_file), &live());
    assert_eq!(out.0, Some(0));
    assert!(m.scratch("s").is_dir());
    assert!(!parent.join("claude-scratch-s").exists());
    let out = run("end", &event("s"), &parent, Some(&m.env_file), &live());
    assert_eq!(out.0, Some(0));
    assert!(!m.scratch("s").exists());
}

// spec: SCR-7
#[test]
fn a_resumed_session_keeps_its_files() {
    let m = machine();
    m.start("s", DEAD);
    let dir = m.scratch("s");
    std::fs::write(dir.join("out.log"), "x").unwrap();
    m.start("s", &live());
    assert!(dir.join("out.log").is_file());
    assert_eq!(
        std::fs::read_to_string(dir.join(".cardinal-session")).unwrap(),
        format!("{}\n", live())
    );
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// spec: SCR-8
#[test]
fn the_shipped_settings_entries_declare_the_two_hooks() {
    let text = std::fs::read_to_string(repo_root().join("entries/.claude/settings.json")).unwrap();
    let entries: serde_json::Value = serde_json::from_str(&text).unwrap();
    let declared = |event: &str, command: &str| {
        entries["hooks"][event].as_array().is_some_and(|groups| {
            groups.iter().any(|g| {
                g["hooks"]
                    .as_array()
                    .is_some_and(|h| h.iter().any(|h| h["command"] == command))
            })
        })
    };
    assert!(declared(
        "SessionStart",
        "cardinal run session-scratch start"
    ));
    assert!(declared("SessionEnd", "cardinal run session-scratch end"));
}

// spec: SCR-9
#[test]
fn the_rule_asks_for_the_scratch_directory() {
    let rule =
        std::fs::read_to_string(repo_root().join("files/.claude/rules/scratch-files.md")).unwrap();
    for needed in ["$TMPDIR", "mktemp", "/tmp/<name>", "cardinal-keep"] {
        assert!(rule.contains(needed), "{needed}");
    }
}

// spec: SCR-10
#[test]
fn the_feature_is_the_two_hooks_and_the_rule() {
    let features: toml::Table = std::fs::read_to_string(repo_root().join("features.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let feature = features["session-scratch"].as_table().unwrap();
    let strings = |key: &str| -> Vec<&str> {
        feature[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect()
    };
    assert_eq!(strings("files"), [".claude/rules/scratch-files.md"]);
    assert_eq!(
        strings("entries"),
        [
            "hook SessionEnd cardinal run session-scratch end",
            "hook SessionStart cardinal run session-scratch start",
        ]
    );
    assert!(feature.get("needs").is_none());
}
