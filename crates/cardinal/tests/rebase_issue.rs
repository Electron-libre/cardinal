//! Behavior test for `cardinal tool rebase-issue`: the rebase of the branch of a `merge pending` issue onto the trunk,
//! in its worktree, against throwaway git repositories.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git_out(dir: &Path, args: &[&str]) -> (bool, String) {
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
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).trim_end().to_string(),
    )
}

fn git(dir: &Path, args: &[&str]) -> String {
    let (ok, out) = git_out(dir, args);
    assert!(ok, "git {args:?} failed");
    out
}

/// A repository `repo` on `trunk` with `ROADMAP.md` and `README.md`, a linked worktree `foo` on branch `foo`, and the
/// claim `merge pending` of the issue `foo` on that branch.
struct Repo {
    _root: tempfile::TempDir,
    main: PathBuf,
    ws: PathBuf,
}

const ROADMAP: &str =
    "# Roadmap\n\n## One\n\n- Issues:\n  - issues/old.md\n\n## Two\n\n- State: idea\n";

impl Repo {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let main = std::fs::canonicalize(root.path()).unwrap().join("repo");
        std::fs::create_dir_all(&main).unwrap();
        std::fs::write(main.join("ROADMAP.md"), ROADMAP).unwrap();
        std::fs::write(main.join("README.md"), "one\ntwo\nthree\n").unwrap();
        git(&main, &["init", "-q", "-b", "trunk", "."]);
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "init", "--no-verify"]);
        let ws = main.with_file_name("repo-foo");
        git(
            &main,
            &["worktree", "add", "-q", "-b", "foo", ws.to_str().unwrap()],
        );
        let claim = format!(
            "session s1\npid 1\nworktree {}\nbranch foo\nstate merge pending\n",
            ws.display()
        );
        let path = main.join("claim.txt");
        std::fs::write(&path, claim).unwrap();
        let oid = git(&main, &["hash-object", "-w", path.to_str().unwrap()]);
        std::fs::remove_file(&path).unwrap();
        git(&main, &["update-ref", "refs/cardinal/claims/foo", &oid]);
        Repo {
            _root: root,
            main,
            ws,
        }
    }

    fn rev(&self, name: &str) -> String {
        git(&self.main, &["rev-parse", name])
    }

    /// Writes the lock of the worktree `foo` (WSL-9).
    fn lock(&self, session: &str, pid: u32) {
        let dir = git(&self.ws, &["rev-parse", "--absolute-git-dir"]);
        std::fs::write(
            Path::new(&dir).join("cardinal-session"),
            format!("session {session}\npid {pid}\n"),
        )
        .unwrap();
    }

    fn rebase_in_progress(&self) -> bool {
        let dir = git(&self.ws, &["rev-parse", "--absolute-git-dir"]);
        Path::new(&dir).join("rebase-merge").exists()
            || Path::new(&dir).join("rebase-apply").exists()
    }
}

/// Writes `text` to `file` in `dir` and commits it there.
fn commit(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", file, "--no-verify"]);
}

/// `cardinal tool rebase-issue <stem>` in `cwd` as the session `session`: (exit code, stdout, stderr).
fn rebase(cwd: &Path, session: &str, stem: &str) -> (Option<i32>, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.args(["tool", "rebase-issue", stem])
        .current_dir(cwd)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        // the config of the developer (signing, conflict style) never reaches the tool
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("CLAUDE_PID")
        .env_remove("CLAUDE_CODE_SESSION_ID");
    if !session.is_empty() {
        cmd.env("CLAUDE_CODE_SESSION_ID", session);
    }
    let out = cmd.output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The pid of a process that ended.
fn dead_pid() -> u32 {
    let mut child = Command::new("true").spawn().unwrap();
    let pid = child.id();
    child.wait().unwrap();
    pid
}

// spec: RBI-H1
// spec: RBI-1
#[test]
fn rebases_a_diverged_branch_in_its_worktree() {
    let r = Repo::new();
    commit(&r.ws, "feature.txt", "f\n");
    commit(&r.main, "other.txt", "o\n");
    let trunk = r.rev("trunk");
    let (code, out, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("rebased foo onto trunk"), "{out}");
    assert_eq!(git(&r.main, &["merge-base", "trunk", "foo"]), trunk);
    assert_eq!(git(&r.ws, &["rev-parse", "HEAD"]), r.rev("foo"));
    assert!(r.ws.join("other.txt").is_file());
    assert_eq!(git(&r.ws, &["status", "--porcelain"]), "");
    assert_eq!(r.rev("trunk"), trunk, "the trunk never moves");
}

// spec: RBI-1
#[test]
fn rewrites_nothing_on_a_branch_that_holds_the_trunk() {
    let r = Repo::new();
    commit(&r.ws, "feature.txt", "f\n");
    let foo = r.rev("foo");
    let (code, out, _) = rebase(&r.ws, "", "foo");
    assert_eq!(code, Some(0));
    assert!(out.contains("foo already holds trunk"), "{out}");
    assert_eq!(r.rev("foo"), foo);
}

// spec: RBI-H2
// spec: RBI-2
#[test]
fn settles_lines_added_to_the_same_list_on_both_sides() {
    let r = Repo::new();
    let with = |extra: &str| {
        ROADMAP.replace(
            "  - issues/old.md\n",
            &format!("  - issues/old.md\n{extra}"),
        )
    };
    commit(&r.ws, "ROADMAP.md", &with("  - issues/b.md\n"));
    commit(&r.main, "ROADMAP.md", &with("  - issues/a.md\n"));
    let (code, out, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("settled ROADMAP.md"), "{out}");
    assert!(out.contains("rebased foo onto trunk"), "{out}");
    let text = std::fs::read_to_string(r.ws.join("ROADMAP.md")).unwrap();
    assert_eq!(text, with("  - issues/a.md\n  - issues/b.md\n"));
    assert_eq!(git(&r.ws, &["status", "--porcelain"]), "");
    assert!(!r.rebase_in_progress());
}

// spec: RBI-3
#[test]
fn aborts_on_a_line_changed_on_both_sides() {
    let r = Repo::new();
    commit(&r.ws, "README.md", "one\nTWO-branch\nthree\n");
    commit(&r.main, "README.md", "one\nTWO-trunk\nthree\n");
    let foo = r.rev("foo");
    let (code, out, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(1), "{out}{err}");
    assert!(
        err.contains("foo: conflict in README.md: rebase aborted, settle it in"),
        "{err}"
    );
    assert!(err.contains(r.ws.to_str().unwrap()), "{err}");
    assert_eq!(r.rev("foo"), foo);
    assert!(!r.rebase_in_progress());
    assert_eq!(git(&r.ws, &["status", "--porcelain"]), "");
}

// spec: RBI-3
#[test]
fn names_only_the_files_it_could_not_settle() {
    let r = Repo::new();
    let with = |extra: &str| {
        ROADMAP.replace(
            "  - issues/old.md\n",
            &format!("  - issues/old.md\n{extra}"),
        )
    };
    std::fs::write(r.ws.join("README.md"), "one\nTWO-branch\nthree\n").unwrap();
    commit(&r.ws, "ROADMAP.md", &with("  - issues/b.md\n"));
    std::fs::write(r.main.join("README.md"), "one\nTWO-trunk\nthree\n").unwrap();
    commit(&r.main, "ROADMAP.md", &with("  - issues/a.md\n"));
    let (code, _, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(1), "{err}");
    assert!(
        err.contains("foo: conflict in README.md: rebase aborted"),
        "{err}"
    );
    assert!(!err.contains("ROADMAP.md"), "{err}");
    assert!(!r.rebase_in_progress());
}

// spec: RBI-3
#[test]
fn aborts_on_a_file_deleted_on_one_side_and_changed_on_the_other() {
    let r = Repo::new();
    commit(&r.ws, "README.md", "one\ntwo\nthree\nfour\n");
    git(&r.main, &["rm", "-q", "README.md"]);
    git(&r.main, &["commit", "-qm", "rm", "--no-verify"]);
    let (code, _, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(1), "{err}");
    assert!(err.contains("conflict in README.md"), "{err}");
    assert!(!r.rebase_in_progress());
}

// spec: RBI-4
#[test]
fn refuses_a_worktree_with_an_untracked_file() {
    let r = Repo::new();
    commit(&r.ws, "feature.txt", "f\n");
    commit(&r.main, "other.txt", "o\n");
    std::fs::write(r.ws.join("stray.txt"), "s").unwrap();
    let foo = r.rev("foo");
    let (code, _, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!(
            "worktree {} has uncommitted changes or untracked files",
            r.ws.display()
        )),
        "{err}"
    );
    assert_eq!(r.rev("foo"), foo);
}

// spec: RBI-4
#[test]
fn refuses_a_worktree_another_running_session_holds() {
    let r = Repo::new();
    commit(&r.ws, "feature.txt", "f\n");
    commit(&r.main, "other.txt", "o\n");
    let foo = r.rev("foo");
    r.lock("live", std::process::id());
    let (code, _, err) = rebase(&r.main, "other", "foo");
    assert_eq!(code, Some(1));
    assert!(
        err.contains(&format!("session live works in {}", r.ws.display())),
        "{err}"
    );
    assert_eq!(r.rev("foo"), foo);
    let (code, out, err) = rebase(&r.ws, "live", "foo");
    assert_eq!(
        code,
        Some(0),
        "the holder rebases its own worktree: {out}{err}"
    );
}

// spec: RBI-4
#[test]
fn rebases_a_worktree_whose_session_ended() {
    let r = Repo::new();
    commit(&r.ws, "feature.txt", "f\n");
    commit(&r.main, "other.txt", "o\n");
    r.lock("gone", dead_pid());
    let (code, out, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(0), "{out}{err}");
}

// spec: RBI-4
#[test]
fn refuses_a_pushed_branch() {
    let r = Repo::new();
    commit(&r.ws, "feature.txt", "f\n");
    commit(&r.main, "other.txt", "o\n");
    let foo = r.rev("foo");
    git(&r.main, &["update-ref", "refs/remotes/origin/foo", &foo]);
    let (code, _, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(1));
    assert!(err.contains("foo was pushed: rebase it by hand"), "{err}");
    assert_eq!(r.rev("foo"), foo);
}

// spec: RBI-4
#[test]
fn refuses_an_issue_that_is_not_merge_pending() {
    let r = Repo::new();
    let (code, _, err) = rebase(&r.main, "", "bar");
    assert_eq!(code, Some(1));
    assert!(err.contains("bar: not merge pending"), "{err}");
}

// spec: RBI-4
#[test]
fn refuses_a_branch_checked_out_nowhere() {
    let r = Repo::new();
    commit(&r.ws, "feature.txt", "f\n");
    commit(&r.main, "other.txt", "o\n");
    git(&r.ws, &["checkout", "-q", "--detach"]);
    let (code, _, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(1));
    assert!(err.contains("foo is checked out in no worktree"), "{err}");
}

// spec: RBI-5
#[test]
fn runs_no_hook_on_the_commits_it_replays() {
    let r = Repo::new();
    let with = |extra: &str| {
        ROADMAP.replace(
            "  - issues/old.md\n",
            &format!("  - issues/old.md\n{extra}"),
        )
    };
    commit(&r.ws, "ROADMAP.md", &with("  - issues/b.md\n"));
    commit(&r.main, "ROADMAP.md", &with("  - issues/a.md\n"));
    let hooks = r.main.join(".git/hooks");
    std::fs::create_dir_all(&hooks).unwrap();
    for hook in ["pre-commit", "commit-msg", "pre-rebase"] {
        let path = hooks.join(hook);
        std::fs::write(&path, "#!/bin/sh\nexit 1\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let (code, out, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("settled ROADMAP.md"), "{out}");
}

// spec: RBI-2
#[test]
fn keeps_once_the_lines_both_sides_added_alike() {
    let r = Repo::new();
    let with = |extra: &str| format!("{ROADMAP}{extra}");
    commit(&r.ws, "ROADMAP.md", &with("\n## Three\n\n- issues/b.md\n"));
    commit(
        &r.main,
        "ROADMAP.md",
        &with("\n## Three\n\n- issues/a.md\n"),
    );
    let (code, out, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    let text = std::fs::read_to_string(r.ws.join("ROADMAP.md")).unwrap();
    assert_eq!(text, with("\n## Three\n\n- issues/a.md\n- issues/b.md\n"));
}

// spec: RBI-2
#[test]
fn settles_a_file_whose_name_holds_glob_characters() {
    let r = Repo::new();
    commit(&r.main, "[d].md", "- one\n");
    git(&r.ws, &["rebase", "-q", "trunk"]);
    commit(&r.ws, "[d].md", "- one\n- branch\n");
    commit(&r.main, "[d].md", "- one\n- trunk\n");
    let (code, out, err) = rebase(&r.main, "", "foo");
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("settled [d].md"), "{out}");
    let text = std::fs::read_to_string(r.ws.join("[d].md")).unwrap();
    assert_eq!(text, "- one\n- trunk\n- branch\n");
}
