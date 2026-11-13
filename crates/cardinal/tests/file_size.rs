//! Behavior test for `cardinal tool file-size`: the files a change pushes over the file size limit of
//! `issue-workflow`, or grows while over it, and with `--all` every file over it.

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

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", message, "--no-verify"]);
}

/// A repository whose first commit holds `files`.
fn repo(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main", "."]);
    write(dir.path(), "README.md", "readme\n");
    for (path, body) in files {
        write(dir.path(), path, body);
    }
    commit(dir.path(), "first");
    dir
}

/// The exit code and stdout of the tool run in `dir` with `args`.
fn size(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "file-size"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

fn lines(n: usize) -> String {
    (1..=n)
        .flat_map(|i| ["line ".to_string(), i.to_string(), "\n".into()])
        .collect()
}

const NONE: &str = "0 files over the limit of 800 lines\n";

// spec: FSZ-1, FSZ-7, FSZ-H1
#[test]
fn lists_a_file_the_change_grew_over_the_limit() {
    let dir = repo(&[("src/a.rs", &lines(790))]);
    write(dir.path(), "src/a.rs", &lines(830));
    assert_eq!(
        size(dir.path(), &[]),
        (
            Some(1),
            "830\tsrc/a.rs (was 790)\n1 files over the limit of 800 lines\n".into()
        )
    );
}

// spec: FSZ-1
#[test]
fn a_file_over_the_limit_that_did_not_grow_is_not_listed() {
    let dir = repo(&[("big.rs", &lines(900)), ("other.rs", &lines(900))]);
    write(dir.path(), "big.rs", &lines(850));
    write(dir.path(), "small.rs", &lines(10));
    assert_eq!(size(dir.path(), &[]), (Some(0), NONE.into()));
}

// spec: FSZ-1
#[test]
fn a_new_file_is_listed_with_no_former_count() {
    let dir = repo(&[]);
    write(dir.path(), "b.rs", &lines(801));
    write(dir.path(), "c.rs", &lines(800));
    let (code, stdout) = size(dir.path(), &[]);
    assert_eq!(code, Some(1));
    assert_eq!(stdout, "801\tb.rs\n1 files over the limit of 800 lines\n");
}

// spec: FSZ-1
#[test]
fn a_repository_with_no_commit_compares_with_the_empty_tree() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main", "."]);
    write(dir.path(), "a.rs", &lines(801));
    assert_eq!(
        size(dir.path(), &[]).1,
        "801\ta.rs\n1 files over the limit of 800 lines\n"
    );
}

// spec: FSZ-1
#[test]
fn a_move_with_no_growth_is_not_listed() {
    let dir = repo(&[("old.rs", &lines(900))]);
    std::fs::rename(dir.path().join("old.rs"), dir.path().join("new.rs")).unwrap();
    assert_eq!(size(dir.path(), &[]), (Some(0), NONE.into()));
}

// spec: FSZ-2
#[test]
fn a_revision_adds_the_working_tree_a_range_does_not() {
    let dir = repo(&[("a.rs", &lines(790))]);
    write(dir.path(), "a.rs", &lines(810));
    commit(dir.path(), "grow");
    write(dir.path(), "a.rs", &lines(820));
    assert_eq!(
        size(dir.path(), &["HEAD~1..HEAD"]).1,
        "810\ta.rs (was 790)\n1 files over the limit of 800 lines\n"
    );
    assert_eq!(
        size(dir.path(), &["HEAD~1"]).1,
        "820\ta.rs (was 790)\n1 files over the limit of 800 lines\n"
    );
    for range in ["HEAD~1..", "HEAD~1...", "HEAD~1...HEAD"] {
        assert_eq!(
            size(dir.path(), &[range]).1,
            "810\ta.rs (was 790)\n1 files over the limit of 800 lines\n",
            "{range}"
        );
    }
}

// spec: FSZ-3, FSZ-7, FSZ-H2
#[test]
fn all_lists_every_file_over_the_limit_largest_first() {
    let dir = repo(&[
        ("a.rs", &lines(900)),
        ("b.rs", &lines(1200)),
        ("c.rs", &lines(10)),
    ]);
    write(dir.path(), "d.rs", &lines(900));
    assert_eq!(
        size(dir.path(), &["--all"]),
        (
            Some(1),
            "1200\tb.rs\n900\ta.rs\n900\td.rs\n3 files over the limit of 800 lines\n".into()
        )
    );
}

// spec: FSZ-4
#[test]
fn a_last_line_with_no_newline_counts() {
    let dir = repo(&[]);
    write(dir.path(), "a.rs", &format!("{}last", lines(800)));
    assert_eq!(
        size(dir.path(), &["--all"]).1,
        "801\ta.rs\n1 files over the limit of 800 lines\n"
    );
}

// spec: FSZ-5
#[test]
fn leaves_out_the_issues_the_lock_files_the_binaries_and_the_deleted_files() {
    let dir = repo(&[
        ("gone.rs", &lines(900)),
        (".gitattributes", "*.gen -diff\n"),
    ]);
    std::fs::remove_file(dir.path().join("gone.rs")).unwrap();
    write(dir.path(), "issues/a.md", &lines(900));
    write(dir.path(), "Cargo.lock", &lines(900));
    write(dir.path(), "data.bin", &format!("{}\0", lines(900)));
    write(dir.path(), "out.gen", &lines(900));
    assert_eq!(size(dir.path(), &[]), (Some(0), NONE.into()));
    assert_eq!(size(dir.path(), &["--all"]), (Some(0), NONE.into()));
}

// spec: FSZ-5, FEA-37
#[test]
fn lists_the_issues_with_the_issue_files_off() {
    let dir = repo(&[(
        ".claude/cardinal.toml",
        "[cardinal]\ndisabled = [\"issue-files\"]\n",
    )]);
    write(dir.path(), "issues/a.md", &lines(801));
    assert_eq!(
        size(dir.path(), &[]).1,
        "801\tissues/a.md\n1 files over the limit of 800 lines\n"
    );
}

// spec: FSZ-6
#[test]
fn the_limit_comes_from_the_configuration_and_0_turns_it_off() {
    let dir = repo(&[
        (".claude/cardinal.toml", "[project]\nfile_size_limit = 10\n"),
        ("a.rs", &lines(5)),
    ]);
    write(dir.path(), "a.rs", &lines(11));
    assert_eq!(
        size(dir.path(), &[]),
        (
            Some(1),
            "11\ta.rs (was 5)\n1 files over the limit of 10 lines\n".into()
        )
    );
    let dir = repo(&[(".claude/cardinal.toml", "[project]\nfile_size_limit = 0\n")]);
    write(dir.path(), "a.rs", &lines(2000));
    let off = (Some(0), "0 files over the limit of 0 lines\n".to_string());
    assert_eq!(size(dir.path(), &[]), off);
    assert_eq!(size(dir.path(), &["--all"]), off);
}

// spec: FSZ-8
#[test]
fn a_git_failure_a_bad_argument_or_a_bad_limit_exits_3_listing_nothing() {
    let dir = repo(&[(
        ".claude/cardinal.toml",
        "[project]\nfile_size_limit = \"big\"\n",
    )]);
    assert_eq!(size(dir.path(), &[]), (Some(3), String::new()));
    let dir = repo(&[(".claude/cardinal.toml", "[project]\nfile_size_limit = -1\n")]);
    assert_eq!(size(dir.path(), &[]), (Some(3), String::new()));
    let dir = repo(&[]);
    assert_eq!(size(dir.path(), &["nope"]), (Some(3), String::new()));
    assert_eq!(size(dir.path(), &["--output=x"]), (Some(3), String::new()));
    assert!(!dir.path().join("x").exists());
    assert_eq!(
        size(dir.path(), &["HEAD", "--all"]),
        (Some(3), String::new())
    );
    for target in ["HEAD..--output=x", "--output=x...HEAD", "HEAD^!", "HEAD^@"] {
        assert_eq!(
            size(dir.path(), &[target]),
            (Some(3), String::new()),
            "{target}"
        );
    }
    assert!(!dir.path().join("x").exists());
    let outside = tempfile::tempdir().unwrap();
    assert_eq!(size(outside.path(), &[]), (Some(3), String::new()));
}

// spec: FSZ-1
#[cfg(unix)]
#[test]
fn a_path_that_is_not_utf8_is_listed() {
    use std::os::unix::ffi::OsStrExt;
    let dir = repo(&[]);
    let name = std::ffi::OsStr::from_bytes(b"caf\xe9.rs");
    std::fs::write(dir.path().join(name), lines(700)).unwrap();
    commit(dir.path(), "latin-1");
    std::fs::write(dir.path().join(name), lines(900)).unwrap();
    let (code, stdout) = size(dir.path(), &[]);
    assert_eq!(code, Some(1), "{stdout}");
    assert!(
        stdout.ends_with(" (was 700)\n1 files over the limit of 800 lines\n"),
        "{stdout}"
    );
}

fn status(dir: &Path) -> String {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

// spec: FSZ-9
#[test]
fn leaves_the_index_alone() {
    let dir = repo(&[]);
    write(dir.path(), "b.rs", &lines(801));
    let before = status(dir.path());
    assert_eq!(size(dir.path(), &[]).0, Some(1));
    assert_eq!(size(dir.path(), &["--all"]).0, Some(1));
    assert_eq!(status(dir.path()), before);
}
