//! Behavior test of `cardinal tool project-check`: the `check_command` of `.claude/cardinal.toml`, then the specs, in
//! a git project of a temporary directory.
#![cfg(unix)]

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::{tempdir, TempDir};

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn check(dir: &Path, args: &[&str]) -> Outcome {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "project-check"])
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    Outcome {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// A git project whose `.claude/cardinal.toml` holds `config`, with a subdirectory `sub/`.
fn project(config: &str) -> TempDir {
    let root = tempdir().unwrap();
    let dir = root.path();
    let ok = Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(dir)
        .status()
        .unwrap()
        .success();
    assert!(ok);
    fs::create_dir_all(dir.join(".claude")).unwrap();
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join(".claude/cardinal.toml"), config).unwrap();
    root
}

/// An accepted spec whose requirement no test cites: `check-specs` reports it.
fn untraced_spec(dir: &Path) {
    fs::create_dir_all(dir.join("specs")).unwrap();
    fs::write(
        dir.join("specs/rerun.md"),
        "---\nid: RER\nstatus: accepted\nadrs: []\n---\n\n# Rerun\n\n## Intent\n\nReuse.\n\n## Requirements\n\n\
         - **RER-1** (deterministic) A pass must reuse a saved result.\n\n## Scenarios\n\n- (RER-1) Given a, when b, \
         then c.\n\n## Non-goals\n\n- None.\n",
    )
    .unwrap();
}

// spec: PCK-1
#[test]
fn runs_the_check_command_with_sh_from_the_root() {
    let root = project("[project]\ncheck_command = \"pwd > ran.txt && echo noise\"\n");
    let out = check(&root.path().join("sub"), &[]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    let ran = fs::read_to_string(root.path().join("ran.txt")).unwrap();
    assert_eq!(
        fs::canonicalize(ran.trim()).unwrap(),
        fs::canonicalize(root.path()).unwrap()
    );
}

// spec: PCK-2
#[test]
fn fails_when_no_check_command_is_declared() {
    for config in [
        "[project]\nroot_docs = []\n",
        "[project]\ncheck_command = \"  \"\n",
        "[project]\ncheck_command = 1\n",
        "not toml [",
    ] {
        let root = project(config);
        let out = check(root.path(), &[]);
        assert_eq!(out.code, 1, "{config}");
        assert_eq!(out.stderr.lines().count(), 1, "{config}: {}", out.stderr);
        assert!(out.stderr.contains("check_command"), "{}", out.stderr);
        assert!(out.stderr.contains("[project]"), "{}", out.stderr);
        assert!(
            out.stderr.contains(".claude/cardinal.toml"),
            "{}",
            out.stderr
        );
    }
    let root = tempdir().unwrap();
    Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(root.path())
        .status()
        .unwrap();
    let out = check(root.path(), &[]);
    assert_eq!(out.code, 1);
    assert!(out.stderr.contains("check_command"), "{}", out.stderr);
}

// spec: PCK-3, PCK-H1
#[test]
fn is_quiet_on_success_and_prints_the_whole_output_of_a_failure() {
    let root = project("[project]\ncheck_command = \"echo all good; echo warn >&2\"\n");
    let out = check(root.path(), &[]);
    assert_eq!(
        (out.code, out.stdout.as_str(), out.stderr.as_str()),
        (0, "", "")
    );

    let root = project("[project]\ncheck_command = \"seq 1 100; echo broken >&2; exit 3\"\n");
    untraced_spec(root.path());
    let out = check(root.path(), &[]);
    assert_eq!(out.code, 1);
    let all = format!("{}{}", out.stdout, out.stderr);
    assert!(all.contains("\n100\n"), "{all}");
    assert!(
        all.find("100").unwrap() < all.find("broken").unwrap(),
        "{all}"
    );
    assert!(
        !all.contains("RER-1"),
        "the specs ran after a failure: {all}"
    );
}

// spec: PCK-4
#[test]
fn checks_the_specs_after_a_passing_command_unless_turned_off() {
    let root = project("[project]\ncheck_command = \"true\"\n");
    untraced_spec(root.path());
    let out = check(root.path(), &[]);
    assert_eq!(out.code, 1);
    assert!(out.stdout.contains("RER-1"), "{}", out.stdout);

    let root =
        project("[cardinal]\ndisabled = [\"specs\"]\n\n[project]\ncheck_command = \"true\"\n");
    untraced_spec(root.path());
    let out = check(root.path(), &[]);
    assert_eq!(
        (out.code, out.stdout.as_str(), out.stderr.as_str()),
        (0, "", "")
    );
}

// spec: PCK-5
#[test]
fn takes_no_argument() {
    let root = project("[project]\ncheck_command = \"touch ran.txt\"\n");
    let out = check(root.path(), &["--strict"]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("usage"), "{}", out.stderr);
    assert!(!root.path().join("ran.txt").exists());
}
