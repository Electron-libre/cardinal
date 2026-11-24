//! `cardinal` whose stdout reader closes before the end (`| head -1`): it ends as `cat` does, with no panic.
#![cfg(unix)]

use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};

/// `SIGPIPE` on every Unix the binary is released for (Linux, macOS).
const SIGPIPE: i32 = 13;

/// More milestones than a pipe buffers (64 KiB on Linux), so the tool still writes after the reader closed.
fn large_roadmap() -> String {
    let mut text = String::from("# Roadmap\n");
    for i in 0..4000 {
        let _ = write!(text, "\n## Milestone number {i}\n\n- State: idea\n");
    }
    text
}

// spec: BIN-23, BIN-24
#[test]
fn a_tool_whose_reader_closes_its_stdout_ends_by_sigpipe_without_a_panic() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join(".claude")).unwrap();
    std::fs::write(
        root.path().join(".claude/cardinal.toml"),
        "[project]\nroadmap = [\"ROADMAP.md\"]\n",
    )
    .unwrap();
    std::fs::write(root.path().join("ROADMAP.md"), large_roadmap()).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "roadmap"])
        .current_dir(root.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut first = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut first)
        .unwrap();
    // the reader is dropped here, as `head -1` exits after its line
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    let status = child.wait().unwrap();
    assert!(first.starts_with("| milestone |"), "{first}");
    assert_eq!(stderr, "");
    assert_eq!(status.signal(), Some(SIGPIPE), "{status:?}");
}
