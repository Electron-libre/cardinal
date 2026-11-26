//! Behavior test for the close of an issue under `cardinal tool review-gate native` (REV-20, FEA-36): the commit of
//! a close needs no review, anything else in it does, and with the issue files off it waits for one like any change.

mod review_gate_common;

use review_gate_common::*;
use std::fs;
use std::path::Path;
use std::process::Command;

/// A repository with `issues/foo.md` cited from the roadmap (the last open issue of its milestone `Last`), a doc and
/// another issue, closed by `close-issue` and
/// staged, then changed by `edit` (which stages what the commit records); whether `native` lets the commit through without any review.
fn close_passes(hook: &Path, edit: impl Fn(&Path)) -> bool {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().to_path_buf();
    for dir in ["issues/done", "docs", "script", ".claude"] {
        fs::create_dir_all(project.join(dir)).unwrap();
    }
    sh(&project, &["init", "-q", "."]);
    fs::write(
        project.join("issues/foo.md"),
        "---\npriority: high\n---\n# Foo\n",
    )
    .unwrap();
    fs::write(
        project.join("issues/bar.md"),
        "---\npriority: low\n---\n# Bar\nAfter [foo](foo.md).\n",
    )
    .unwrap();
    fs::write(project.join("issues/done/old.md"), "# Old\n").unwrap();
    fs::write(
        project.join(".claude/cardinal.toml"),
        "[project]\nroadmap = [\"ROADMAP.md\"]\n",
    )
    .unwrap();
    // close-issue sets `Last` done (CLS-8); `Open` still lists `bar`, `Other` does not list `foo`
    fs::write(
        project.join("ROADMAP.md"),
        "# Roadmap\n\n## Open\n- State: in issues\n- Issues:\n  - [Foo](issues/foo.md)\n  - [Bar](issues/bar.md)\n\n\
## Last\n- State: in issues\n- Issues:\n  - [Foo](issues/foo.md)\n  - [Old](issues/done/old.md)\n\n\
## Other\n- State: in issues\n- Issues:\n  - [Old](issues/done/old.md)\n",
    )
    .unwrap();
    fs::write(
        project.join("docs/a.md"),
        "See [foo](../issues/foo.md#goal).\n",
    )
    .unwrap();
    // close-issue never rewrites a shell script, nor a submodule
    fs::write(project.join("script/x.sh"), "cat issues/foo.md\n").unwrap();
    let sub = project.join("sub");
    fs::create_dir(&sub).unwrap();
    sh(&sub, &["init", "-q", "."]);
    sh(&sub, &["commit", "-q", "--allow-empty", "-m", "one"]);
    sh(&project, &["add", "-A"]);
    sh(&project, &["commit", "-qm", "init"]);
    let closed = Command::new(hook)
        .args(["tool", "close-issue", "issues/foo.md"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(closed.status.success(), "close-issue failed");
    sh(&project, &["add", "-A"]);
    edit(&project);
    native_run(hook, &project, &[]).0 == 0
}

/// Replaces `from` by `to` in the file `name` of `project`, and stages it when `staged`.
fn replace_in(project: &Path, name: &str, from: &str, to: &str, staged: bool) {
    let path = project.join(name);
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains(from), "{name} lacks {from}");
    fs::write(path, text.replacen(from, to, 1)).unwrap();
    if staged {
        sh(project, &["add", "--", name]);
    }
}

// spec: REV-20
/// The commit of a close holds, outside `issues/`, the rewrites `close-issue` made: it needs no review, and anything
/// else in it does.
fn close_checks(hook: &Path) -> Vec<(&'static str, bool)> {
    vec![
        (
            "native lets through the close of an issue with the references and the milestone close-issue rewrote",
            close_passes(hook, |_| {}),
        ),
        (
            "native lets through a close whose milestone state close-issue set was put back",
            close_passes(hook, |p| {
                let last = "## Last\n- State: done";
                replace_in(p, "ROADMAP.md", last, "## Last\n- State: in issues", true);
            }),
        ),
        (
            "native refuses a close that also sets done a milestone with an issue still open",
            !close_passes(hook, |p| {
                let open = "## Open\n- State: in issues";
                replace_in(p, "ROADMAP.md", open, "## Open\n- State: done", true);
            }),
        ),
        (
            "native refuses a close that also sets done a milestone that does not list the closed issue",
            !close_passes(hook, |p| {
                let other = "## Other\n- State: in issues";
                replace_in(p, "ROADMAP.md", other, "## Other\n- State: done", true);
            }),
        ),
        (
            "native refuses a milestone set done when the index lacks one of its issues",
            !close_passes(hook, |p| sh(p, &["rm", "-q", "--", "issues/done/old.md"])),
        ),
        (
            "native refuses a close with an edited word on a rewritten line",
            !close_passes(hook, |p| replace_in(p, "docs/a.md", "See", "Read", true)),
        ),
        (
            "native refuses a close with a file added outside issues/",
            !close_passes(hook, |p| {
                fs::write(p.join("new.md"), "x\n").unwrap();
                sh(p, &["add", "--", "new.md"]);
            }),
        ),
        (
            "native refuses a close with a file deleted outside issues/",
            !close_passes(hook, |p| sh(p, &["rm", "-qf", "--", "docs/a.md"])),
        ),
        (
            "native refuses a close with a file whose mode changed",
            !close_passes(hook, |p| sh(p, &["update-index", "--chmod=+x", "docs/a.md"])),
        ),
        (
            "native refuses a close with a submodule moved",
            !close_passes(hook, |p| {
                sh(&p.join("sub"), &["commit", "-q", "--allow-empty", "-m", "two"]);
                sh(p, &["add", "sub"]);
            }),
        ),
        (
            "native refuses a close with a reference rewritten in a file close-issue never rewrites",
            !close_passes(hook, |p| {
                replace_in(p, "script/x.sh", "issues/foo.md", "issues/done/foo.md", true);
            }),
        ),
        (
            "native refuses rewritten references when the move of the issue is not staged",
            !close_passes(hook, |p| {
                sh(p, &["mv", "issues/done/foo.md", "issues/foo.md"]);
            }),
        ),
        (
            "native lets through a close while an unstaged change waits in the work tree",
            close_passes(hook, |p| {
                replace_in(p, "docs/a.md", "See", "Read", false);
                fs::write(p.join("new.md"), "x\n").unwrap();
            }),
        ),
    ]
}

#[test]
fn review_gate_close() {
    let hook = Path::new(env!("CARGO_BIN_EXE_cardinal"));
    assert_checks(close_checks(hook));
}

// spec: FEA-36
/// With the issue files off, `issues/` is a directory like any other: a commit of it, or of a close, waits for a review.
#[test]
fn issues_are_code_with_the_issue_files_off() {
    let hook = Path::new(env!("CARGO_BIN_EXE_cardinal")).to_path_buf();
    let off = "[cardinal]\ndisabled = [\"issue-files\"]\n";
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    fs::create_dir_all(project.join("issues")).unwrap();
    fs::create_dir_all(project.join(".claude")).unwrap();
    sh(project, &["init", "-q", "."]);
    fs::write(project.join(".claude/cardinal.toml"), off).unwrap();
    fs::write(project.join("issues/i.md"), "one\n").unwrap();
    fs::write(project.join("code.txt"), "one\n").unwrap();
    sh(project, &["add", "-A"]);
    sh(project, &["commit", "-qm", "init"]);
    fs::write(project.join("code.txt"), "two\n").unwrap();
    review_done(&hook, project, "code-review");
    assert_eq!(
        native_run(&hook, project, &[]).0,
        0,
        "the reviewed diff passes"
    );
    fs::write(project.join("issues/x.md"), "new\n").unwrap();
    let (code, stderr) = native_run(&hook, project, &[]);
    assert!(
        code == 2 && stderr.contains("never saw"),
        "a new file under issues/ is one the review never saw: {stderr}"
    );
    fs::remove_file(project.join("issues/x.md")).unwrap();
    fs::write(project.join("code.txt"), "one\n").unwrap();
    fs::write(project.join("issues/i.md"), "two\n").unwrap();
    assert_eq!(
        native_run(&hook, project, &[]).0,
        2,
        "a diff under issues/ only waits for a review"
    );
    fs::write(project.join("code.txt"), "two\n").unwrap();
    sh(project, &["add", "--", "issues/i.md"]);
    assert_eq!(
        native_run(&hook, project, &[]).0,
        2,
        "an index under issues/ only waits for a review"
    );
    assert!(
        !close_passes(&hook, |p| {
            let config = fs::read_to_string(p.join(".claude/cardinal.toml")).unwrap();
            fs::write(p.join(".claude/cardinal.toml"), format!("{config}{off}")).unwrap();
        }),
        "the close of an issue waits for a review"
    );
}
