//! Behavior test for `cardinal tool session-metrics` on fixture transcripts: columns, deduplicated assistant messages,
//! sub-agents, typed messages, hook refusals, skills, project, tolerance to unknown lines, no message text.

mod session_metrics_common;

use session_metrics_common::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The main transcript of the session `s1`, in `worktree`.
fn first_session(worktree: &Path) -> String {
    [
        typed(worktree, "2026-10-26T10:00:00.000Z", "please fix SECRET"),
        typed(worktree, "2026-10-26T10:00:05.000Z", "OK!"),
        typed(worktree, "2026-10-26T10:00:06.000Z", "1 a, 2 c"),
        typed(worktree, "2026-10-26T10:00:07.000Z", "c'est bon."),
        typed(worktree, "2026-10-26T10:00:08.000Z", "4k"),
        typed(worktree, "2026-10-26T10:00:09.000Z", "2a"),
        assistant(
            worktree,
            "2026-10-26T10:00:10.000Z",
            "m1",
            "model-a",
            SKILL,
            &usage(10, 20, Some(5), 100, 7),
        ),
        assistant(
            worktree,
            "2026-10-26T10:00:11.000Z",
            "m1",
            "model-a",
            BASH,
            &usage(10, 20, Some(5), 100, 7),
        ),
        assistant(
            worktree,
            "2026-10-26T10:00:12.000Z",
            "m1",
            "model-a",
            TEXT,
            &usage(10, 20, Some(5), 100, 7),
        ),
        assistant(
            worktree,
            "2026-10-26T10:00:20.000Z",
            "m2",
            "model-b",
            BASH,
            &usage(1, 2, None, 3, 4),
        ),
        refusal(
            worktree,
            "2026-10-26T10:00:21.000Z",
            ".claude/hooks/require_code_review.rs",
        ),
        refusal(
            worktree,
            "2026-10-26T10:00:22.000Z",
            ".claude/hooks/require_code_review.rs",
        ),
        refusal(worktree, "2026-10-26T10:00:23.000Z", "nothing"),
        refusal(
            worktree,
            "2026-10-26T10:00:23.500Z",
            "~/.rsync/notes.rst .claude/hooks/block_ai_attribution.rs",
        ),
        error_result(
            worktree,
            "2026-10-26T10:00:23.700Z",
            "PreToolUse:Bash failed: the hook error: was elsewhere",
        ),
        // the hooks became tools of `cardinal run` (`harness run` before the rename): same names as the scripts they
        // were
        error_result(
            worktree,
            "2026-10-26T10:00:23.800Z",
            "PreToolUse:Bash hook error: [cardinal run review-gate check]: SECRET refusal text",
        ),
        error_result(
            worktree,
            "2026-10-26T10:00:23.900Z",
            "PreToolUse:Bash hook error: [harness run some-new-tool]: SECRET refusal text",
        ),
        line("brand-new-type", worktree, "2026-10-26T10:00:24.000Z", ""),
        "{not json\n".to_string(),
        line(
            "assistant",
            worktree,
            "2026-10-26T10:00:25.000Z",
            ",\"message\":{\"id\":\"m9\",\"model\":\"model-a\",\"content\":[]}",
        ),
        line("attachment", worktree, "2026-10-26T11:00:00.000Z", ""),
        "{\"type\":\"ai-title\",\"sessionId\":\"x\"}\n".to_string(),
        // the metadata types of Claude Code since 2026-11, in their real shape: no timestamp, cwd nor gitBranch
        "{\"type\":\"tools-used\",\"tools\":[\"Bash\"],\"sessionId\":\"x\"}\n".to_string(),
        "{\"type\":\"relocated\",\"relocatedCwd\":\"/elsewhere\",\"sessionId\":\"x\"}\n"
            .to_string(),
        "{\"type\":\"worktree-state\",\"worktreeSession\":{},\"sessionId\":\"x\"}\n".to_string(),
        "{\"type\":\"permission-mode\",\"permissionMode\":\"auto\",\"sessionId\":\"x\"}\n"
            .to_string(),
    ]
    .concat()
}

/// The sessions of removed worktrees named apart from their branches, next to `repo` and to a repository `myrepo-docs`.
fn renamed_worktrees(tmp: &Path, repo: &Path, proj: &Path) {
    let docs = tmp.join("myrepo-docs");
    git_repo(&docs);
    std::fs::create_dir_all(repo.join("plain")).unwrap();
    for (id, cwd, branch) in [
        ("s7", tmp.join("myrepo-feat"), "issue-b"),
        ("s8", tmp.join("myrepo-docs-fix"), "docs-fix"),
        ("s9", tmp.join("myrepo-docs-x/crates"), "main"),
        ("s10", repo.join("plain-x"), "main"),
    ] {
        let session = line(
            "user",
            &cwd,
            "2026-10-26T09:00:00.000Z",
            &format!(
                ",\"gitBranch\":\"{branch}\",\"message\":{{\"role\":\"user\",\"content\":\"ok\"}}"
            ),
        );
        std::fs::write(proj.join(format!("{id}.jsonl")), session).unwrap();
    }
}

/// Builds the fixture projects directory; returns (projects dir, main repository).
fn fixture(tmp: &Path) -> (PathBuf, PathBuf) {
    let repo = tmp.join("myrepo");
    git_repo(&repo);
    let worktree = tmp.join("myrepo-my_issue");
    sh(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "my_issue",
            worktree.to_str().unwrap(),
        ],
    );
    let projects = tmp.join("projects");
    let proj = projects.join("-tmp-myrepo");
    std::fs::create_dir_all(proj.join("s1/subagents")).unwrap();
    let s1 = first_session(&worktree);
    std::fs::write(proj.join("s1.jsonl"), s1).unwrap();
    let sub = [
        line("user", &worktree, "2026-10-26T10:00:30.000Z", ",\"isSidechain\":true,\"message\":{\"role\":\"user\",\"content\":\"subagent SECRET prompt\"}"),
        assistant(&worktree, "2026-10-26T10:00:31.000Z", "m3", "model-c", REVIEW, &usage(1000, 0, None, 0, 0)),
        refusal(&worktree, "2026-10-26T10:00:32.000Z", ".claude/hooks/clippy_gate.rs"),
        "garbage\n".to_string(),
    ]
    .concat();
    std::fs::write(proj.join("s1/subagents/agent-x.jsonl"), sub).unwrap();
    let gone = Path::new("/nonexistent/elsewhere");
    let s2 = [typed(gone, "2026-10-26T09:00:00.000Z", "yes")].concat();
    std::fs::create_dir_all(projects.join("-nonexistent-elsewhere")).unwrap();
    std::fs::write(projects.join("-nonexistent-elsewhere/s2.jsonl"), s2).unwrap();
    let lib = tmp.join("lib");
    git_repo(&lib);
    sh(
        &repo,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            lib.to_str().unwrap(),
            "vendor/lib",
        ],
    );
    let s3 = typed(&repo.join("vendor/lib"), "2026-10-26T09:00:00.000Z", "yes");
    let removed = tmp.join("myrepo-gone_issue");
    let s4 = line(
        "user",
        &removed,
        "2026-10-26T09:00:00.000Z",
        ",\"gitBranch\":\"gone_issue\",\"message\":{\"role\":\"user\",\"content\":\"ok\"}",
    );
    std::fs::write(proj.join("s3.jsonl"), s3).unwrap();
    std::fs::write(proj.join("s4.jsonl"), s4).unwrap();
    let nested = repo.join(".claude/worktrees/gone_nested/crates");
    let s5 = line(
        "user",
        &nested,
        "2026-10-26T09:00:00.000Z",
        ",\"gitBranch\":\"gone_nested\",\"message\":{\"role\":\"user\",\"content\":\"ok\"}",
    );
    std::fs::write(proj.join("s5.jsonl"), s5).unwrap();
    renamed_worktrees(tmp, &repo, &proj);
    let inline_subagent = line(
        "assistant",
        &repo,
        "2026-10-26T09:00:01.000Z",
        &format!(",\"isSidechain\":true,\"message\":{{\"id\":\"side\",\"model\":\"model-a\",\"content\":[]{}}}", usage(9000, 0, None, 0, 0)),
    );
    let s6 = [
        assistant(
            &repo,
            "2026-10-26T09:00:00.000Z",
            "own",
            "model-a",
            TEXT,
            &usage(50, 0, None, 0, 0),
        ),
        inline_subagent,
    ]
    .concat();
    std::fs::write(proj.join("s6.jsonl"), s6).unwrap();
    (projects, repo)
}

// spec: EVA-1, EVA-2, EVA-3, EVA-4, EVA-5, EVA-6, EVA-7, EVA-9, EVA-10, EVA-16, EVA-H1
fn row_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let (projects, _) = fixture(tmp);
    let out = tool_command(dir.join("session_metrics.rs"))
        .arg(&projects)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let rows = rows(&stdout).unwrap_or_default();
    let empty = HashMap::new();
    let s1 = rows.get("s1").unwrap_or(&empty);
    let s2 = rows.get("s2").unwrap_or(&empty);
    let s3 = rows.get("s3").unwrap_or(&empty);
    let s4 = rows.get("s4").unwrap_or(&empty);
    let s5 = rows.get("s5").unwrap_or(&empty);
    let s6 = rows.get("s6").unwrap_or(&empty);
    let s7 = rows.get("s7").unwrap_or(&empty);
    let s8 = rows.get("s8").unwrap_or(&empty);
    let s9 = rows.get("s9").unwrap_or(&empty);
    let s10 = rows.get("s10").unwrap_or(&empty);
    let col =
        |r: &HashMap<String, String>, c: &str| r.get(c).cloned().unwrap_or_else(|| "<none>".into());
    vec![
        ("the script succeeds", out.status.success()),
        ("header names every column, one row per session, none for the sub-agent", rows.len() == 10),
        ("a message repeated on 3 lines is one turn; the sub-agent adds one", col(s1, "turns") == "3"),
        ("tool calls count every tool_use block, sub-agent included", col(s1, "tool_calls") == "4"),
        ("tokens are added once per message, sub-agent included", col(s1, "input_tokens") == "1011" && col(s1, "output_tokens") == "22"),
        ("thinking tokens come from output_tokens_details", col(s1, "thinking_tokens") == "5"),
        ("cache tokens are added once per message", col(s1, "cache_read_tokens") == "103" && col(s1, "cache_creation_tokens") == "11"),
        ("the main context peak is the largest main turn, sub-agents excluded", col(s1, "main_context_peak") == "117"),
        ("a sub-agent turn written in the main transcript is not a main turn", col(s6, "main_context_peak") == "50"),
        ("a session with no assistant message has a peak of 0", col(s2, "main_context_peak") == "0"),
        ("models in order of first appearance, sub-agent included", col(s1, "models") == "model-a,model-b,model-c"),
        ("versions in order of first appearance", col(s1, "versions") == "2.1.1,2.1.2"),
        ("issues are the branches other than main", col(s1, "issues") == "my_issue"),
        ("start is the earliest timestamp, duration to the latest", col(s1, "start") == "2026-10-26T10:00:00.000Z" && col(s1, "duration_s") == "3600"),
        ("plain validations and stop answers are told apart from the rest", col(s1, "validations") == "3" && col(s1, "other_messages") == "3"),
        ("a bare yes is a validation", col(s2, "validations") == "1" && col(s2, "other_messages") == "0"),
        ("hook refusals are counted by hook, sorted, sub-agent included", col(s1, "hook_refusals") == "block_ai_attribution=1;clippy_gate=1;require_code_review=3;some_new_tool=1;unknown=1"),
        ("skills are counted by name, sorted", col(s1, "skills") == "code-review=1;commit-message=1"),
        ("a linked worktree counts with its main repository", col(s1, "project") == "myrepo"),
        ("a cwd outside git falls back to its last component", col(s2, "project") == "elsewhere"),
        ("a submodule is its own project", col(s3, "project") == "lib"),
        ("a removed issue worktree counts with its repository", col(s4, "project") == "myrepo"),
        ("a removed worktree under .claude/worktrees counts with its repository", col(s5, "project") == "myrepo"),
        ("a removed worktree named apart from its branches counts with its repository", col(s7, "project") == "myrepo"),
        ("the branch of the session names the repository of a removed worktree first", col(s8, "project") == "myrepo"),
        ("else the longest repository among the prefixes of a removed ancestor", col(s9, "project") == "myrepo-docs"),
        ("a prefix that is not the top level of a work tree is no repository", col(s10, "project") == "plain-x"),
        ("unreadable lines are counted, sub-agent included", col(s1, "unreadable_lines") == "2"),
        ("unknown line types are counted, tools-used, relocated, worktree-state and permission-mode are known", col(s1, "unknown_lines") == "1"),
        ("assistant lines without usage are missing fields, metadata lines without timestamp are not", col(s1, "missing_fields") == "1"),
        ("no text of a message, a tool input or a tool result is printed", !stdout.contains("SECRET")),
    ]
}

// spec: EVA-12
fn shared_reader_checks(_dir: &Path) -> Vec<(&'static str, bool)> {
    let tools = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tools");
    let uses = |name: &str| {
        std::fs::read_to_string(tools.join(name))
            .is_ok_and(|s| s.contains("use super::transcripts;"))
    };
    vec![
        (
            "audit_sources reads the transcripts through the shared module",
            uses("audit_sources.rs"),
        ),
        (
            "session_metrics reads the transcripts through the shared module",
            uses("session_metrics.rs"),
        ),
    ]
}

// spec: EVA-15
fn default_root_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let config = tmp.join("default-root/config");
    let home = tmp.join("default-root/home");
    let session = |root: &Path, name: &str| {
        let proj = root.join("projects/-tmp-p");
        std::fs::create_dir_all(&proj).unwrap();
        let text = typed(tmp, "2026-10-26T09:00:00.000Z", "hi");
        std::fs::write(proj.join(format!("{name}.jsonl")), text).unwrap();
    };
    session(&config, "from_config");
    session(&home.join(".claude"), "from_home");
    let sessions = |cmd: &mut std::process::Command| {
        let out = cmd.output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let mut names: Vec<String> = rows(&stdout).unwrap_or_default().into_keys().collect();
        names.sort();
        (out.status.success(), names)
    };
    let set = sessions(
        tool_command(dir.join("session_metrics.rs"))
            .env("CLAUDE_CONFIG_DIR", &config)
            .env("HOME", &home),
    );
    let unset = sessions(
        tool_command(dir.join("session_metrics.rs"))
            .env("CLAUDE_CONFIG_DIR", "")
            .env("HOME", &home),
    );
    let neither = tool_command(dir.join("session_metrics.rs"))
        .env_remove("CLAUDE_CONFIG_DIR")
        .env_remove("HOME")
        .output()
        .unwrap();
    let error = String::from_utf8_lossy(&neither.stderr).into_owned();
    vec![
        (
            "the default projects directory is the one of CLAUDE_CONFIG_DIR",
            set == (true, vec!["from_config".to_string()]),
        ),
        (
            "an empty CLAUDE_CONFIG_DIR falls back to the home directory",
            unset == (true, vec!["from_home".to_string()]),
        ),
        (
            "with neither variable, the error names both",
            !neither.status.success()
                && error.contains("CLAUDE_CONFIG_DIR")
                && error.contains("HOME"),
        ),
    ]
}

#[test]
fn session_metrics() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    let checks: Vec<_> = row_checks(&dir, &tmp)
        .into_iter()
        .chain(shared_reader_checks(&dir))
        .chain(default_root_checks(&dir, &tmp))
        .collect();
    assert_checks(checks);
}

/// A tool result refused by a hook with `text`, as Claude Code writes it.
fn refused(text: &str) -> serde_json::Value {
    serde_json::json!({"type": "user", "cwd": "/nonexistent/p", "timestamp": "2026-11-12T10:00:00.000Z",
        "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "t", "is_error": true,
        "content": text}]}})
}

// spec: EVA-5
#[test]
fn a_refusal_of_read_hint_is_named_after_it() {
    let files = tempfile::tempdir().unwrap();
    let big = files.path().join("big.rs");
    std::fs::write(&big, format!("{}\n", "x".repeat(99)).repeat(300)).unwrap();
    let event = serde_json::json!({"hook_event_name": "PreToolUse", "session_id": "s", "tool_name": "Read",
        "tool_input": {"file_path": big}});
    let mut hook = std::process::Command::new(env!("CARGO_BIN_EXE_cardinal"));
    hook.args(["tool", "read-hint"])
        .env("TMPDIR", files.path())
        .env("USER", "tester")
        .env_remove("XDG_RUNTIME_DIR")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped());
    let mut child = hook.spawn().unwrap();
    std::io::Write::write_all(
        &mut child.stdin.take().unwrap(),
        event.to_string().as_bytes(),
    )
    .unwrap();
    let out = child.wait_with_output().unwrap();
    let output: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let reason = output["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .unwrap();
    let current = refused(&format!("PreToolUse:Read hook error: {reason}"));
    // the text of the revisions before this requirement, whose reason was the same
    let older = refused("PreToolUse:Read hook error: A whole read of this file returns about 30000 bytes: read the range you need");
    let other = refused("PreToolUse:Read hook error: some other denial");
    assert_eq!(
        cell_of(&[current, older, other], "hook_refusals"),
        "read_hint=2;unknown=1"
    );
}

/// An assistant line on `branch` running the shell `commands`, as one message of `Bash` calls.
fn running(branch: &str, id: &str, commands: &[&str]) -> serde_json::Value {
    let blocks: Vec<serde_json::Value> = commands
        .iter()
        .map(|c| serde_json::json!({"type": "tool_use", "id": "b", "name": "Bash", "input": {"command": c}}))
        .collect();
    serde_json::json!({"type": "assistant", "cwd": "/nonexistent/p", "timestamp": "2026-11-12T10:00:00.000Z",
        "gitBranch": branch, "message": {"id": id, "model": "m", "content": blocks,
        "usage": {"input_tokens": 1, "output_tokens": 1}}})
}

// spec: EVA-6, EVA-21
#[test]
fn a_session_is_tied_to_the_issues_it_claims_or_closes() {
    let claims = [
        "cardinal run issue-claim issues/a.md",
        "cd /x && cardinal run close-issue /x/issues/done/b.md && git status",
        "cardinal run close-issue issues/a.md; cat issues/c.md",
        "cardinal run issue-claim 'issues/d.md' | tee log",
        "echo cardinal run issue-claim; grep x issues/e.md",
        "cardinal run next-issue issues/f.md",
        "cardinal run issue-claim docs/g.md",
        "just close-issue issues/h.md",
        "git commit -m 'then cardinal run close-issue issues/i.md'",
        "git commit -F - <<EOF\nfix: x\n\nthen cardinal run close-issue issues/j.md moves it\nEOF",
        "target/release/cardinal run issue-claim issues/k.md",
        "echo target/cardinal run issue-claim issues/l.md",
        "notcardinal run issue-claim issues/m.md",
        "./bin/just close-issue issues/n.md",
    ];
    let on_main = [running("main", "m1", &claims)];
    assert_eq!(cell_of(&on_main, "issues"), "a,b,d,h,k");
    // a claim names the issue, whatever the branch is called; with none, the branch does
    let on_branch = [running("z", "m1", &claims[..1])];
    assert_eq!(cell_of(&on_branch, "issues"), "a");
    let unclaimed = [running("z", "m1", &claims[5..6])];
    assert_eq!(cell_of(&unclaimed, "issues"), "z");
    // a sidechain line is the work of a sub-agent, not of the session
    let mut sidechain = running("main", "m2", &claims[..1]);
    sidechain["isSidechain"] = serde_json::json!(true);
    assert_eq!(cell_of(&[sidechain], "issues"), "");
}
