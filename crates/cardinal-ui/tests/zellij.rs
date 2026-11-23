//! Behavior test for the `zellij` calls of `cardinal-ui` (SUZ): a call that does not answer, against a throwaway
//! repository whose main worktree holds the skill `audit-code` (SUA-24), a fake `cardinal` and a fake `zellij`.

mod common;

use common::{git, ok, project, ui_in, Project, QUEUE, ROADMAP};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn script(path: &Path, body: &str) {
    std::fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A project whose main worktree holds the skill `audit-code`, committed, so `--audit` opens its tab (SUA-24).
fn audited() -> Project {
    let p = project();
    let skill = p.repo.join(".claude/skills/audit-code");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "audit\n").unwrap();
    git(&p.repo, &["add", "."]);
    git(&p.repo, &["commit", "-qm", "skill", "--no-verify"]);
    p
}

/// A fake `zellij` whose call `action <hung>` starts a `sleep 30`, writes its pid to `pid` and waits for it; every
/// other call prints `[]` and exits 0.
fn hanging(p: &Project, hung: &str) -> (PathBuf, PathBuf) {
    let path = p.root.path().join("zellij");
    let pid = p.root.path().join("pid");
    script(
        &path,
        &format!(
            "if [ \"$2\" = {hung} ]; then sleep 30 & echo $! > '{}'; wait; fi\necho '[]'\n",
            pid.display()
        ),
    );
    (path, pid)
}

/// Whether the process whose pid the file `pid` holds still runs.
fn runs(pid: &Path) -> bool {
    let pid = std::fs::read_to_string(pid).unwrap();
    Command::new("kill")
        .args(["-0", pid.trim()])
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap()
        .success()
}

// spec: SUZ-2, SUZ-4
#[test]
fn a_new_tab_that_never_answers_fails_after_5_seconds_and_leaves_no_process() {
    let p = audited();
    let (zellij, pid) = hanging(&p, "new-tab");
    let cardinal = p.root.path().join("cardinal");
    script(&cardinal, "echo \"bad call: $*\" >&2; exit 9\n");
    let start = Instant::now();
    let (code, out, err) = ui_in(&p.ws, &cardinal, Some(&zellij), &["--audit"]);
    let took = start.elapsed();
    assert_eq!(code, Some(1), "{out}{err}");
    assert!(
        err.contains("zellij action new-tab: no answer after 5 s (zellij session renamed?)"),
        "{err}"
    );
    assert!(
        took >= Duration::from_secs(5) && took < Duration::from_secs(15),
        "{took:?}"
    );
    assert!(!runs(&pid), "the sleep of the call still runs");
}

// spec: SUZ-2, SUZ-4
#[test]
fn a_list_tabs_that_never_answers_reads_as_no_tab_after_5_seconds() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let (zellij, pid) = hanging(&p, "list-tabs");
    let start = Instant::now();
    let (code, out, err) = ui_in(&p.repo, &cardinal, Some(&zellij), &["--print"]);
    let took = start.elapsed();
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(!common::line(&out, "  ⠿ a  ").contains("tab"), "{out}");
    assert!(took < Duration::from_secs(15), "{took:?}");
    assert!(!runs(&pid), "the sleep of the call still runs");
}
