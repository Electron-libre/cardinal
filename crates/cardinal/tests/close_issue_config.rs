//! Behavior test for `cardinal tool close-issue` on a broken `.claude/cardinal.toml`: refused before any file moves.

use std::path::Path;
use std::process::Command;

/// A git repository holding the committed `issues/foo.md`, `issues/done/old.md`, a `docs/a.md` citing both, and the
/// configuration `config`.
fn project(config: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for (path, text) in [
        ("issues/foo.md", "# Foo\n"),
        ("issues/done/old.md", "# Old\n"),
        ("docs/a.md", "issues/foo.md issues/old.md\n"),
        (".claude/cardinal.toml", config),
    ] {
        std::fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        std::fs::write(root.join(path), text).unwrap();
    }
    for args in [
        &["init", "-q", "-b", "trunk", "."][..],
        &["add", "-A"],
        &["commit", "-qm", "init", "--no-verify"],
    ] {
        let ok = Command::new("git")
            .args([
                "-c",
                "user.email=t@t",
                "-c",
                "user.name=t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .current_dir(root)
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }
    dir
}

/// The exit code and the standard error of `close-issue <arg>` in `root`.
fn close(root: &Path, arg: &str) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "close-issue", arg])
        .current_dir(root)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// spec: CLS-10
#[test]
fn a_broken_configuration_is_refused_before_any_move() {
    for (config, named) in [
        ("[project\n", "cardinal.toml"),
        ("[project]\nskipped_dirs = \"vendor\"\n", "`skipped_dirs`"),
        ("[project]\nroadmap = [\"ROADMAP.md\", 1]\n", "`roadmap`"),
    ] {
        for (arg, done) in [
            ("issues/foo.md", true),
            ("--relink", true),
            ("--relink", false),
        ] {
            let dir = project(config);
            let root = dir.path();
            if !done {
                std::fs::remove_file(root.join("issues/done/old.md")).unwrap();
            }
            let (code, stderr) = close(root, arg);
            assert_eq!(code, Some(1), "{config:?} {arg}: {stderr}");
            assert!(
                stderr.contains("cardinal.toml") && stderr.contains(named),
                "{config:?} {arg}: {stderr}"
            );
            assert!(root.join("issues/foo.md").is_file(), "{config:?} {arg}");
            assert_eq!(
                std::fs::read_to_string(root.join("docs/a.md")).unwrap(),
                "issues/foo.md issues/old.md\n",
                "{config:?} {arg}"
            );
        }
    }
}

// spec: CLS-10
#[test]
fn a_missing_configuration_reads_as_empty_lists() {
    let dir = project("");
    let root = dir.path();
    std::fs::remove_file(root.join(".claude/cardinal.toml")).unwrap();
    let (code, stderr) = close(root, "issues/foo.md");
    assert_eq!(code, Some(0), "{stderr}");
    assert!(root.join("issues/done/foo.md").is_file());
}
