//! Behavior test for rule `user-language` of `cardinal tool rule-breaches` (BRE-10, BRE-13, BRE-14): the replies not in
//! the `user_language` of the project, the personal one first, `intermediate` apart from `final`.

mod rule_breaches_common;

use rule_breaches_common::*;
use serde_json::json;
use std::path::Path;

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

#[test]
fn rule_breaches_language() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    let checks: Vec<_> = language_checks(&dir, &tmp)
        .into_iter()
        .chain(language_kind_checks(&dir, &tmp))
        .chain(personal_language_checks(&dir, &tmp))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
