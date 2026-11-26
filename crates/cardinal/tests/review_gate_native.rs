//! Behavior test for `cardinal tool review-gate native` (REV-9, REV-10, REV-11): the repository git resolved, the
//! waiver, the hook installed as `pre-commit` and the linked worktrees.

mod review_gate_common;

use review_gate_common::*;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

/// The waiver under the native hook: honoured, unreadable, not UTF-8 or read-only.
fn native_waiver_checks(hook: &Path, current: &Path) -> Vec<(&'static str, bool)> {
    let mut out = Vec::new();
    let waiver = current
        .join(".git")
        .join(["claude-review", "waived"].join("-"));
    fs::write(&waiver, "").unwrap();
    out.push((
        "native honours the waiver",
        native_run(hook, current, &[]).0 == 0,
    ));
    fs::remove_file(&waiver).unwrap();
    // a waiver that cannot be read is ignored too: it is neither stamped nor dropped, and the user is told to fix it
    fs::write(&waiver, "").unwrap();
    fs::set_permissions(&waiver, std::os::unix::fs::PermissionsExt::from_mode(0o200)).unwrap();
    // root reads a write-only file: the case cannot be built then
    if fs::File::open(&waiver).is_err() {
        let (code, stderr) = native_run(hook, current, &[]);
        out.push((
            "native refuses an unreviewed diff whose waiver it cannot read",
            code == 2 && stderr.contains("/code-review low"),
        ));
        let path = waiver.canonicalize().unwrap();
        out.push(("that refusal tells the agent to ask the user to fix the unreadable waiver, by its path", stderr.contains("could not be read") && stderr.contains("ask the user to fix") && stderr.contains(path.to_str().unwrap())));
        out.push((
            "native leaves the unreadable waiver unstamped",
            fs::metadata(&waiver).is_ok_and(|m| m.len() == 0),
        ));
        review_done(hook, current, "code-review");
        out.push((
            "native lets through a reviewed diff whose waiver it cannot read",
            native_run(hook, current, &[]).0 == 0,
        ));
        // a file the review never saw, so the next cases judge an unreviewed diff again
        fs::write(current.join("unseen.txt"), "x\n").unwrap();
    }
    fs::set_permissions(&waiver, std::os::unix::fs::PermissionsExt::from_mode(0o644)).unwrap();
    fs::remove_file(&waiver).unwrap();
    // content that is not UTF-8 is read, not unreadable: it is no stamp, so it is dropped like a stale one
    fs::write(&waiver, [0xff, 0xfe]).unwrap();
    let (code, stderr) = native_run(hook, current, &[]);
    out.push((
        "native drops a waiver whose content is not UTF-8 and is no stamp",
        code == 2 && !stderr.contains("could not be read") && !waiver.exists(),
    ));
    // a waiver that cannot be stamped is ignored: the review alone judges the commit, and the user is told to fix it
    fs::write(&waiver, "").unwrap();
    let mut readonly = fs::metadata(&waiver).unwrap().permissions();
    readonly.set_readonly(true);
    fs::set_permissions(&waiver, readonly.clone()).unwrap();
    // root writes a read-only file: the case cannot be built then
    if fs::OpenOptions::new().write(true).open(&waiver).is_err() {
        let (code, stderr) = native_run(hook, current, &[]);
        out.push((
            "native refuses an unreviewed diff whose waiver it cannot stamp",
            code == 2 && stderr.contains("/code-review low"),
        ));
        let path = waiver.canonicalize().unwrap();
        out.push((
            "that refusal tells the agent to ask the user to fix the waiver, by its path",
            stderr.contains("ask the user to fix") && stderr.contains(path.to_str().unwrap()),
        ));
        review_done(hook, current, "code-review");
        out.push((
            "native lets through a reviewed diff whose waiver it cannot stamp",
            native_run(hook, current, &[]).0 == 0,
        ));
    }
    #[allow(clippy::permissions_set_readonly_false)]
    readonly.set_readonly(false);
    fs::set_permissions(&waiver, readonly).unwrap();
    fs::remove_file(&waiver).unwrap();
    out
}

// spec: REV-9, REV-10
fn native_checks(hook: &Path) -> Vec<(&'static str, bool)> {
    let (current, other) = target_fixture(hook, "rev9");
    let (code, stderr) = native_run(hook, &other, &[]);
    let mut out = vec![
        ("native refuses an unreviewed repository with a message naming the review and its level", code != 0 && stderr.contains("/code-review low")),
        ("native lets through a reviewed repository", native_run(hook, &current, &[]).0 == 0),
    ];
    // git resolved the repository: the text of the command is not involved
    let (git_dir, tree) = (other.join(".git"), other.clone());
    out.push((
        "native judges the repository named by GIT_DIR and GIT_WORK_TREE",
        native_run(
            hook,
            &current,
            &[("GIT_DIR", &git_dir), ("GIT_WORK_TREE", &tree)],
        )
        .0 != 0,
    ));
    review_done(hook, &other, "code-review");
    fs::write(current.join("new.txt"), "x\n").unwrap();
    out.push((
        "native lets through a reviewed GIT_DIR while the current directory is unreviewed",
        native_run(
            hook,
            &current,
            &[("GIT_DIR", &git_dir), ("GIT_WORK_TREE", &tree)],
        )
        .0 == 0,
    ));
    out.push((
        "native refuses the unreviewed current directory",
        native_run(hook, &current, &[]).0 != 0,
    ));
    out.extend(native_waiver_checks(hook, &current));
    fs::remove_file(current.join("new.txt")).unwrap();
    let _ = fs::remove_file(current.join("unseen.txt"));
    fs::write(current.join("a.txt"), "one\n").unwrap();
    out.push((
        "native lets through a repository with nothing to commit",
        native_run(hook, &current, &[]).0 == 0,
    ));
    let _ = fs::remove_dir_all(current.parent().unwrap());
    out
}

// spec: REV-9, REV-10, REV-H1
/// The hook as git runs it: installed as `pre-commit`, it refuses a real commit nobody reviewed and lets the reviewed one through.
fn native_hook_checks(hook: &Path) -> Vec<(&'static str, bool)> {
    use std::os::unix::fs::PermissionsExt;
    let (current, _other) = target_fixture(hook, "rev9git");
    let pre_commit = current.join(".git/hooks/pre-commit");
    fs::write(
        &pre_commit,
        format!(
            "#!/bin/sh\nexec '{}' tool review-gate native\n",
            hook.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&pre_commit, fs::Permissions::from_mode(0o755)).unwrap();
    let commit = |args: &[&str]| {
        Command::new("git")
            .args(["-c", "user.email=t@t", "-c", "user.name=t"])
            .args(args)
            .current_dir(&current)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    };
    let commits = || {
        let log = Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .current_dir(&current)
            .output()
            .unwrap();
        String::from_utf8_lossy(&log.stdout)
            .trim()
            .parse::<u32>()
            .unwrap()
    };
    let before = commits();
    let reviewed = commit(&["commit", "-qam", "x"]) && commits() == before + 1;
    fs::write(current.join("a.txt"), "three\n").unwrap();
    let refusal = Command::new("git")
        .args([
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qam",
            "y",
        ])
        .current_dir(&current)
        .output()
        .unwrap();
    let refused = !refusal.status.success()
        && String::from_utf8_lossy(&refusal.stderr).contains("Run /code-review low");
    // the commit records the issues only: the unreviewed change of `a.txt` stays out of it
    fs::create_dir_all(current.join("issues")).unwrap();
    fs::write(current.join("issues/i.md"), "one\n").unwrap();
    fs::write(current.join("issues/j.md"), "one\n").unwrap();
    let staged_issue = commit(&["add", "issues/i.md"])
        && commit(&["commit", "-qm", "issue"])
        && commits() == before + 2;
    let only_issue = commit(&["add", "-N", "issues/j.md"])
        && commit(&["commit", "-qm", "issue", "--", "issues/j.md"])
        && commits() == before + 3;
    let issue_with_code = commit(&["add", "issues/i.md", "a.txt"]) && {
        fs::write(current.join("issues/i.md"), "two\n").unwrap();
        commit(&["add", "issues/i.md"]) && !commit(&["commit", "-qm", "both"])
    };
    let out = vec![
        (
            "git runs the native hook and a reviewed commit -a passes",
            reviewed,
        ),
        (
            "git refuses through the native hook a commit nobody reviewed, telling to run /code-review low",
            refused,
        ),
        (
            "git lets through a commit of a staged issue file while an unreviewed file stays modified",
            staged_issue,
        ),
        (
            "git lets through `commit -- issues/<file>` while an unreviewed file stays modified",
            only_issue,
        ),
        (
            "git refuses a commit whose index holds an unreviewed file next to the issue",
            issue_with_code,
        ),
        (
            "the refused commits left the history alone",
            commits() == before + 3,
        ),
    ];
    let _ = fs::remove_dir_all(current.parent().unwrap());
    out
}

// spec: REV-11
fn native_worktree_checks(hook: &Path) -> Vec<(&'static str, bool)> {
    let root = tempfile::tempdir().unwrap().keep();
    let _ = fs::remove_dir_all(&root);
    let (main, linked) = (root.join("main"), root.join("linked"));
    fs::create_dir_all(&main).unwrap();
    sh(&main, &["init", "-q", "."]);
    fs::write(main.join("a.txt"), "one\n").unwrap();
    sh(&main, &["add", "-A"]);
    sh(&main, &["commit", "-qm", "init"]);
    sh(
        &main,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "other",
            linked.to_str().unwrap(),
        ],
    );
    fs::write(main.join("a.txt"), "two\n").unwrap();
    fs::write(linked.join("a.txt"), "two\n").unwrap();
    review_done(hook, &linked, "code-review");
    let linked_git = git_dir_of(&linked);
    let out = vec![
        (
            "native lets through a reviewed linked worktree",
            native_run(hook, &linked, &[("GIT_DIR", &linked_git)]).0 == 0,
        ),
        (
            "native refuses the unreviewed main worktree",
            native_run(hook, &main, &[]).0 != 0,
        ),
    ];
    let _ = fs::remove_dir_all(&root);
    out
}

#[test]
fn review_gate_native() {
    let hook = Path::new(env!("CARGO_BIN_EXE_cardinal"));
    assert_checks(
        [native_checks, native_hook_checks, native_worktree_checks]
            .into_iter()
            .flat_map(|checks| checks(hook)),
    );
}
