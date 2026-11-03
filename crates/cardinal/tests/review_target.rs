//! Behavior test for `cardinal tool review-target`: the target of a code review, without the issue files and the
//! managed copies.

use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
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
        .current_dir(dir)
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok, "git {args:?} failed");
}

fn write(dir: &Path, path: &str, body: &str) {
    let file = dir.join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, body).unwrap();
}

/// A repository with one commit, holding `files`.
fn repo(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main", "."]);
    write(dir.path(), "README.md", "readme\n");
    for (path, body) in files {
        write(dir.path(), path, body);
    }
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-qm", "first", "--no-verify"]);
    dir
}

const LOCK: &str = "# Written by `cardinal sync`. Do not edit.
format 8
aaaa  AGENTS.md
bbbb+x  .claude/rules/a.md
entry  {\"allow\":\"Bash(cardinal run *)\"}
";

/// The exit code, stdout and stderr of the tool run in `dir` with `args`.
fn target(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "review-target"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// spec: RVT-1, RVT-3
#[test]
fn prints_the_range_without_the_issues_and_the_managed_files() {
    let dir = repo(&[(".claude/cardinal.lock", LOCK)]);
    let (code, stdout, _) = target(dir.path(), &["HEAD"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        stdout,
        "HEAD..HEAD -- ':/' ':(top,exclude)issues' ':(top,exclude).claude/rules/a.md' ':(top,exclude)AGENTS.md'\n"
    );
}

// spec: RVT-2
#[test]
fn prints_the_working_tree_against_head_with_no_argument() {
    let dir = repo(&[]);
    let (code, stdout, _) = target(dir.path(), &[]);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "HEAD -- ':/' ':(top,exclude)issues'\n");
}

// spec: RVT-1, RVT-2
#[test]
fn the_line_leaves_the_excluded_paths_out_of_git_diff() {
    let dir = repo(&[(".claude/cardinal.lock", LOCK)]);
    write(dir.path(), "AGENTS.md", "managed\n");
    write(dir.path(), "issues/x.md", "tracking\n");
    write(dir.path(), "src/a.rs", "code\n");
    git(dir.path(), &["add", "-A"]);
    let sub = dir.path().join("src");
    let (_, stdout, _) = target(&sub, &[]);
    // the shell splits the line as the reviewer's `git diff <target>` does, from a subdirectory too
    let out = Command::new("sh")
        .args(["-c", &format!("git diff --name-only {}", stdout.trim_end())])
        .current_dir(&sub)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "src/a.rs\n");
}

// spec: RVT-5
#[test]
fn quotes_a_path_holding_a_quote() {
    let dir = repo(&[(".claude/cardinal.lock", "aaaa  it's.md\n")]);
    let (_, stdout, _) = target(dir.path(), &[]);
    assert_eq!(
        stdout,
        "HEAD -- ':/' ':(top,exclude)issues' ':(top,exclude)it'\\''s.md'\n"
    );
}

// spec: RVT-5
#[test]
fn quotes_a_base_a_shell_would_expand() {
    let dir = repo(&[]);
    git(dir.path(), &["branch", "b"]);
    let (_, stdout, _) = target(dir.path(), &["b@{0}"]);
    assert!(stdout.starts_with("'b@{0}..HEAD' -- "), "{stdout}");
    let (_, stdout, _) = target(dir.path(), &["HEAD~0"]);
    assert!(stdout.starts_with("HEAD~0..HEAD -- "), "{stdout}");
}

// spec: FEA-39
#[test]
fn keeps_the_issues_when_the_issue_files_are_off() {
    let dir = repo(&[
        (".claude/cardinal.lock", LOCK),
        (
            ".claude/cardinal.toml",
            "[cardinal]\ndisabled = [\"issue-files\"]\n",
        ),
    ]);
    let (code, stdout, _) = target(dir.path(), &[]);
    assert_eq!(code, Some(0));
    assert_eq!(
        stdout,
        "HEAD -- ':/' ':(top,exclude).claude/rules/a.md' ':(top,exclude)AGENTS.md'\n"
    );
}

// spec: RVT-4
#[test]
fn refuses_an_option_an_unknown_revision_two_arguments_and_no_repository() {
    let dir = repo(&[]);
    for args in [&["--stat"][..], &["nope"], &["HEAD", "HEAD"]] {
        let (code, stdout, stderr) = target(dir.path(), args);
        assert_eq!(code, Some(3), "{args:?}");
        assert_eq!(stdout, "", "{args:?}");
        assert!(!stderr.is_empty(), "{args:?}");
    }
    let bare = tempfile::tempdir().unwrap();
    let (code, stdout, _) = target(bare.path(), &[]);
    assert_eq!((code, stdout.as_str()), (Some(3), ""));
}

// spec: RVT-4
#[test]
fn refuses_a_lock_it_cannot_read() {
    let dir = repo(&[(".claude/cardinal.lock", "format 999\n")]);
    let (code, stdout, _) = target(dir.path(), &[]);
    assert_eq!((code, stdout.as_str()), (Some(3), ""));
}
