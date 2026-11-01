//! Behavior test for `cardinal tool issue-worktree` (`start` and `finish`) against a throwaway git repository.

use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> bool {
    Command::new("git")
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
        .success()
}

fn git_ok(dir: &Path, args: &[&str]) {
    assert!(git(dir, args), "git {args:?} failed");
}

fn branch_exists(repo: &Path, name: &str) -> bool {
    git(
        repo,
        &["rev-parse", "--verify", "-q", &format!("refs/heads/{name}")],
    )
}

/// The `info/exclude` the user had before `start`: a Latin-1 comment, and no final newline.
const EXCLUDE: &[u8] = b"# caf\xe9\n*.log";

struct Ctx {
    root: PathBuf,
    repo: PathBuf,
    script: PathBuf,
}

type Checks = Vec<(&'static str, bool)>;

impl Ctx {
    fn run(&self, cwd: &Path, args: &[&str]) -> (bool, String) {
        if !cwd.is_dir() {
            return (false, format!("{} does not exist", cwd.display()));
        }
        let out = tool_command(&self.script)
            .args(args)
            .current_dir(cwd)
            .output()
            .unwrap();
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned()
                + &String::from_utf8_lossy(&out.stderr),
        )
    }

    fn wt(&self, stem: &str) -> PathBuf {
        self.repo.join(".claude/worktrees").join(stem)
    }

    /// Where `start` put the worktrees before PAR-1 nested them.
    fn sibling(&self, stem: &str) -> PathBuf {
        self.root.join(format!("repo-{stem}"))
    }

    fn entries(&self) -> usize {
        std::fs::read_dir(&self.root).unwrap().count()
            + std::fs::read_dir(self.repo.join(".claude/worktrees")).map_or(0, Iterator::count)
    }
}

// spec: PAR-1
fn start_creates_the_worktree(c: &Ctx) -> Checks {
    let (ok, out) = c.run(&c.repo, &["start", "issues/x.md"]);
    let created = ok && c.wt("x").join("issues/x.md").is_file() && branch_exists(&c.repo, "x");
    let (linked_ok, _) = c.run(&c.wt("x"), &["start", "issues/y.md"]);
    vec![
        (
            "start creates the worktree under .claude/worktrees on a branch named after the stem",
            created,
        ),
        (
            "start prints the worktree path",
            out.trim() == c.wt("x").to_str().unwrap(),
        ),
        (
            "start works from a linked worktree and nests the new one in the main one",
            linked_ok && c.wt("y").is_dir() && !c.wt("x").join(".claude/worktrees/y").exists(),
        ),
    ]
}

// spec: PAR-13
fn start_makes_git_ignore_the_worktrees(c: &Ctx) -> Checks {
    let bytes = std::fs::read(c.repo.join(".git/info/exclude")).unwrap_or_default();
    let exclude = String::from_utf8_lossy(&bytes);
    let status = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(&c.repo)
        .output()
        .unwrap();
    let status = String::from_utf8_lossy(&status.stdout).into_owned();
    vec![
        (
            "the main worktree's status lists no issue worktree despite `!.claude/`",
            !status.contains(".claude/worktrees"),
        ),
        (
            "info/exclude holds the line once after two starts",
            exclude
                .lines()
                .filter(|l| *l == "/.claude/worktrees/")
                .count()
                == 1,
        ),
        (
            "info/exclude keeps its earlier lines, even not in UTF-8",
            bytes.starts_with(EXCLUDE),
        ),
    ]
}

const PERSONAL: &str = ".claude/cardinal.local.toml";
const PERSONAL_TEXT: &str = "[project]\nuser_language = \"English\"\n";

// spec: PAR-17
fn start_copies_the_personal_configuration(c: &Ctx) -> Checks {
    let copied = std::fs::read_to_string(c.wt("x").join(PERSONAL)).ok();
    vec![(
        "start copies the personal configuration of the main worktree into the new one",
        copied.as_deref() == Some(PERSONAL_TEXT),
    )]
}

// spec: PAR-2
fn start_refuses_and_changes_nothing(c: &Ctx) -> Checks {
    let before = c.entries();
    let (taken, _) = c.run(&c.repo, &["start", "issues/x.md"]);
    let (missing, _) = c.run(&c.repo, &["start", "issues/missing.md"]);
    let (uncommitted, _) = c.run(&c.repo, &["start", "issues/new.md"]);
    std::fs::create_dir_all(c.wt("z")).unwrap();
    let (occupied, _) = c.run(&c.repo, &["start", "issues/z.md"]);
    std::fs::remove_dir_all(c.wt("z")).unwrap();
    git_ok(&c.repo, &["checkout", "-q", "--detach"]);
    let (detached, _) = c.run(&c.repo, &["start", "issues/z.md"]);
    let detached_clean = !branch_exists(&c.repo, "z") && !c.wt("z").exists();
    git_ok(&c.repo, &["checkout", "-q", "trunk"]);
    vec![
        (
            "start refuses a detached main worktree",
            !detached && detached_clean,
        ),
        ("start refuses an issue whose branch exists", !taken),
        (
            "start refuses a missing issue file",
            !missing && !branch_exists(&c.repo, "missing"),
        ),
        (
            "start refuses an issue file the main branch does not hold",
            !uncommitted && !branch_exists(&c.repo, "new"),
        ),
        (
            "start refuses an existing directory",
            !occupied && !branch_exists(&c.repo, "z"),
        ),
        ("a refusal changes nothing", c.entries() == before),
    ]
}

// spec: PAR-6
fn finish_needs_a_merged_branch(c: &Ctx) -> Checks {
    std::fs::write(c.wt("x").join("work.txt"), "work\n").unwrap();
    git_ok(&c.wt("x"), &["add", "-A"]);
    git_ok(&c.wt("x"), &["commit", "-qm", "work", "--no-verify"]);
    let (unmerged, _) = c.run(&c.repo, &["finish", "x"]);
    let kept = c.wt("x").is_dir() && branch_exists(&c.repo, "x");
    git_ok(&c.repo, &["merge", "-q", "--ff-only", "x"]);
    let (merged, _) = c.run(&c.repo, &["finish", "x"]);
    vec![
        (
            "finish refuses an unmerged branch and keeps both",
            !unmerged && kept,
        ),
        (
            "finish removes the worktree and the branch once merged",
            merged && !c.wt("x").exists() && !branch_exists(&c.repo, "x"),
        ),
    ]
}

// spec: PAR-7
fn finish_removes_only_a_clean_worktree(c: &Ctx) -> Checks {
    std::fs::write(c.wt("y").join("dirty.txt"), "wip\n").unwrap();
    let (dirty, _) = c.run(&c.repo, &["finish", "y"]);
    let elsewhere = c.root.join("elsewhere");
    git_ok(
        &c.repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "w",
            elsewhere.to_str().unwrap(),
        ],
    );
    let (stranger, _) = c.run(&c.repo, &["finish", "w"]);
    // an agent worktree nested in the issue worktree is ignored there, so `git status` alone does not see it
    let (ok, _) = c.run(&c.repo, &["start", "issues/z.md"]);
    let agent = c.wt("z").join(".claude/worktrees/agent");
    git_ok(
        &c.repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "agent",
            agent.to_str().unwrap(),
        ],
    );
    std::fs::write(agent.join("wip.txt"), "wip\n").unwrap();
    let (holding, _) = c.run(&c.repo, &["finish", "z", "--force"]);
    let held = agent.join("wip.txt").is_file() && c.wt("z").is_dir();
    git_ok(
        &c.repo,
        &["worktree", "remove", "--force", agent.to_str().unwrap()],
    );
    let (freed, _) = c.run(&c.repo, &["finish", "z"]);
    vec![
        (
            "finish refuses a worktree with uncommitted changes",
            !dirty && c.wt("y").is_dir() && branch_exists(&c.repo, "y"),
        ),
        (
            "finish refuses, even with --force, a worktree holding another worktree",
            ok && !holding && held,
        ),
        (
            "finish removes it once the inner worktree is gone",
            freed && !c.wt("z").exists() && !branch_exists(&c.repo, "z"),
        ),
        (
            "finish does not remove a worktree that is not the one of the stem",
            !stranger && elsewhere.is_dir() && branch_exists(&c.repo, "w"),
        ),
    ]
}

// spec: PAR-15
fn finish_removes_a_legacy_sibling(c: &Ctx) -> Checks {
    git_ok(
        &c.repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "l",
            c.sibling("l").to_str().unwrap(),
        ],
    );
    let (merged, _) = c.run(&c.repo, &["finish", "l"]);
    git_ok(
        &c.repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "k",
            c.sibling("k").to_str().unwrap(),
        ],
    );
    commit_file(&c.sibling("k"), "k.txt");
    let (unmerged, _) = c.run(&c.repo, &["finish", "k"]);
    let (inside, _) = c.run(&c.sibling("k"), &["finish", "k", "--force"]);
    let kept = c.sibling("k").is_dir() && branch_exists(&c.repo, "k");
    let (forced, _) = c.run(&c.repo, &["finish", "k", "--force"]);
    vec![
        (
            "finish removes a merged worktree left at <repo>-<stem>",
            merged && !c.sibling("l").exists() && !branch_exists(&c.repo, "l"),
        ),
        (
            "finish refuses an unmerged one, and from inside it, keeping both",
            !unmerged && !inside && kept,
        ),
        (
            "finish --force removes it",
            forced && !c.sibling("k").exists() && !branch_exists(&c.repo, "k"),
        ),
    ]
}

fn commit_file(wt: &Path, name: &str) {
    std::fs::write(wt.join(name), format!("{name}\n")).unwrap();
    git_ok(wt, &["add", "-A"]);
    git_ok(wt, &["commit", "-qm", name, "--no-verify"]);
}

// spec: PAR-6
fn finish_accepts_a_patch_equivalent_branch(c: &Ctx) -> Checks {
    for stem in ["r", "m"] {
        let (ok, _) = c.run(&c.repo, &["start", &format!("issues/{stem}.md")]);
        assert!(ok);
        commit_file(&c.wt(stem), &format!("{stem}.txt"));
    }
    commit_file(&c.repo, "trunk-moved.txt");
    git_ok(&c.repo, &["cherry-pick", "r"]);
    git_ok(&c.repo, &["merge", "-q", "--squash", "m"]);
    git_ok(&c.repo, &["commit", "-qm", "squash m", "--no-verify"]);
    let (rebased, _) = c.run(&c.repo, &["finish", "r"]);
    let (squashed, _) = c.run(&c.repo, &["finish", "m"]);
    vec![
        (
            "finish accepts a rebased branch",
            rebased && !c.wt("r").exists() && !branch_exists(&c.repo, "r"),
        ),
        (
            "finish accepts a branch squashed from one commit",
            squashed && !c.wt("m").exists() && !branch_exists(&c.repo, "m"),
        ),
    ]
}

// spec: PAR-6
fn finish_accepts_a_squash_of_several_commits(c: &Ctx) -> Checks {
    for stem in ["s", "t"] {
        let (ok, _) = c.run(&c.repo, &["start", &format!("issues/{stem}.md")]);
        assert!(ok);
        commit_file(&c.wt(stem), &format!("{stem}1.txt"));
        commit_file(&c.wt(stem), &format!("{stem}2.txt"));
        git_ok(&c.repo, &["merge", "-q", "--squash", stem]);
        git_ok(
            &c.repo,
            &["commit", "-qm", &format!("squash {stem}"), "--no-verify"],
        );
    }
    let (squashed, _) = c.run(&c.repo, &["finish", "s"]);
    // trunk rewrites a squashed file: the content of `t` is no longer all on trunk, so it counts as unmerged.
    std::fs::write(c.repo.join("t1.txt"), "rewritten\n").unwrap();
    git_ok(&c.repo, &["commit", "-qam", "rewrite t1", "--no-verify"]);
    let (rewritten, _) = c.run(&c.repo, &["finish", "t"]);
    vec![
        (
            "finish accepts a branch squashed from several commits",
            squashed && !c.wt("s").exists() && !branch_exists(&c.repo, "s"),
        ),
        (
            "finish keeps a squashed branch whose content trunk changed afterwards",
            !rewritten && c.wt("t").is_dir() && branch_exists(&c.repo, "t"),
        ),
    ]
}

// spec: PAR-10
fn finish_forces_work_that_is_not_on_trunk(c: &Ctx) -> Checks {
    let (ok, _) = c.run(&c.repo, &["start", "issues/u.md"]);
    assert!(ok);
    commit_file(&c.wt("u"), "u1.txt");
    let (plain, msg) = c.run(&c.repo, &["finish", "u"]);
    let kept = c.wt("u").is_dir() && branch_exists(&c.repo, "u");
    std::fs::write(c.wt("u").join("dirty.txt"), "wip\n").unwrap();
    let (dirty, _) = c.run(&c.repo, &["finish", "u", "--force"]);
    let dirty_kept = c.wt("u").is_dir() && branch_exists(&c.repo, "u");
    std::fs::remove_file(c.wt("u").join("dirty.txt")).unwrap();
    let (forced, _) = c.run(&c.repo, &["finish", "u", "--force"]);
    vec![
        ("finish refuses work that is not on trunk without --force, names --force and keeps both", !plain && kept && msg.contains("--force")),
        ("finish --force still refuses a worktree with uncommitted changes", !dirty && dirty_kept),
        ("finish --force removes the worktree and the branch", forced && !c.wt("u").exists() && !branch_exists(&c.repo, "u")),
    ]
}

// spec: PAR-11
fn finish_refuses_from_inside_the_worktree(c: &Ctx) -> Checks {
    let (ok, _) = c.run(&c.repo, &["start", "issues/c.md"]);
    assert!(ok);
    let sub = c.wt("c").join("issues");
    let (inside, msg) = c.run(&c.wt("c"), &["finish", "c"]);
    let (below, _) = c.run(&sub, &["finish", "c"]);
    let (forced, _) = c.run(&sub, &["finish", "c", "--force"]);
    let kept = c.wt("c").is_dir() && branch_exists(&c.repo, "c");
    commit_file(&c.wt("c"), "c.txt");
    let (unmerged, unmerged_msg) = c.run(&c.wt("c"), &["finish", "c"]);
    let (outside_unmerged, _) = c.run(&c.repo, &["finish", "c"]);
    git_ok(&c.repo, &["merge", "-q", "--ff-only", "c"]);
    let (outside, _) = c.run(&c.repo, &["finish", "c"]);
    vec![
        (
            "finish refuses from the worktree to remove and names the main worktree",
            !inside && msg.contains(c.repo.to_str().unwrap()),
        ),
        (
            "finish refuses from a subdirectory of it, with or without --force",
            !below && !forced,
        ),
        ("a refusal keeps the worktree and the branch", kept),
        (
            "an unmerged branch gets the same refusal from inside",
            !unmerged && unmerged_msg.contains("inside") && !outside_unmerged,
        ),
        (
            "finish from the main worktree still removes it",
            outside && !c.wt("c").exists() && !branch_exists(&c.repo, "c"),
        ),
    ]
}

#[test]
fn issue_worktree() {
    let script = std::path::PathBuf::from("/tools").join("issue_worktree.rs");
    let root = std::fs::canonicalize(tempfile::tempdir().unwrap().keep()).unwrap();
    let _ = std::fs::remove_dir_all(&root);
    let repo = root.join("repo");
    std::fs::create_dir_all(repo.join("issues")).unwrap();
    for s in ["x", "y", "z", "w", "r", "m", "s", "t", "u", "c"] {
        std::fs::write(repo.join(format!("issues/{s}.md")), format!("# {s}\n")).unwrap();
    }
    // a project that tracks `.claude/` against a global gitignore un-ignores it
    std::fs::write(repo.join(".gitignore"), "!.claude/\n").unwrap();
    // the managed `.claude/.gitignore` and a personal configuration it ignores
    std::fs::create_dir_all(repo.join(".claude")).unwrap();
    std::fs::write(repo.join(".claude/.gitignore"), "cardinal.local.toml\n").unwrap();
    std::fs::write(repo.join(PERSONAL), PERSONAL_TEXT).unwrap();
    git_ok(&repo, &["init", "-q", "-b", "trunk", "."]);
    git_ok(&repo, &["add", "-A"]);
    git_ok(&repo, &["commit", "-qm", "init", "--no-verify"]);
    std::fs::write(repo.join("issues/new.md"), "# new\n").unwrap();
    std::fs::create_dir_all(repo.join(".git/info")).unwrap();
    std::fs::write(repo.join(".git/info/exclude"), EXCLUDE).unwrap();

    let c = Ctx {
        root: root.clone(),
        repo,
        script,
    };
    let mut failed = false;
    for checks in [
        start_creates_the_worktree(&c),
        start_makes_git_ignore_the_worktrees(&c),
        start_copies_the_personal_configuration(&c),
        start_refuses_and_changes_nothing(&c),
        finish_needs_a_merged_branch(&c),
        finish_removes_only_a_clean_worktree(&c),
        finish_removes_a_legacy_sibling(&c),
        finish_accepts_a_patch_equivalent_branch(&c),
        finish_accepts_a_squash_of_several_commits(&c),
        finish_forces_work_that_is_not_on_trunk(&c),
        finish_refuses_from_inside_the_worktree(&c),
    ] {
        for (name, ok) in checks {
            println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
            failed |= !ok;
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    assert!(!failed, "a check failed: see the FAIL lines");
}

/// `cardinal tool <name>` for the script path the checks were written against (`next_issue.rs` is `next-issue`).
fn tool_command(script: impl AsRef<std::path::Path>) -> std::process::Command {
    let stem = script
        .as_ref()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .replace('_', "-");
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.arg("tool").arg(stem);
    cmd
}
