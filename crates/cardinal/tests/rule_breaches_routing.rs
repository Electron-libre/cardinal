//! Behavior test for the skill routing rules of `cardinal tool rule-breaches` (BRE-15 to BRE-18): the skills a session
//! is due to load before their trigger, typed or loaded with their prefix, and none for a sub-agent or a project that
//! synced none of them.

mod rule_breaches_common;

use rule_breaches_common::*;
use serde_json::json;
use std::path::Path;

/// The lock of a project that synced the three routed skills.
const ROUTED_LOCK: &str = "# lock\nh1  .claude/skills/commit-message/SKILL.md\nh2  .claude/skills/issue-workflow/SKILL.md\nh3  .claude/skills/write-spec/SKILL.md\n";

/// The sessions of the routing checks besides the main one: a typed skill, a sub-agent, a project with no routed skill.
fn routing_other_sessions(tmp: &Path, repo: &Path, config: &str) {
    let projects = tmp.join("projects");
    // a skill typed with its prefix is loaded
    let mut typed = Transcript::new(repo, "2026-02-01T11:00:00.000Z");
    typed.push(json!({"type": "user", "message": {"role": "user", "content": "<command-name>/files:commit-message</command-name>"}}));
    typed.run("c1", "Bash", &bash("git commit -m SECRET"), None);
    typed.write(&projects.join("-r/u.jsonl"));
    // a sub-agent shares neither the skills nor the triggers of the session
    let mut sub = Transcript::new(repo, "2026-02-01T10:00:00.000Z");
    sub.run("x1", "Bash", &bash("git commit -m SECRET"), None);
    sub.write(&projects.join("-r/s/subagents/agent-1.jsonl"));
    // a session that synced none of the routed skills is not due to load them
    let bare = new_repo(&tmp.join("n"));
    commit_at(
        &bare,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[
            (".claude/cardinal.toml", config),
            (".claude/cardinal.lock", "# lock\nh1  AGENTS.md\n"),
        ],
    );
    let mut n = Transcript::new(&bare, "2026-02-01T10:00:00.000Z");
    n.typed("go");
    n.run("c1", "Bash", &bash("git commit -m SECRET"), None);
    n.write(&projects.join("-n/s.jsonl"));
}

/// The commits of BRE-16: the throwaway or checked ones trigger nothing; the lines of those in the project, which do.
fn skipped_commits(t: &mut Transcript, repo: &Path) -> Vec<usize> {
    let repo = repo.display();
    let skipped = [
        "cd \"$(mktemp -d)\" && git init -q && git commit -q -m SECRET".to_string(),
        "git init -q t && cd t && git commit -m SECRET".to_string(),
        "git clone -q . t && cd t && git commit -m SECRET".to_string(),
        "cd /tmp/x && git commit -m SECRET".to_string(),
        "git -C /tmp/y commit -m SECRET".to_string(),
        "printf '%s\\n' \"$m\" | cardinal run commit-check && git commit -m \"$m\"".to_string(),
        "git commit -m SECRET && just check-commit".to_string(),
        "d=$(mktemp -d); cd $d && git init -q && git commit -m SECRET".to_string(),
        format!("cd {repo}/../x && git commit -m SECRET"),
    ];
    let counted = [
        "git commit -m 'fix: drop the mktemp dirs'".to_string(),
        "git commit -m 'fix: stop leaking $(mktemp -d) dirs'".to_string(),
        "git add src/commit_check.rs && git commit -m SECRET".to_string(),
        "git commit -m SECRET && out=$(mktemp)".to_string(),
        format!("cd /tmp/s && cd {repo} && git commit -m SECRET"),
        format!("cd {repo}/sub/.. && git commit -m SECRET"),
    ];
    for (i, command) in skipped.iter().enumerate() {
        t.run(&format!("k{i}"), "Bash", &bash(command), None);
    }
    counted
        .iter()
        .enumerate()
        .map(|(i, command)| t.run(&format!("n{i}"), "Bash", &bash(command), None))
        .collect()
}

// spec: BRE-15, BRE-16, BRE-17, BRE-18
fn routing_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("routing");
    let projects = tmp.join("projects");
    let config = "[cardinal]\nsource = \"x\"\nrev = \"r1\"\n";
    let repo = new_repo(&tmp.join("r"));
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[
            (".claude/cardinal.toml", config),
            (".claude/cardinal.lock", ROUTED_LOCK),
            ("specs/old.md", "SECRET"),
        ],
    );
    let write =
        |path: &str, content: &str| json!({"file_path": repo.join(path), "content": content});
    let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
    t.typed("go");
    let refused = t.run("c0", "Bash", &bash("git commit -m SECRET"), Some("exit 1"));
    // a call that failed loaded nothing; a command that writes no commit message triggers nothing
    t.run(
        "c1a",
        "Skill",
        &json!({"skill": "commit-message"}),
        Some("Unknown skill"),
    );
    t.run(
        "c1b",
        "Bash",
        &bash("git log --oneline | grep commit"),
        None,
    );
    t.run("c1c", "Bash", &bash("git commit --amend --no-edit"), None);
    let unloaded = t.run(
        "c1",
        "Bash",
        &bash("git add a && git -C . commit -m SECRET && git rebase --continue --no-edit"),
        None,
    );
    let counted = skipped_commits(&mut t, &repo);
    t.run("c2", "Skill", &json!({"skill": "commit-message"}), None);
    t.run("c3", "Bash", &bash("git commit -m SECRET"), None);
    let plan = t.run(
        "w1",
        "Write",
        &write("issues/i.md", "# Tracking\n\nPlan: proposed\n"),
        None,
    );
    t.push(json!({"type": "user", "message": {"role": "user", "content": "<command-name>/issue-workflow</command-name>\n<command-args>SECRET</command-args>"}}));
    t.run("w2", "Write", &write("issues/j.md", "Plan: proposed"), None);
    let spec = t.run("w3", "Write", &write("specs/new.md", "SECRET"), None);
    t.run("w4", "Write", &write("specs/old.md", "SECRET"), None);
    t.run("w4b", "Write", &write("specs/new.md", "SECRET again"), None);
    t.run("w5", "Write", &write("specs/README.md", "SECRET"), None);
    t.run("w6", "Write", &write("specs/notes.txt", "SECRET"), None);
    t.run("w7", "Skill", &json!({"skill": "files:write-spec"}), None);
    t.run("w8", "Write", &write("specs/other.md", "SECRET"), None);
    t.write(&projects.join("-r/s.jsonl"));
    routing_other_sessions(&tmp, &repo, config);
    let row = |rule: &str, kind: &str, line: usize, detail: &str| {
        format!("s\tr\tr1\tskill-routing-{rule}\t{kind}\ts.jsonl:{line}\t{detail}")
    };
    let mut expected = vec![
        row("commit-message", "blocked", refused, ""),
        row("commit-message", "breach", unloaded, ""),
    ];
    expected.extend(
        counted
            .iter()
            .map(|&line| row("commit-message", "breach", line, "")),
    );
    expected.extend([
        row("issue-workflow", "breach", plan, ""),
        row("write-spec", "breach", spec, "specs/new.md"),
    ]);
    let (ok, out) = run(dir, &projects, false);
    let rows: Vec<String> = out
        .lines()
        .filter(|l| l.contains("skill-routing"))
        .map(str::to_string)
        .collect();
    let (_, summary) = run(dir, &projects, true);
    let summary: Vec<&str> = summary
        .lines()
        .filter(|l| l.starts_with("skill-routing"))
        .collect();
    let want_summary = [
        "skill-routing-commit-message\tr1\t2\t1\t7\t1\t",
        "skill-routing-issue-workflow\tr1\t1\t1\t1\t0\t",
        "skill-routing-write-spec\tr1\t1\t1\t1\t0\t",
    ];
    if rows != expected || summary != want_summary {
        println!("{out}{}", summary.join("\n"));
    }
    vec![
        ("the routing rules succeed".to_string(), ok),
        ("a trigger is a finding until its skill is called or typed, in the session's own transcript only".to_string(), rows == expected),
        ("a routing rule applies to the sessions holding its trigger whose lock lists its skill".to_string(), summary == want_summary),
    ]
}

#[test]
fn rule_breaches_routing() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    let checks: Vec<_> = routing_checks(&dir, &tmp).into_iter().collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
