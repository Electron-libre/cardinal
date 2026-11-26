//! Behavior test for `cardinal tool rule-breaches` on fixture transcripts and repositories: the findings of each rule,
//! `breach` apart from `blocked`, their locations, the summary per rule and revision, no message text. The commit rules
//! are in `rule_breaches_commits.rs`.

mod rule_breaches_common;

use rule_breaches_common::*;
use serde_json::json;
use std::path::{Path, PathBuf};

/// Writes the session `a` of `repo` (and its subagent) under `proj`; returns its expected finding rows.
fn session_a(repo: &Path, proj: &Path) -> Vec<String> {
    let in_repo = |p: &str| repo.join(p);
    let mut t = Transcript::new(repo, "2026-02-01T10:00:00.000Z");
    t.typed("please SECRET");
    let no_verify = t.run("t1", "Bash", &bash("git commit --no-verify -m SECRET"), Some("PreToolUse:Bash hook error: [x/require_code_review.rs check]: Refused: --no-verify skips the gate."));
    let refused1 = t.call(
        "t2",
        "Bash",
        &bash("git commit -m \"SECRET mentions --no-verify\""),
    );
    t.result(
        "t2",
        true,
        "Refused: the last code review predates the last commit.",
    );
    t.run(
        "t3",
        "Skill",
        &json!({"skill": "code-review", "args": "low"}),
        None,
    );
    t.run(
        "t4",
        "Bash",
        &bash("git commit -q -F - <<'EOF'\nfeat: SECRET\n\n--no-verify\nEOF\n"),
        None,
    );
    t.call("t5", "Bash", &bash("git commit -m x"));
    let refused2 = t.result("t5", true, "Refused: the diff holds files the last code review never saw. Or ask the user to waive the review.");
    let bypass = t.run("t6", "Bash", &bash("git add a && git commit -m y"), None);
    let denied = Some("Permission to use Bash has been denied.");
    let waive = t.run("t7", "Bash", &bash("just waive-review"), denied);
    let waive_tool = t.run("t7b", "Bash", &bash("cardinal run waive-review"), denied);
    let waiver_file = t.run(
        "t8",
        "Write",
        &json!({"file_path": in_repo(".git/claude-review-waived"), "content": ""}),
        None,
    );
    // a line that only names the waiver runs none
    let names = "touch somewhere && cardinal run check-specs && grep -rn waive-review docs";
    t.call("t9", "Bash", &bash(names));
    let waive_refused = t.result(
        "t9",
        true,
        "PreToolUse:Bash hook error: [x]: Refused: only the user waives the code review.",
    );
    let managed = t.run("t10", "Edit", &edit(&in_repo(".claude/hooks/x.rs")), None);
    t.run(
        "t11",
        "Edit",
        &edit(&in_repo("files/.claude/hooks/x.rs")),
        None,
    );
    t.run(
        "t12",
        "Write",
        &json!({"file_path": in_repo("issues/a.md"), "content": "SECRET\n# Tracking\n\nPlan: proposed\n"}),
        None,
    );
    t.run("t13", "Edit", &edit(&in_repo("specs/a.md")), None);
    t.run("t14", "Edit", &edit(&in_repo("docs/adr/0001-a.md")), None);
    let early = t.run("t15", "Edit", &edit(&in_repo("src/main.rs")), None);
    t.run(
        "t16",
        "Edit",
        &edit(Path::new("/elsewhere/memory.md")),
        None,
    );
    t.typed("ok");
    t.run("t17", "Edit", &edit(&in_repo("src/lib.rs")), None);
    t.write(&proj.join("a.jsonl"));
    let mut sub = Transcript::new(repo, "2026-02-01T10:30:00.000Z");
    let pushed = sub.run("s1", "Bash", &bash("git push --no-verify"), None);
    sub.write(&proj.join("a/subagents/agent-x.jsonl"));

    let a = |rule: &str, kind: &str, n: usize, detail: &str| {
        row("a", "r1", rule, kind, &format!("a.jsonl:{n}"), detail)
    };
    vec![
        a("no-verify", "blocked", no_verify, ""),
        a("review-before-commit", "blocked", refused1 + 1, ""),
        a("review-before-commit", "blocked", refused2, ""),
        a("review-before-commit", "breach", bypass, ""),
        a("review-waiver", "blocked", waive, ""),
        a("review-waiver", "blocked", waive_tool, ""),
        a("review-waiver", "breach", waiver_file, ""),
        a("review-waiver", "blocked", waive_refused, ""),
        a("managed-copy", "breach", managed, ".claude/hooks/x.rs"),
        a("validation-stop", "breach", early, "src/main.rs"),
        row(
            "a",
            "r1",
            "no-verify",
            "breach",
            &format!("subagents/agent-x.jsonl:{pushed}"),
            "",
        ),
    ]
}

/// Writes the sessions `b` to `h` of `repo` (and `c` of `bare`) under `projects`; returns their expected finding rows.
#[allow(clippy::many_single_char_names)] // each transcript is named after its file, `b` writes `b.jsonl`
fn other_sessions(tmp: &Path, repo: &Path, bare: &Path, projects: &Path) -> Vec<String> {
    let proj = projects.join("-proj");
    let in_repo = |p: &str| repo.join(p);
    let mut b = Transcript::new(repo, "2026-02-02T10:00:00.000Z");
    b.typed("hello");
    b.run("t1", "Edit", &edit(&in_repo("src/x.rs")), None);
    b.run(
        "t2",
        "Write",
        &json!({"file_path": in_repo("tests/t.rs"), "content": "SECRET \"Plan: proposed\""}),
        None,
    );
    b.run("t3", "Edit", &edit(&in_repo("src/y.rs")), None);
    b.run("t4", "Bash", &bash("python3 edit.py"), Some("PreToolUse:Bash hook error: [x]: Refused: the diff changed since the last code review."));
    b.run("t5", "Bash", &bash("git commit -m z"), None);
    b.write(&proj.join("b.jsonl"));

    let mut c = Transcript::new(bare, "2026-02-02T10:00:00.000Z");
    c.run("t1", "Bash", &bash("git commit --no-verify -m x"), None);
    c.write(&projects.join("-bare/c.jsonl"));

    let mut d = Transcript::new(repo, "2025-12-01T10:00:00.000Z");
    d.run("t1", "Bash", &bash("git commit --no-verify -m x"), None);
    d.write(&proj.join("d.jsonl"));

    let mut g = Transcript::new(repo, "2026-02-03T10:00:00.000Z");
    g.call("t1", "Bash", &bash("git commit -m x"));
    let refused3 = g.result(
        "t1",
        true,
        "Refused: no code review has run in this project.",
    );
    g.typed("<command-name>/code-review</command-name>\n<command-args>low</command-args>");
    g.run("t2", "Bash", &bash("git commit -m x"), None);
    g.write(&proj.join("g.jsonl"));

    let linked = tmp.join("proj-foo");
    sh(
        repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "foo",
            linked.to_str().unwrap(),
        ],
    );
    let mut h = Transcript::new(repo, "2026-02-04T10:00:00.000Z");
    h.typed("start the issue");
    h.cwd = linked.display().to_string();
    let switched = h.run(
        "t1",
        "Edit",
        &edit(&linked.join(".claude/hooks/x.rs")),
        None,
    );
    let quoted = h.run(
        "t2",
        "Bash",
        &bash("echo \"a << b\"\ngit commit --no-verify -m x"),
        None,
    );
    h.write(&proj.join("h.jsonl"));

    let mut e = Transcript::new(repo, "2026-03-02T10:00:00.000Z");
    let late = e.run("t1", "Bash", &bash("git -C x commit --no-verify"), None);
    e.write(&proj.join("e.jsonl"));

    vec![
        row(
            "e",
            "r2",
            "no-verify",
            "breach",
            &format!("e.jsonl:{late}"),
            "",
        ),
        row(
            "g",
            "r1",
            "review-before-commit",
            "blocked",
            &format!("g.jsonl:{refused3}"),
            "",
        ),
        row(
            "h",
            "r1",
            "managed-copy",
            "breach",
            &format!("h.jsonl:{switched}"),
            ".claude/hooks/x.rs",
        ),
        row(
            "h",
            "r1",
            "no-verify",
            "breach",
            &format!("h.jsonl:{quoted}"),
            "",
        ),
    ]
}

/// Builds the fixture; returns the projects directory and the expected finding rows (without header).
fn fixture(tmp: &Path) -> (PathBuf, Vec<String>) {
    let repo = new_repo(&tmp.join("proj"));
    let toml = |rev: &str| {
        format!("[harness]\nsource = \"https://example.org/harness\"\nrev = \"{rev}\"\n")
    };
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[
            (".claude/harness.toml", &toml("r1")),
            (
                ".claude/harness.lock",
                "abc+x  .claude/hooks/x.rs\ndef  AGENTS.md\n",
            ),
        ],
    );
    commit_at(
        &repo,
        "2026-03-01T00:00:00Z",
        "chore: bump",
        &[(".claude/harness.toml", &toml("r2"))],
    );
    let bare = new_repo(&tmp.join("bare"));
    commit_at(&bare, "2026-01-01T00:00:00Z", "init", &[("README", "x")]);
    let projects = tmp.join("projects");
    let mut expected = session_a(&repo, &projects.join("-proj"));
    expected.extend(other_sessions(tmp, &repo, &bare, &projects));
    (projects, expected)
}

// spec: BRE-1, BRE-2, BRE-4, BRE-5, BRE-6, BRE-7, BRE-8, BRE-11, BRE-H1
fn finding_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let (projects, expected) = fixture(tmp);
    let (ok, out) = run(dir, &projects, false);
    let mut lines = out.lines();
    let header = lines.next().unwrap_or_default().to_string();
    let rows: Vec<String> = lines.map(str::to_string).collect();
    let mut checks = vec![
        ("the script succeeds".to_string(), ok),
        (
            "the header names the columns".to_string(),
            header == COLUMNS,
        ),
        (
            "no text of a message, a tool input or a tool result is printed".to_string(),
            !out.contains("SECRET"),
        ),
        (
            "no finding for a project without harness, nor before its harness".to_string(),
            !out.contains("\tbare\t") && !out.contains("d.jsonl"),
        ),
        (
            "no finding beyond the expected ones".to_string(),
            sorted(rows.clone()) == sorted(expected.clone()),
        ),
    ];
    for want in expected {
        checks.push((format!("finds {want}"), rows.contains(&want)));
    }
    if !checks.iter().all(|(_, ok)| *ok) {
        println!("{out}");
    }
    checks
}

// spec: BRE-3, BRE-H2
fn summary_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let (projects, _) = fixture(&tmp.join("summary"));
    let (ok, out) = run(dir, &projects, true);
    let expected = [
        SUMMARY,
        "managed-copy\tr1\t3\t2\t2\t0\t",
        "no-verify\tr1\t4\t2\t2\t1\t",
        "no-verify\tr2\t1\t1\t1\t0\t",
        "oversized-result\tr1\t4\t0\t0\t0\t0",
        "oversized-result\tr2\t1\t0\t0\t0\t0",
        "oversized-result-subagent\tr1\t1\t0\t0\t0\t0",
        "review-before-commit\tr1\t4\t1\t1\t3\t",
        "review-before-commit\tr2\t1\t0\t0\t0\t",
        "review-waiver\tr1\t4\t1\t1\t3\t",
        "review-waiver\tr2\t1\t0\t0\t0\t",
        "scratch-outside-tmpdir\tr1\t4\t0\t0\t0\t",
        "scratch-outside-tmpdir\tr2\t1\t0\t0\t0\t",
        "stage-everything\tr1\t1\t0\t0\t0\t",
        "validation-stop\tr1\t1\t1\t1\t0\t",
    ];
    let lines: Vec<&str> = out.lines().collect();
    if lines != expected {
        println!("{out}");
    }
    vec![(
        "the summary counts per rule and revision, sorted".to_string(),
        ok && lines == expected,
    )]
}

// spec: BRE-23
fn project_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let (projects, expected) = fixture(&tmp.join("project"));
    let with = |args: &[&str]| {
        let out = tool_command(dir.join("rule_breaches.rs"))
            .arg(&projects)
            .args(args)
            .output()
            .unwrap();
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
        )
    };
    let (_, all) = with(&["--summary"]);
    let (own_ok, own) = with(&["--summary", "--project", "proj"]);
    let (other_ok, other) = with(&["--project", "bare"]);
    let (_, other_summary) = with(&["--project", "bare", "--summary"]);
    let (refused, _) = with(&["--project"]);
    let (flag_refused, _) = with(&["--project", "--summary"]);
    let unknown = tool_command(dir.join("rule_breaches.rs"))
        .arg(&projects)
        .args(["--project", "nope"])
        .output()
        .unwrap();
    vec![
        (
            "--project keeps the sessions of that project in the summary".to_string(),
            own_ok && !expected.is_empty() && own == all,
        ),
        (
            "--project leaves out the findings and the summary rows of the other projects"
                .to_string(),
            other_ok && other.trim() == COLUMNS && other_summary.trim() == SUMMARY,
        ),
        (
            "--project with no value, or an option as its value, is refused".to_string(),
            !refused && !flag_refused,
        ),
        (
            "--project naming no project of the sessions warns".to_string(),
            unknown.status.success() && String::from_utf8_lossy(&unknown.stderr).contains("nope"),
        ),
    ]
}

const FRENCH: &str = "Le livrable est fait : les tests passent et la revue ne trouve rien, donc je committe puis je passe au suivant dans la foulée.";
const ENGLISH: &str = "The deliverable is done: the tests pass and the review finds nothing, so I commit it and move on to the next one with the plan.";
/// French with no French stop word and many `an` and `as`, also French words: an English block for the old lists.
const AN_AS: &str = "Bilan : an premier, an deuxième, an troisième, an quatrième, an cinquième, an sixième ; as gagnant, as perdant, as volant, as rouge, as noir, as vert.";

// spec: BRE-10
fn language_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("language");
    let projects = tmp.join("projects");
    let mut expected = Vec::new();
    for (name, language) in [
        ("fr", Some(" french ")),
        ("de", Some("German")),
        ("none", None),
    ] {
        let repo = new_repo(&tmp.join(name));
        let key = language.map_or_default(|l| format!("user_language = \"{l}\"\n"));
        commit_at(
            &repo,
            "2026-01-01T00:00:00Z",
            "chore: sync",
            &[(
                ".claude/harness.toml",
                &format!("[harness]\nsource = \"x\"\nrev = \"r1\"\n\n[project]\n{key}"),
            )],
        );
        let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
        t.typed("SECRET");
        t.text("m1", FRENCH);
        t.text("m7", AN_AS);
        let english = t.text("m2", ENGLISH);
        t.text("m3", "The tests pass, so I commit it and move on.");
        t.text(
            "m4",
            &format!("Voici le diff :\n```\n{ENGLISH}\n```\net `{ENGLISH}` fini."),
        );
        t.text("m8", &format!("{FRENCH} ``{ENGLISH} {ENGLISH} ` x``"));
        // a code span runs over the line endings of its paragraph
        t.text("m9", &format!("{FRENCH}\n`{ENGLISH}\n{ENGLISH}` fini."));
        t.text(
            "m6",
            &format!("Voici la sortie :\n   ~~~\n{ENGLISH}\n   ~~~\nfini."),
        );
        // an indented block and a fenced block inside a quote are code, as in CommonMark
        t.text("m10", &format!("{FRENCH}\n\n    {ENGLISH}\n    {ENGLISH}"));
        t.text("m11", &format!("{FRENCH}\n\n> ```\n> {ENGLISH}\n> ```"));
        // the backtick of the line that closes an HTML block is HTML: it pairs with none after it
        let html = t.text(
            "m12",
            &format!("{FRENCH}\n<!--\n--> `\n{ENGLISH} {ENGLISH}` fini."),
        );
        t.write(&projects.join(format!("-{name}/s.jsonl")));
        let mut sub = Transcript::new(&repo, "2026-02-01T10:01:00.000Z");
        sub.text("m5", ENGLISH);
        sub.write(&projects.join(format!("-{name}/s/subagents/agent-y.jsonl")));
        if name == "fr" {
            expected.push(format!(
                "s\tfr\tr1\tuser-language\tbreach\ts.jsonl:{english}\tintermediate"
            ));
            expected.push(format!(
                "s\tfr\tr1\tuser-language\tbreach\ts.jsonl:{html}\tfinal"
            ));
        }
    }
    let (ok, out) = run(dir, &projects, false);
    let rows: Vec<String> = out.lines().skip(1).map(str::to_string).collect();
    let (_, summary) = run(dir, &projects, true);
    let summary: Vec<&str> = summary.lines().collect();
    let want_summary = [SUMMARY, "user-language\tr1\t1\t1\t2\t0\t"];
    if rows != expected || summary != want_summary {
        println!("{out}{}", summary.join("\n"));
    }
    vec![
        ("the language rule succeeds".to_string(), ok && !out.contains("SECRET") && !out.contains("deliverable")),
        ("a long block of the session in the other language is a breach; short, code, sub-agent blocks and French with `an` and `as` are not".to_string(), rows == expected),
        ("the language rule applies only with a French or English user_language, in any case".to_string(), summary == want_summary),
    ]
}

// spec: BRE-14
fn personal_language_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("personal");
    let projects = tmp.join("projects");
    let mut expected = Vec::new();
    for (name, personal) in [("en", "English"), ("de", "German")] {
        let repo = new_repo(&tmp.join(name));
        commit_at(
            &repo,
            "2026-01-01T00:00:00Z",
            "chore: sync",
            &[(
                ".claude/cardinal.toml",
                "[cardinal]\nsource = \"x\"\nrev = \"r1\"\n\n[project]\nuser_language = \"French\"\n",
            )],
        );
        // written after the session's start, never committed: read as it is when the measure runs
        std::fs::write(
            repo.join(".claude/cardinal.local.toml"),
            format!("[project]\nuser_language = \"{personal}\"\n"),
        )
        .unwrap();
        let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
        t.typed("go");
        t.text("m1", ENGLISH);
        let french = t.text("m2", FRENCH);
        t.write(&projects.join(format!("-{name}/s.jsonl")));
        if name == "en" {
            expected.push(format!(
                "s\ten\tr1\tuser-language\tbreach\ts.jsonl:{french}\tfinal"
            ));
        }
    }
    let (ok, out) = run(dir, &projects, false);
    let rows: Vec<String> = out.lines().skip(1).map(str::to_string).collect();
    let (_, summary) = run(dir, &projects, true);
    let summary: Vec<&str> = summary.lines().collect();
    let want_summary = [SUMMARY, "user-language\tr1\t1\t1\t1\t0\t"];
    if rows != expected || summary != want_summary {
        println!("{out}{}", summary.join("\n"));
    }
    vec![
        ("the personal language rule succeeds".to_string(), ok),
        (
            "the personal user_language wins over the committed one".to_string(),
            rows == expected,
        ),
        (
            "a personal language with no stop words leaves the session unjudged".to_string(),
            summary == want_summary,
        ),
    ]
}

const HINT: &str =
    "Your text is not in French, the user_language of .claude/cardinal.toml: write in French.";

// spec: BRE-13
fn language_kind_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("kinds");
    let projects = tmp.join("projects");
    let repo = new_repo(&tmp.join("k"));
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(
            ".claude/cardinal.toml",
            "[cardinal]\nsource = \"x\"\nrev = \"r1\"\n\n[project]\nuser_language = \"French\"\n",
        )],
    );
    let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
    t.typed("go");
    let hinted_tool = t.text("e1", ENGLISH);
    t.push(json!({"type": "attachment", "attachment": {"type": "hook_additional_context", "content": [HINT], "hookName": "PostToolUse:Bash"}}));
    t.text("f1", FRENCH);
    t.typed("go");
    let intermediate = t.text("e2", ENGLISH);
    t.text("f2", "Fini.");
    t.typed("go");
    let last = t.text("e3", ENGLISH);
    t.typed("go");
    let hinted_stop = t.text("e4", ENGLISH);
    t.push(json!({"type": "user", "isMeta": true, "message": {"role": "user", "content": format!("Stop hook feedback:\n[cardinal run language-hint]: {HINT}")}}));
    let end = t.text("e5", ENGLISH);
    // each hint covers the run of blocks it judged: the blocks of later calls in the turn are not hinted
    t.typed("go");
    let first_run = t.text("e6", ENGLISH);
    t.run("c1", "Read", &json!({}), None);
    t.push(json!({"type": "attachment", "attachment": {"type": "hook_additional_context", "content": [HINT], "hookName": "PostToolUse:Read"}}));
    let second_run = t.text("e7", ENGLISH);
    t.run("c2", "Read", &json!({}), None);
    let third_run = t.text("e8", ENGLISH);
    t.run("c3", "Read", &json!({}), None);
    // a meta line that is not Stop hook feedback is no hint, even holding its words
    t.push(json!({"type": "user", "isMeta": true, "message": {"role": "user", "content": format!("Read {HINT}")}}));
    let reply = t.text("e9", ENGLISH);
    t.push(json!({"type": "user", "isMeta": true, "message": {"role": "user", "content": format!("Stop hook feedback:\n[cardinal run language-hint]: {HINT}")}}));
    t.write(&projects.join("-k/s.jsonl"));
    let row = |line: usize, kind: &str| {
        format!("s\tk\tr1\tuser-language\tbreach\ts.jsonl:{line}\t{kind}")
    };
    let expected = vec![
        row(hinted_tool, "hinted"),
        row(intermediate, "intermediate"),
        row(last, "final"),
        row(hinted_stop, "hinted"),
        row(end, "final"),
        row(first_run, "hinted"),
        row(second_run, "intermediate"),
        row(third_run, "intermediate"),
        row(reply, "hinted"),
    ];
    let (ok, out) = run(dir, &projects, false);
    let rows: Vec<String> = out.lines().skip(1).map(str::to_string).collect();
    if rows != expected {
        println!("{out}");
    }
    vec![
        ("the language kinds succeed".to_string(), ok),
        ("a language breach is hinted when the hook's hint follows it in the turn, else final or intermediate".to_string(), rows == expected),
    ]
}

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

// spec: BRE-19, BRE-20, BRE-22
fn waste_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("waste");
    let projects = tmp.join("projects");
    let repo = new_repo(&tmp.join("w"));
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(
            ".claude/cardinal.toml",
            "[cardinal]\nsource = \"x\"\nrev = \"r1\"\n",
        )],
    );
    let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
    t.typed("go");
    // results over 25,000 bytes, a string or parts joined by newlines (the newline tips this one over), an error too
    t.call("o1", "Bash", &bash("cat big"));
    t.result("o1", false, &format!("{}SECR", "SECRET".repeat(4_166)));
    t.call("o2", "Bash", &bash("cat bigger"));
    let oversized = t.result("o2", true, &format!("{}SECRE", "SECRET".repeat(4_166)));
    t.call("o3", "Read", &json!({"file_path": repo.join("d.rs")}));
    let parts = t.push(json!({"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "o3", "content": [{"type": "text", "text": "SECRET".repeat(2_500)}, {"type": "text", "text": format!("{}SECR", "SECRET".repeat(1_666))}]}]}}));
    t.write(&projects.join("-w/s.jsonl"));
    // a reviewer of `/code-review` (no description), an `Explore` sub-agent, and one with no `.meta.json`
    let subagents = projects.join("-w/s/subagents");
    let mut subs = Vec::new();
    for (name, meta) in [
        (
            "agent-1",
            Some(r#"{"agentType":"general-purpose","isBuiltIn":true}"#),
        ),
        (
            "agent-2",
            Some(r#"{"agentType":"Explore","description":"find the parser"}"#),
        ),
        ("agent-3", None),
    ] {
        let mut sub = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
        sub.call("x1", "Bash", &bash("cat big"));
        subs.push(sub.result("x1", false, &"SECRET".repeat(5_000)));
        sub.write(&subagents.join(format!("{name}.jsonl")));
        if let Some(meta) = meta {
            std::fs::write(subagents.join(format!("{name}.meta.json")), meta).unwrap();
        }
    }
    let row = |rule: &str, location: String, detail: &str| {
        format!("s\tw\tr1\t{rule}\tbreach\t{location}\t{detail}")
    };
    let expected = vec![
        row("oversized-result", format!("s.jsonl:{oversized}"), "25001"),
        row("oversized-result", format!("s.jsonl:{parts}"), "25001"),
        row(
            "oversized-result-review",
            format!("subagents/agent-1.jsonl:{}", subs[0]),
            "30000",
        ),
        row(
            "oversized-result-subagent",
            format!("subagents/agent-2.jsonl:{}", subs[1]),
            "30000",
        ),
        row(
            "oversized-result-subagent",
            format!("subagents/agent-3.jsonl:{}", subs[2]),
            "30000",
        ),
    ];
    let (ok, out) = run(dir, &projects, false);
    let rows: Vec<String> = out.lines().skip(1).map(str::to_string).collect();
    let (_, summary) = run(dir, &projects, true);
    let summary: Vec<&str> = summary.lines().skip(1).collect();
    let want_summary = [
        "oversized-result\tr1\t1\t1\t2\t0\t50002",
        "oversized-result-review\tr1\t1\t1\t1\t0\t30000",
        "oversized-result-subagent\tr1\t1\t1\t2\t0\t60000",
        "scratch-outside-tmpdir\tr1\t1\t0\t0\t0\t",
    ];
    if sorted(rows.clone()) != sorted(expected.clone()) || summary != want_summary {
        println!("{out}{}", summary.join("\n"));
    }
    vec![
        (
            "the waste rules succeed".to_string(),
            ok && !out.contains("SECRET"),
        ),
        (
            "a tool result over 25,000 bytes is a breach at its line, of the rule of its transcript's source"
                .to_string(),
            sorted(rows) == sorted(expected),
        ),
        (
            "each waste rule applies to the sessions with a tool result of its source and sums its bytes"
                .to_string(),
            summary == want_summary,
        ),
    ]
}

// spec: BRE-21
fn stage_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("stage");
    let projects = tmp.join("projects");
    let repo = new_repo(&tmp.join("g"));
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(
            ".claude/cardinal.toml",
            "[cardinal]\nsource = \"x\"\nrev = \"r1\"\n",
        )],
    );
    let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
    t.typed("go");
    let chained = t.run(
        "a1",
        "Bash",
        &bash("git add -A && git commit -m 'SECRET'"),
        None,
    );
    t.run(
        "a2",
        "Bash",
        &bash("git add src/a.rs && git commit -m 'git add -A .'"),
        None,
    );
    let in_repo = t.run(
        "a3",
        "Bash",
        &bash("git -C repo add ."),
        Some("SECRET refused"),
    );
    let long = t.run("a4", "Bash", &bash("git add --all; ls ."), None);
    t.run("a5", "Bash", &bash("git commit -am x"), None);
    t.run("a7", "Bash", &bash("git add a.rs && ls ."), None);
    t.run("a6", "Bash", &bash("git status . && git add b.rs"), None);
    t.write(&projects.join("-g/s.jsonl"));
    let mut other = Transcript::new(&repo, "2026-02-02T10:00:00.000Z");
    other.typed("go");
    let alone = other.run("b1", "Bash", &bash("git add -A"), None);
    other.write(&projects.join("-g/t.jsonl"));
    let row = |s: &str, kind: &str, line: usize| {
        format!("{s}\tg\tr1\tstage-everything\t{kind}\t{s}.jsonl:{line}\t")
    };
    let expected = vec![
        row("s", "breach", chained),
        row("s", "blocked", in_repo),
        row("s", "breach", long),
        row("t", "breach", alone),
    ];
    let (ok, out) = run(dir, &projects, false);
    let rows: Vec<String> = out
        .lines()
        .filter(|l| l.contains("\tstage-everything\t"))
        .map(str::to_string)
        .collect();
    let (_, summary) = run(dir, &projects, true);
    let summary: Vec<&str> = summary
        .lines()
        .filter(|l| l.starts_with("stage-everything\t"))
        .collect();
    let want_summary = ["stage-everything\tr1\t2\t2\t3\t1\t"];
    if sorted(rows.clone()) != sorted(expected.clone()) || summary != want_summary {
        println!("{out}{}", summary.join("\n"));
    }
    vec![
        (
            "the staging rule succeeds".to_string(),
            ok && !out.contains("SECRET"),
        ),
        (
            "a git add of -A, --all or . is a finding, not one of named files nor a quoted one"
                .to_string(),
            sorted(rows) == sorted(expected),
        ),
        (
            "the staging rule applies to the sessions with a git add".to_string(),
            summary == want_summary,
        ),
    ]
}

// spec: FEA-26
fn gate_off_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("gate-off");
    let repo = new_repo(&tmp.join("off"));
    let config = "[cardinal]\nsource = \"x\"\nrev = \"r1\"\ndisabled = [\"review-gate\"]\n";
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/cardinal.toml", config)],
    );
    let projects = tmp.join("projects");
    let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
    t.typed("go");
    t.run("a1", "Bash", &bash("git add -A"), None);
    t.run(
        "a2",
        "Bash",
        &bash("git commit -m x"),
        Some("Refused: run a code review first"),
    );
    t.run("a3", "Bash", &bash("git commit --no-verify -m x"), None);
    t.run("a4", "Bash", &bash("just waive-review"), None);
    t.run(
        "a5",
        "Write",
        &json!({"file_path": repo.join(".git/claude-review-waived"), "content": "SECRET"}),
        Some("Refused: only the user waives the review"),
    );
    t.write(&projects.join("-off/s.jsonl"));
    let (ok, out) = run(dir, &projects, false);
    let (_, summary) = run(dir, &projects, true);
    let rules = [
        "no-verify",
        "stage-everything",
        "review-waiver",
        "review-before-commit",
    ];
    let quiet = |text: &str| !rules.iter().any(|r| text.contains(&format!("{r}\t")));
    if !quiet(&out) || !quiet(&summary) {
        println!("{out}{summary}");
    }
    // the session is read: the waste rule applies to it
    vec![(
        "a session of a project that turned off review-gate is not checked against its rules"
            .to_string(),
        ok && quiet(&out) && quiet(&summary) && summary.contains("oversized-result\t"),
    )]
}

// spec: FEA-38
fn issue_files_off_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let tmp = tmp.join("issue-files-off");
    let repo = new_repo(&tmp.join("off"));
    let config = "[cardinal]\nsource = \"x\"\nrev = \"r1\"\ndisabled = [\"issue-files\"]\n";
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/cardinal.toml", config)],
    );
    let projects = tmp.join("projects");
    let mut t = Transcript::new(&repo, "2026-02-01T10:00:00.000Z");
    t.typed("go");
    t.run(
        "p1",
        "Write",
        &json!({"file_path": repo.join("issues/x.md"), "content": "# Tracking\n\nPlan: proposed\n"}),
        None,
    );
    t.run(
        "p2",
        "Write",
        &json!({"file_path": repo.join("src/main.rs"), "content": "fn main() {}\n"}),
        None,
    );
    t.write(&projects.join("-off/s.jsonl"));
    let (ok, out) = run(dir, &projects, false);
    let (_, summary) = run(dir, &projects, true);
    let quiet = |text: &str| !text.contains("validation-stop\t");
    if !quiet(&out) || !quiet(&summary) {
        println!("{out}{summary}");
    }
    vec![(
        "a session of a project that turned off issue-files is not checked against validation-stop"
            .to_string(),
        // the session is read: the waste rule applies to it
        ok && quiet(&out) && quiet(&summary) && summary.contains("oversized-result\t"),
    )]
}

#[test]
fn rule_breaches() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    let checks: Vec<_> = finding_checks(&dir, &tmp)
        .into_iter()
        .chain(summary_checks(&dir, &tmp))
        .chain(language_checks(&dir, &tmp))
        .chain(language_kind_checks(&dir, &tmp))
        .chain(personal_language_checks(&dir, &tmp))
        .chain(routing_checks(&dir, &tmp))
        .chain(waste_checks(&dir, &tmp))
        .chain(stage_checks(&dir, &tmp))
        .chain(gate_off_checks(&dir, &tmp))
        .chain(issue_files_off_checks(&dir, &tmp))
        .chain(project_checks(&dir, &tmp))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
