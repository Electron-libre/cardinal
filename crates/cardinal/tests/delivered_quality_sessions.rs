//! Behavior test for the transcript columns of `cardinal tool delivered-quality` (QUA-7, QUA-8, QUA-9, QUA-15, QUA-16,
//! QUA-17): the sessions that made the commits of each done issue, their failed checks, typed messages, tokens and
//! setup, and the summary per revision, setup and milestone.

mod delivered_quality_common;

use delivered_quality_common::*;
use std::path::{Path, PathBuf};

/// A transcript line of `kind` in `cwd` at `ts`, with `fields` merged in.
fn line(kind: &str, cwd: &Path, ts: &str, fields: &serde_json::Value) -> String {
    let mut line = serde_json::json!({"type": kind, "cwd": cwd, "timestamp": ts, "sessionId": "x", "gitBranch": "main"});
    line.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    format!("{line}\n")
}

fn typed(cwd: &Path, ts: &str, text: &str) -> String {
    line(
        "user",
        cwd,
        ts,
        &serde_json::json!({"message": {"role": "user", "content": text}}),
    )
}

/// A `Bash` call at `ts` and its result a second later, an error when `error`.
fn bash(cwd: &Path, ts: &str, id: &str, command: &str, error: bool) -> String {
    let call = line(
        "assistant",
        cwd,
        &format!("{ts}:00Z"),
        &serde_json::json!({"message": {"id": id, "content": [{"type": "tool_use", "id": id, "name": "Bash", "input": {"command": command}}]}}),
    );
    let result = line(
        "user",
        cwd,
        &format!("{ts}:01Z"),
        &serde_json::json!({"message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": id, "is_error": error, "content": "SECRET output"}]}}),
    );
    call + &result
}

/// An assistant message `id` at `ts` with the usage `input`, `cache_read`, `cache_creation`, `output`.
fn usage(cwd: &Path, ts: &str, id: &str, tokens: [u64; 4]) -> String {
    let [input, cache_read, cache_creation, output] = tokens;
    line(
        "assistant",
        cwd,
        ts,
        &serde_json::json!({"message": {"id": id, "content": [{"type": "text", "text": "SECRET answer"}], "usage": {"input_tokens": input, "cache_read_input_tokens": cache_read, "cache_creation_input_tokens": cache_creation, "output_tokens": output}}}),
    )
}

/// An advised assistant message `id` at `ts`: a top-level usage of 1 input and 1 output token, and iterations whose
/// tokens it already adds up (the executor's) or never holds (the advisor's).
fn advised(cwd: &Path, ts: &str, id: &str) -> String {
    let iteration = |kind: &str| serde_json::json!({"type": kind, "input_tokens": 500, "cache_read_input_tokens": 500, "output_tokens": 500});
    line(
        "assistant",
        cwd,
        ts,
        &serde_json::json!({"message": {"id": id, "content": [], "usage": {"input_tokens": 1, "output_tokens": 1, "iterations": [iteration("message"), iteration("advisor_message")]}}}),
    )
}

fn write(path: &Path, lines: &[String]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, lines.concat()).unwrap();
}

// spec: QUA-7, QUA-8, QUA-9, QUA-15
fn session_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("sessions");
    let repo = fixture(&tmp);
    let projects = tmp.join("projects");
    // S0, on another branch, starts after S1 and holds the first commit of alpha too: S1, on alpha's branch, wins.
    write(
        &projects.join("-proj/s0.jsonl"),
        &[
            typed(&repo, "2026-01-02T20:00:00Z", "SECRET early"),
            bash(&repo, "2026-01-03T10:00", "s0a", "just check", true),
        ],
    );
    // S1 runs on the branch of alpha from before the issues were opened: all its span up to alpha's last commit counts.
    let s1 = [
        typed(&repo, "2026-01-01T23:00:00Z", "SECRET prepare alpha"),
        typed(&repo, "2026-01-02T23:00:00Z", "SECRET do alpha"),
        usage(&repo, "2026-01-02T23:30:00Z", "u1", [10, 1000, 200, 50]),
        usage(&repo, "2026-01-02T23:31:00Z", "u1", [10, 1000, 200, 50]),
        advised(&repo, "2026-01-02T23:35:00Z", "u4"),
        bash(&repo, "2026-01-03T01:00", "a1", "just check", true),
        bash(&repo, "2026-01-03T02:00", "a2", "just test-harness", false),
        bash(&repo, "2026-01-03T03:00", "a3", "cargo check", true),
        bash(&repo, "2026-01-03T03:30", "a4", "echo just a check", true),
        bash(
            &repo,
            "2026-01-03T03:40",
            "a7",
            "cd x && ~/.cargo/bin/cardinal run project-check",
            true,
        ),
        bash(&repo, "2026-01-03T03:45", "a8", "cardinal check", true),
        bash(
            &repo,
            "2026-01-04T05:00",
            "a5",
            "cd x && just check-harness",
            true,
        ),
        typed(&repo, "2026-01-04T08:00:00Z", "ok"),
        bash(&repo, "2026-01-04T09:00", "a6", "just check", true),
        typed(&repo, "2026-01-05T00:30:00Z", "1 a, 2 b"),
        typed(&repo, "2026-01-05T01:00:00Z", "SECRET after"),
        usage(&repo, "2026-01-05T00:45:00Z", "u2", [1000, 0, 0, 1000]),
    ];
    write(
        &projects.join("-proj/s1.jsonl"),
        &s1.map(|l| l.replace("\"gitBranch\":\"main\"", "\"gitBranch\":\"alpha\"")),
    );
    write(
        &projects.join("-proj/s1/subagents/agent-1.jsonl"),
        &[
            bash(&repo, "2026-01-03T04:00", "b1", "just check", true),
            usage(&repo, "2026-01-02T23:40:00Z", "u3", [5, 0, 0, 7]),
        ],
    );
    // delta, closed after every session ended, has no session: its tokens are unknown
    commit_at(
        &repo,
        "2026-03-01T00:00:00Z",
        "docs: close delta",
        &[("issues/done/delta.md", Some("# delta\n\n# Tracking\n"))],
    );
    let (ok, out) = run(dir, &repo, &["--projects", projects.to_str().unwrap()]);
    let r = rows(&out);
    let (_, summed) = run(
        dir,
        &repo,
        &["--projects", projects.to_str().unwrap(), "--summary"],
    );
    let tail = |revision: &str| {
        summed
            .lines()
            .find(|l| l.starts_with(&format!("{revision}\t")))
            .map_or_default(|l| l.split('\t').rev().take(4).collect::<Vec<_>>().join(" "))
    };
    let (_, none) = run(dir, &repo, &[]);
    let n = rows(&none);
    vec![
        ("the script reads the transcripts".to_string(), ok),
        ("the sessions of an issue are the latest-starting ones holding its commits, its branch first".to_string(), has(&r, "alpha", "sessions", "1") && has(&r, "beta", "sessions", "1")),
        ("the failed checks of its span, through just or cardinal run project-check, are counted, not other commands nor sub-agents".to_string(), has(&r, "alpha", "failed_checks", "4") && has(&r, "beta", "failed_checks", "1")),
        ("the typed messages of its span are split into validations and the rest".to_string(), has(&r, "alpha", "validations", "1") && has(&r, "alpha", "other_messages", "2") && has(&r, "beta", "validations", "1") && has(&r, "beta", "other_messages", "0")),
        ("the top-level tokens of its span are summed once per message, sub-agents included, not the iterations".to_string(), has(&r, "alpha", "input_tokens", "1216") && has(&r, "alpha", "cache_read_tokens", "1000") && has(&r, "alpha", "output_tokens", "58") && has(&r, "beta", "input_tokens", "0")),
        ("the tokens of an issue with no session are empty, not 0".to_string(), has(&r, "delta", "sessions", "0") && ["input_tokens", "cache_read_tokens", "output_tokens"].iter().all(|c| has(&r, "delta", c, ""))),
        ("the summary sums the known tokens and counts their issues".to_string(), tail("r1") == "1 58 1000 1216" && tail("r3") == "0 0 0 0"),
        ("the transcript columns are empty when the projects cannot be read".to_string(), ["sessions", "failed_checks", "validations", "other_messages", "input_tokens", "cache_read_tokens", "output_tokens"].iter().all(|c| has(&n, "alpha", c, ""))),
        ("no text of a message is printed".to_string(), !out.contains("SECRET")),
    ]
}

// spec: QUA-7
fn claim_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("claims");
    let repo = fixture(&tmp);
    commit_at(
        &repo,
        "2026-02-01T12:00:00Z",
        "docs: close zeta",
        &[("issues/done/zeta.md", Some("# zeta\n\n# Tracking\n"))],
    );
    let projects = tmp.join("projects");
    // both sessions hold the close of zeta: the earlier one, on the default branch, claimed it
    write(
        &projects.join("-proj/claimer.jsonl"),
        &[
            bash(
                &repo,
                "2026-02-01T08:00",
                "c1",
                "cardinal run issue-claim issues/zeta.md",
                false,
            ),
            usage(&repo, "2026-02-01T09:00:00Z", "z1", [1, 0, 0, 3]),
            typed(&repo, "2026-02-01T13:00:00Z", "SECRET end"),
        ],
    );
    // the later one runs on the branch of zeta and claims another issue: the claim of zeta outranks both
    let other = [
        bash(
            &repo,
            "2026-02-01T10:00",
            "c2",
            "cardinal run issue-claim issues/alpha.md",
            false,
        ),
        usage(&repo, "2026-02-01T10:30:00Z", "z2", [1, 0, 0, 9]),
        typed(&repo, "2026-02-01T13:00:00Z", "SECRET end"),
    ];
    write(
        &projects.join("-proj/other.jsonl"),
        &other.map(|l| l.replace("\"gitBranch\":\"main\"", "\"gitBranch\":\"zeta\"")),
    );
    let (_, out) = run(dir, &repo, &["--projects", projects.to_str().unwrap()]);
    let r = rows(&out);
    vec![(
        "a commit counts with the session that claimed its issue, before one on its branch or one that claimed another".to_string(),
        has(&r, "zeta", "sessions", "1") && has(&r, "zeta", "output_tokens", "3"),
    )]
}

/// An assistant message `id` at `ts` of `model`, with `advisor` set as its advisor model when not empty.
fn modeled(cwd: &Path, ts: &str, id: &str, model: &str, advisor: &str) -> String {
    let mut fields = serde_json::json!({"message": {"id": id, "model": model, "content": [], "usage": {"input_tokens": 1, "output_tokens": 1}}});
    if !advisor.is_empty() {
        fields["advisorModel"] = advisor.into();
    }
    line("assistant", cwd, ts, &fields)
}

/// The repository of `fixture` and its sessions under several setups, with two more issues closed in `r3`.
fn setup_fixture(tmp: &Path) -> (PathBuf, PathBuf) {
    let repo = fixture(tmp);
    let projects = tmp.join("projects");
    let (sonnet, opus) = ("claude-sonnet-5-5", "claude-opus-5-5");
    let on_alpha = |lines: Vec<String>| -> Vec<String> {
        lines
            .iter()
            .map(|l| l.replace("\"gitBranch\":\"main\"", "\"gitBranch\":\"alpha\""))
            .collect()
    };
    // alpha: two sessions of Sonnet advised by Opus, one with a Haiku sub-agent, then one of Opus alone: mixed.
    write(
        &projects.join("-proj/sa.jsonl"),
        &on_alpha(vec![
            modeled(&repo, "2026-01-02T22:00:00Z", "a1", sonnet, opus),
            modeled(&repo, "2026-01-03T01:00:00Z", "a2", sonnet, opus),
        ]),
    );
    write(
        &projects.join("-proj/sb.jsonl"),
        &on_alpha(vec![
            modeled(&repo, "2026-01-03T22:00:00Z", "b1", sonnet, opus),
            modeled(&repo, "2026-01-04T01:00:00Z", "b2", sonnet, opus),
        ]),
    );
    write(
        &projects.join("-proj/sb/subagents/agent-1.jsonl"),
        &[modeled(
            &repo,
            "2026-01-03T23:00:00Z",
            "b3",
            "claude-haiku-5-5",
            "",
        )],
    );
    write(
        &projects.join("-proj/sc.jsonl"),
        &on_alpha(vec![
            modeled(&repo, "2026-01-04T23:00:00Z", "c1", opus, ""),
            modeled(&repo, "2026-01-05T01:00:00Z", "c2", opus, ""),
        ]),
    );
    // beta: one session of Sonnet advised by Opus holds its close commit.
    write(
        &projects.join("-proj/sd.jsonl"),
        &[
            modeled(&repo, "2026-01-04T11:00:00Z", "d1", sonnet, opus),
            modeled(&repo, "2026-01-04T13:00:00Z", "d2", sonnet, opus),
        ],
    );
    // in r3, epsilon has a session of Opus with an Opus advisor set, alone all the same, after a failed request, and delta none: two rows of the summary for one revision; gamma, left in
    // the work tree, would be closed with them.
    std::fs::remove_file(repo.join("issues/done/gamma.md")).unwrap();
    commit_at(
        &repo,
        "2026-03-01T00:00:00Z",
        "docs: close epsilon",
        &[("issues/done/epsilon.md", Some("# epsilon\n\n# Tracking\n"))],
    );
    write(
        &projects.join("-proj/se.jsonl"),
        &[
            modeled(&repo, "2026-02-28T22:00:00Z", "e0", "<synthetic>", ""),
            modeled(&repo, "2026-02-28T23:00:00Z", "e1", opus, opus),
            modeled(&repo, "2026-03-01T01:00:00Z", "e2", opus, ""),
        ],
    );
    commit_at(
        &repo,
        "2026-03-02T00:00:00Z",
        "docs: close delta",
        &[("issues/done/delta.md", Some("# delta\n\n# Tracking\n"))],
    );
    (repo, projects)
}

// spec: QUA-16, QUA-17
fn setup_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let (repo, projects) = setup_fixture(&tmp.join("setups"));
    let projects_arg = projects.to_str().unwrap();
    let (ok, out) = run(dir, &repo, &["--projects", projects_arg]);
    let r = rows(&out);
    let (_, summed) = run(dir, &repo, &["--projects", projects_arg, "--summary"]);
    let heads: Vec<String> = summed
        .lines()
        .map(|l| {
            let cells: Vec<&str> = l.split('\t').collect();
            // revision, setup and issues: no issue of this fixture is in a milestone
            [cells[0], cells[1], cells[3]].join("|")
        })
        .collect();
    let (_, none) = run(dir, &repo, &[]);
    vec![
        ("the setups are read".to_string(), ok),
        (
            "an issue made under several setups is mixed".to_string(),
            has(&r, "alpha", "setup", "mixed") && has(&r, "alpha", "sessions", "3"),
        ),
        (
            "an issue made under one setup gives it, the main model first".to_string(),
            has(
                &r,
                "beta",
                "setup",
                "claude-sonnet-5-5 advised by claude-opus-5-5",
            ),
        ),
        (
            "an issue with no session has no setup".to_string(),
            has(&r, "delta", "setup", "") && has(&r, "epsilon", "setup", "claude-opus-5-5 solo"),
        ),
        (
            "the setup is empty when the projects cannot be read".to_string(),
            has(&rows(&none), "beta", "setup", ""),
        ),
        (
            "the summary has one row per revision and setup, the empty setup first".to_string(),
            heads
                == [
                    "revision|setup|issues",
                    "r1|mixed|1",
                    "r2|claude-sonnet-5-5 advised by claude-opus-5-5|1",
                    "r3||1",
                    "r3|claude-opus-5-5 solo|1",
                ],
        ),
    ]
}

// spec: QUA-9, QUA-H2
fn summary_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("summary");
    let repo = fixture(&tmp);
    // gamma, closed in the revision of beta, makes r2 a sum of two issues; its copy left in the work tree goes first.
    std::fs::remove_file(repo.join("issues/done/gamma.md")).unwrap();
    commit_at(
        &repo,
        "2026-02-01T00:00:00Z",
        "chore: unpin back",
        &[(".claude/harness.toml", Some(&toml("r2")))],
    );
    commit_at(
        &repo,
        "2026-02-02T00:00:00Z",
        "feat(files): gamma",
        &[
            (
                "issues/done/gamma.md",
                Some("# gamma\n\n# Tracking\n\n1. [x] one\n- Code review: 2 findings\n"),
            ),
            ("src/g.rs", Some("g")),
        ],
    );
    // delta, in a revision named before the others, comes last: the revisions are in the order they were used.
    commit_at(
        &repo,
        "2026-02-03T00:00:00Z",
        "chore: pin",
        &[(".claude/harness.toml", Some(&toml("a0")))],
    );
    commit_at(
        &repo,
        "2026-02-04T00:00:00Z",
        "feat(files): delta",
        &[(
            "issues/done/delta.md",
            Some("# delta\n\n# Tracking\n\n1. [x] one\n"),
        )],
    );
    let (ok, out) = run(dir, &repo, &["--summary"]);
    let expected = [
        "revision\tsetup\tmilestone\tissues\tdeliverables\tcommits\tfixes_in_issue\tfixes_after\treverts_after\treviews\tcounted_reviews\treview_findings\ttraced\ttraced_without_red\tspec_lines\tdoc_lines\tspecs\tspec_bytes\tspec_requirements\tspec_intent_chars\tspec_happy_paths\tsessions\tfailed_checks\tvalidations\tother_messages\tinput_tokens\tcache_read_tokens\toutput_tokens\ttoken_issues",
        "r1\t\t\t1\t2\t3\t1\t2\t3\t3\t3\t7\t0\t0\t0\t1\t0\t0\t0\t0\t0\t\t\t\t\t0\t0\t0\t0",
        "r2\t\t\t2\t3\t2\t0\t0\t0\t3\t2\t4\t0\t0\t0\t0\t0\t0\t0\t0\t0\t\t\t\t\t0\t0\t0\t0",
        "a0\t\t\t1\t1\t1\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t\t\t\t\t0\t0\t0\t0",
    ];
    let lines: Vec<&str> = out.lines().collect();
    if lines != expected {
        println!("{out}");
    }
    vec![("the summary sums the columns per revision in the order they were used, an empty value leaving its sum empty".to_string(), ok && lines == expected)]
}

#[test]
fn delivered_quality_sessions() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    let checks: Vec<_> = session_checks(&dir, &tmp)
        .into_iter()
        .chain(claim_checks(&dir, &tmp))
        .chain(summary_checks(&dir, &tmp))
        .chain(setup_checks(&dir, &tmp))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
