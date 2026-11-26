//! Behavior test for `cardinal tool cardinal-report` on fixture TSV files: the groups of revisions, the tables per axis, the
//! confounders next to the numbers, the refusal of what it cannot read.

mod cardinal_report_common;

use std::path::{Path, PathBuf};

use cardinal_report_common::*;

// spec: REP-3, REP-4, REP-5, REP-8, REP-9, REP-10, REP-11, REP-12, REP-H1
fn efficiency_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let rows: Vec<String> = sessions()
        .iter()
        .enumerate()
        .map(|(i, s)| s.row(i))
        .collect();
    let file = write(&tmp.join("sessions.tsv"), SESSIONS, &rows);
    let (ok, out, err) = run(dir, &["--sessions".as_ref(), file.as_os_str()]);
    if !ok {
        println!("{err}");
    }
    let t = tables(&out);
    let eff = t.get("Efficiency");
    let first = "self:1111111..r-b";
    let header: Vec<&str> = [
        "group",
        "sessions",
        "issues",
        "projects",
        "models",
        "versions",
        "turns",
        "tool_calls",
        "output_tokens",
        "input_tokens",
        "cache_read_share",
        "main_context_peak",
        "peak_sessions",
        "advisor_models",
        "advisor_sessions",
        "advisor_calls",
        "advisor_input_tokens",
        "advisor_output_tokens",
        "validations",
        "other_messages",
    ]
    .to_vec();
    let groups: Vec<String> =
        eff.map_or_default(|r| r.iter().skip(1).map(|c| c[0].clone()).collect());
    let too_few = eff
        .and_then(|r| r.iter().find(|c| c[0] == "r-c"))
        .is_some_and(|c| c[1] == "4" && c[2..].iter().all(|v| v == "too few"));
    vec![
        (
            "the report prints an efficiency table with its columns",
            ok && eff.and_then(|r| r.first()).is_some_and(|h| *h == header),
        ),
        (
            "revisions in the order of use, grouped up to the threshold, the empty one left out",
            groups == [first, "r-c"],
        ),
        (
            "a 40-digit revision is abbreviated to 7",
            out.contains("self:1111111") && !out.contains("self:11111111"),
        ),
        (
            "the last group under the threshold shows its size and `too few`",
            too_few,
        ),
        (
            "sessions, issues as project and issue pairs",
            cell(eff, first, "sessions") == "11" && cell(eff, first, "issues") == "2",
        ),
        (
            "projects, models and versions by sessions, most frequent first",
            cell(eff, first, "projects") == "p1=8, p2=3"
                && cell(eff, first, "models") == "m1=11, m2=3"
                && cell(eff, first, "versions") == "v1=11",
        ),
        (
            "medians per session of turns, tool calls and tokens",
            cell(eff, first, "turns") == "6"
                && cell(eff, first, "tool_calls") == "2"
                && cell(eff, first, "output_tokens") == "10"
                && cell(eff, first, "input_tokens") == "6",
        ),
        (
            "the median share of the input read from the cache",
            cell(eff, first, "cache_read_share") == "33%",
        ),
        (
            "the median main context peak leaves out the sessions with no peak",
            cell(eff, first, "main_context_peak") == "500",
        ),
        (
            "the sessions with a peak are counted next to its median",
            cell(eff, first, "peak_sessions") == "9",
        ),
        (
            "the advisors set by sessions, and the sessions that set one",
            cell(eff, first, "advisor_models") == "claude-opus-5-5=4"
                && cell(eff, first, "advisor_sessions") == "4",
        ),
        (
            "the medians of the advisor calls and tokens over the sessions that set one",
            cell(eff, first, "advisor_calls") == "2"
                && cell(eff, first, "advisor_input_tokens") == "2000"
                && cell(eff, first, "advisor_output_tokens") == "20",
        ),
    ]
}

// spec: REP-3, REP-4, REP-5, REP-8, REP-9, REP-10, REP-11, REP-12
fn option_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let rows: Vec<String> = sessions()
        .iter()
        .enumerate()
        .map(|(i, s)| s.row(i))
        .collect();
    let file = write(&tmp.join("options-sessions.tsv"), SESSIONS, &rows);
    let first = "self:1111111..r-b";
    let (_, small, _) = run(
        dir,
        &[
            "--sessions".as_ref(),
            file.as_os_str(),
            "--min-sessions".as_ref(),
            "2".as_ref(),
        ],
    );
    let small = tables(&small);
    let (_, anonymous, _) = run(
        dir,
        &[
            "--sessions".as_ref(),
            file.as_os_str(),
            "--anonymous".as_ref(),
        ],
    );
    let anon = tables(&anonymous);
    vec![
        (
            "a group with no peak has an empty main context peak and 0 peak sessions",
            cell(small.get("Efficiency"), "r-c", "main_context_peak").is_empty()
                && cell(small.get("Efficiency"), "r-c", "peak_sessions") == "0"
                && cell(small.get("Efficiency"), "r-c", "turns") == "3",
        ),
        (
            "a group with no advisor set has empty advisor cells and 0 advisor sessions",
            cell(small.get("Efficiency"), "r-c", "advisor_models").is_empty()
                && cell(small.get("Efficiency"), "r-c", "advisor_sessions") == "0"
                && cell(small.get("Efficiency"), "r-c", "advisor_calls").is_empty()
                && cell(small.get("Efficiency"), "r-c", "advisor_output_tokens").is_empty(),
        ),
        (
            "the cache read share is the median of the shares of the sessions, empty with no input",
            cell(small.get("Efficiency"), "r-c", "cache_read_share") == "50%"
                && cell(small.get("Efficiency"), "r-b", "cache_read_share").is_empty(),
        ),
        (
            "an even count takes the mean of the middle values, rounded half up",
            cell(small.get("Efficiency"), "r-c", "turns") == "3"
                && cell(small.get("Efficiency"), "r-c", "other_messages") == "1",
        ),
        (
            "--min-sessions sets the threshold",
            cell(small.get("Efficiency"), "self:1111111", "sessions") == "6"
                && cell(small.get("Efficiency"), "r-b", "sessions") == "5",
        ),
        (
            "--anonymous gives the count of projects and no project name",
            cell(anon.get("Efficiency"), first, "projects") == "2"
                && !anonymous.contains("p1")
                && !anonymous.contains("p2"),
        ),
    ]
}

// spec: REP-6, REP-7, REP-H2
fn axes_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let (sessions, breaches, quality_a, quality_b, context) = axes_inputs(tmp);
    let arg = |s: &'static str| std::ffi::OsStr::new(s);
    let (ok, out, err) = run(
        dir,
        &[
            arg("--sessions"),
            sessions.as_os_str(),
            arg("--breaches"),
            breaches.as_os_str(),
            arg("--quality"),
            quality_a.as_os_str(),
            arg("--quality"),
            quality_b.as_os_str(),
            arg("--context"),
            context.as_os_str(),
        ],
    );
    if !ok {
        println!("{err}");
    }
    let t = tables(&out);
    let first = "self:1111111..r-b";
    let (quality, rules, skills, fixed) = (
        t.get("Quality"),
        t.get("Rules"),
        t.get("Skills"),
        t.get("Fixed context"),
    );
    let rule_rows: Vec<String> = rules.map_or_default(|r| {
        r.iter()
            .skip(1)
            .filter(|c| c[0] == first)
            .map(|c| c[2..].join(" "))
            .collect()
    });
    let rule_rows: Vec<String> = rule_rows
        .into_iter()
        .filter(|r| !r.starts_with("oversized-result"))
        .collect();
    let few = |table: Option<&Vec<Vec<String>>>| {
        table.is_some_and(|r| {
            r.iter().filter(|c| c[0] == "r-c").count() == 1
                && r.iter()
                    .any(|c| c[0] == "r-c" && c[1] == "4" && c[2..].iter().all(|v| v == "too few"))
        })
    };
    let too_few = [quality, rules, skills, fixed].into_iter().all(few);
    vec![
        (
            "quality sums over the revisions and files of a group, other revisions left out",
            ok && cell(quality, first, "issues") == "3"
                && cell(quality, first, "commits") == "8"
                && cell(quality, first, "issue_sessions") == "3"
                && cell(quality, first, "other_messages") == "4"
                && cell(quality, first, "traced") == "6"
                && cell(quality, first, "traced_without_red") == "3",
        ),
        (
            "an empty value empties its sum and its ratio",
            cell(quality, first, "review_findings").is_empty()
                && cell(quality, first, "findings_per_review").is_empty(),
        ),
        (
            "fixes after per issue with two decimals",
            cell(quality, first, "fixes_after_per_issue") == "0.33",
        ),
        (
            "doc lines per spec line with two decimals",
            cell(quality, first, "doc_lines") == "50"
                && cell(quality, first, "doc_lines_per_spec_line") == "0.25",
        ),
        (
            "one rules row per rule of the group, sorted, sums and share",
            rule_rows == ["a-rule 2 0 0 0 0%  ", "x-rule 12 2 4 2 17%  "],
        ),
        (
            "skill calls and hook refusals summed, most frequent first",
            cell(skills, first, "calls") == "code-review=12, files:commit-message=3"
                && cell(skills, first, "hook_refusals") == "require_code_review=1",
        ),
        (
            "unused skills of the revisions, a prefixed call counting",
            cell(skills, first, "unused") == "audit, write-spec",
        ),
        (
            "fixed context means over the rows with bytes, rounded half up",
            cell(fixed, first, "claude_md_bytes") == "151"
                && cell(fixed, first, "agents_md_bytes") == "300"
                && cell(fixed, first, "skill_description_bytes") == "400"
                && cell(fixed, first, "total_bytes") == "851",
        ),
        (
            "a group with too few sessions is one row of `too few` in every table",
            too_few,
        ),
    ]
}

// spec: REP-6
fn token_quality_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let (sessions, _, quality_a, quality_b, _) = axes_inputs(tmp);
    let arg = |s: &'static str| std::ffi::OsStr::new(s);
    let (_, out, _) = run(
        dir,
        &[
            arg("--sessions"),
            sessions.as_os_str(),
            arg("--quality"),
            quality_a.as_os_str(),
            arg("--quality"),
            quality_b.as_os_str(),
        ],
    );
    let t = tables(&out);
    let (quality, first) = (t.get("Quality"), "self:1111111..r-b");
    vec![(
        "tokens summed and per issue, rounded half up to the unit",
        [
            ("input_tokens", "3000001"),
            ("cache_read_tokens", "2000000"),
            ("output_tokens", "10002"),
            ("token_issues", "2"),
            ("input_tokens_per_issue", "1500001"),
            ("output_tokens_per_issue", "5001"),
        ]
        .iter()
        .all(|(c, v)| cell(quality, first, c) == *v),
    )]
}

// spec: REP-19
fn shape_quality_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let (sessions, ..) = axes_inputs(tmp);
    let report = |name: &str, shape: [u64; 5]| {
        let quality = write(&tmp.join(name), QUALITY, &[shape_row(SELF, 2, shape)]);
        let arg = |s: &'static str| std::ffi::OsStr::new(s);
        let (_, out, _) = run(
            dir,
            &[
                arg("--sessions"),
                sessions.as_os_str(),
                arg("--quality"),
                quality.as_os_str(),
            ],
        );
        tables(&out)
    };
    let shaped = report("shape-quality.tsv", [4, 30_002, 50, 1_402, 7]);
    let none = report("shape-none.tsv", [0; 5]);
    let first = "self:1111111..r-b";
    let ratios = [
        "bytes_per_spec",
        "intent_chars_per_spec",
        "requirements_per_spec",
        "happy_paths_per_spec",
    ];
    vec![
        (
            "the spec shape is summed and divided by the specs of the group",
            cell(shaped.get("Quality"), first, "specs") == "4"
                && ratios
                    .iter()
                    .map(|c| cell(shaped.get("Quality"), first, c))
                    .eq(["7501", "351", "12.50", "1.75"]),
        ),
        (
            "no ratio per spec when the group changed no spec",
            ratios
                .iter()
                .all(|c| cell(none.get("Quality"), first, c).is_empty()),
        ),
    ]
}

// spec: REP-13
fn waste_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let (sessions, breaches, ..) = axes_inputs(tmp);
    let arg = |s: &'static str| std::ffi::OsStr::new(s);
    let (_, out, _) = run(
        dir,
        &[
            arg("--sessions"),
            sessions.as_os_str(),
            arg("--breaches"),
            breaches.as_os_str(),
        ],
    );
    let rows: Vec<String> = tables(&out).get("Rules").map_or_default(|r| {
        r.iter()
            .filter(|c| c[0] == "self:1111111..r-b")
            .map(|c| c[2..].join(" "))
            .collect()
    });
    vec![
        (
            "a waste rule sums its bytes and their mean per applied session",
            rows.contains(&"oversized-result 5 2 3 0 40% 100001 20000".to_string()),
        ),
        (
            "a rule that is no waste has empty bytes",
            rows.contains(&"x-rule 12 2 4 2 17%  ".to_string()),
        ),
    ]
}

// spec: REP-6, REP-7
fn axes_threshold_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let (sessions, _, quality_a, _, _) = axes_inputs(tmp);
    let arg = |s: &'static str| std::ffi::OsStr::new(s);
    let only_b = write(
        &tmp.join("breaches-b.tsv"),
        BREACHES,
        &["x-rule\tr-b\t4\t1\t1\t2\t".into()],
    );
    let (_, small, _) = run(
        dir,
        &[
            arg("--sessions"),
            sessions.as_os_str(),
            arg("--min-sessions"),
            arg("2"),
            arg("--breaches"),
            only_b.as_os_str(),
            arg("--quality"),
            quality_a.as_os_str(),
        ],
    );
    let small = tables(&small);
    let no_rule = small
        .get("Rules")
        .and_then(|r| r.iter().find(|c| c[0] == "self:1111111"))
        .is_some_and(|c| c.len() == 10 && c[2..].iter().all(String::is_empty));
    let no_spec = write(
        &tmp.join("quality-no-spec.tsv"),
        QUALITY,
        &[format!(
            "{SELF}\t\t\t1\t1\t1\t0\t0\t0\t1\t1\t0\t0\t0\t0\t9\t0\t0\t0\t0\t0\t1\t0\t0\t0\t0\t0\t0\t0"
        )],
    );
    let (_, docs_only, _) = run(
        dir,
        &[
            arg("--sessions"),
            sessions.as_os_str(),
            arg("--min-sessions"),
            arg("2"),
            arg("--quality"),
            no_spec.as_os_str(),
        ],
    );
    let docs_only = tables(&docs_only);
    let (_, bare, _) = run(dir, &[arg("--sessions"), sessions.as_os_str()]);
    let bare = tables(&bare);
    vec![
        (
            "a group with no quality row has empty sums, not zeros",
            cell(small.get("Quality"), "r-c", "issues").is_empty()
                && cell(small.get("Quality"), "self:1111111", "issues") == "2",
        ),
        (
            "no doc lines per spec line when the group changed no spec line",
            cell(docs_only.get("Quality"), "self:1111111", "doc_lines") == "9"
                && cell(
                    docs_only.get("Quality"),
                    "self:1111111",
                    "doc_lines_per_spec_line",
                )
                .is_empty(),
        ),
        (
            "no tokens per issue when the tokens are unknown",
            cell(docs_only.get("Quality"), "self:1111111", "token_issues") == "0"
                && cell(
                    docs_only.get("Quality"),
                    "self:1111111",
                    "input_tokens_per_issue",
                )
                .is_empty(),
        ),
        (
            "a group with no rule row keeps one row of empty cells",
            no_rule,
        ),
        (
            "a table whose input is not given is not printed",
            bare.contains_key("Skills")
                && !bare.contains_key("Quality")
                && !bare.contains_key("Rules")
                && !bare.contains_key("Fixed context"),
        ),
    ]
}

/// The sessions of three setups over two revisions: 12 of Sonnet advised by Opus (with a Haiku sub-agent, 2 of them on
/// `r2`), 15 of Opus alone (5 on `r2`, 3 with an Opus advisor set, 1 after a failed request), 4 of Sonnet alone, then one with no model and one with no revision, left out.
fn setup_sessions() -> Vec<String> {
    let mut list: Vec<S> = Vec::new();
    list.extend((1..=12).map(|t| S {
        models: "claude-sonnet-5-5,claude-haiku-5-5",
        advisor: "claude-opus-5-5,claude-fable-5-1",
        ..base("p1", if t > 10 { "r2" } else { "r1" }, 2, t)
    }));
    list.extend((1..=15).map(|t| S {
        models: if t == 1 {
            "<synthetic>,claude-opus-5-5"
        } else {
            "claude-opus-5-5"
        },
        advisor: if t > 12 { "claude-opus-5-5" } else { "" },
        ..base("p2", if t > 10 { "r2" } else { "r1" }, 3, t)
    }));
    list.extend((1..=4).map(|t| S {
        models: "claude-sonnet-5-5",
        ..base("p1", "r1", 4, t)
    }));
    list.push(S {
        models: "",
        ..base("p1", "r1", 5, 1)
    });
    list.push(S {
        models: "claude-opus-5-5",
        ..base("p1", "", 5, 1)
    });
    list.iter().enumerate().map(|(i, s)| s.row(i)).collect()
}

/// The inputs of the setup checks: the sessions, the quality summary rows of two setups over two revisions, then the
/// rules, fixed context and changes, which no setup reads.
fn setup_inputs(tmp: &Path) -> [PathBuf; 5] {
    let file = write(&tmp.join("setup-sessions.tsv"), SESSIONS, &setup_sessions());
    let (_, breaches, _, _, context) = axes_inputs(tmp);
    let zeros = "\t0".repeat(24);
    let quality = write(
        &tmp.join("setup-quality.tsv"),
        QUALITY,
        &[
            format!("r1\tclaude-opus-5-5 solo\t\t2{zeros}\t0"),
            format!("r2\tclaude-opus-5-5 solo\t\t1{zeros}\t0"),
            format!("r1\tmixed\t\t4{zeros}\t0"),
            format!("\tclaude-opus-5-5 solo\t\t5{zeros}\t0"),
        ],
    );
    let changes = write(
        &tmp.join("setup-changes.tsv"),
        CHANGES,
        &["c1\tturns\tdown\tr1..r2".into()],
    );
    [file, quality, breaches, context, changes]
}

// spec: REP-14, REP-15, REP-16
fn setup_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let [file, quality, breaches, context, changes] = setup_inputs(tmp);
    let by = |value: &str| {
        run(
            dir,
            &[
                "--sessions".as_ref(),
                file.as_os_str(),
                "--breaches".as_ref(),
                breaches.as_os_str(),
                "--context".as_ref(),
                context.as_os_str(),
                "--changes".as_ref(),
                changes.as_os_str(),
                "--quality".as_ref(),
                quality.as_os_str(),
                "--by".as_ref(),
                value.as_ref(),
            ],
        )
    };
    let (ok, out, err) = by("setup");
    if !ok {
        println!("{err}");
    }
    let t = tables(&out);
    let eff = t.get("Efficiency");
    let groups: Vec<String> =
        eff.map_or_default(|r| r.iter().skip(1).map(|c| c[0].clone()).collect());
    let advised = "claude-sonnet-5-5 advised by claude-opus-5-5";
    let too_few = eff
        .and_then(|r| r.iter().find(|c| c[0] == "claude-sonnet-5-5 solo"))
        .is_some_and(|c| c[1] == "4" && c[2..].iter().all(|v| v == "too few"));
    let header = eff.and_then(|r| r.first());
    let after_versions = header.is_some_and(|h| {
        h.iter()
            .position(|c| c == "versions")
            .is_some_and(|i| h.get(i + 1).is_some_and(|c| c == "revisions"))
    });
    let skills_header = t.get("Skills").and_then(|r| r.first());
    vec![
        (
            "--by setup groups the sessions by setup, most sessions first, the unknown ones left out",
            ok && groups
                == [
                    "claude-opus-5-5 solo",
                    advised,
                    "claude-sonnet-5-5 solo",
                ],
        ),
        (
            "a setup group counts its sessions",
            cell(eff, advised, "sessions") == "12",
        ),
        (
            "a setup group under the threshold is too few",
            too_few,
        ),
        (
            "the efficiency table gives the revisions of a setup after the versions",
            after_versions && cell(eff, "claude-opus-5-5 solo", "revisions") == "r1=10, r2=5",
        ),
        (
            "the quality table sums the summary rows of the setup, over every non-empty revision",
            cell(t.get("Quality"), "claude-opus-5-5 solo", "issues") == "3"
                && cell(t.get("Quality"), advised, "issues").is_empty(),
        ),
        (
            "the skills table leaves out the unused skills by setup",
            skills_header.is_some_and(|h| !h.iter().any(|c| c == "unused")),
        ),
        (
            "the revision-keyed tables are not printed by setup",
            !t.contains_key("Rules") && !t.contains_key("Fixed context") && !out.contains("## Verdicts"),
        ),
        (
            "--by revision keeps the groups of revisions",
            by("revision").0 && tables(&by("revision").1).contains_key("Rules"),
        ),
        ("an unknown --by value is refused", !by("model").0),
        (
            "a --by with no value is refused",
            !run(dir, &["--sessions".as_ref(), file.as_os_str(), "--by".as_ref()]).0,
        ),
    ]
}

// spec: REP-3
fn refusal_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let good = write(
        &tmp.join("good.tsv"),
        SESSIONS,
        &[base("p1", "r", 1, 1).row(0)],
    );
    let bad = write(&tmp.join("bad.tsv"), "session\tsomething_else", &[]);
    let refused = |args: &[&std::ffi::OsStr]| !run(dir, args).0;
    vec![
        ("a missing --sessions is refused", refused(&[])),
        (
            "a sessions file with another header is refused",
            refused(&["--sessions".as_ref(), bad.as_os_str()]),
        ),
        (
            "a quality file with another header is refused",
            refused(&[
                "--sessions".as_ref(),
                good.as_os_str(),
                "--quality".as_ref(),
                bad.as_os_str(),
            ]),
        ),
        (
            "a --min-sessions out of range is refused",
            refused(&[
                "--sessions".as_ref(),
                good.as_os_str(),
                "--min-sessions".as_ref(),
                "0".as_ref(),
            ]),
        ),
        (
            "a --min-issues out of range is refused",
            refused(&[
                "--sessions".as_ref(),
                good.as_os_str(),
                "--min-issues".as_ref(),
                "0".as_ref(),
            ]),
        ),
        (
            "an unknown option is refused",
            refused(&["--sessions".as_ref(), good.as_os_str(), "--bogus".as_ref()]),
        ),
        (
            "a changes file with another header is refused",
            refused(&[
                "--sessions".as_ref(),
                good.as_os_str(),
                "--changes".as_ref(),
                bad.as_os_str(),
            ]),
        ),
        (
            "a readable input is accepted",
            run(dir, &["--sessions".as_ref(), good.as_os_str()]).0,
        ),
    ]
}

// spec: VRD-5, VRD-6, VRD-7, VRD-9, VRD-H2
fn verdict_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    // Groups `ra` (turns 10), `rb` (5), `rc` (8), `rd` (one session: too few).
    let plain: Vec<S> = [
        two("p1", "ra", 1, 10),
        two("p1", "rb", 2, 5),
        two("p1", "rc", 3, 8),
        vec![base("p1", "rd", 4, 3)],
    ]
    .into_iter()
    .flatten()
    .collect();
    let main = verdicts(
        dir,
        tmp,
        &plain,
        &[
            format!("{}\tturns\tdown\trb,rc,rd", hash('a')),
            format!("{}\tbreached_share:x-rule\tdown\trb,rc,rd", hash('a')),
            format!("{}\tturns\tdown\trc,rd", hash('b')),
            format!("{}\tturns\tdown\trd", hash('c')),
            format!("{}\tturns\tdown\t", hash('d')),
            format!("{}\tturns\tup\tra,rb,rc,rd", hash('e')),
        ],
        &[],
    );
    let row = |change: char, metric: &str, rest: &[&str]| -> Vec<String> {
        [&hash(change)[..7], metric]
            .iter()
            .copied()
            .chain(rest.iter().copied())
            .map(str::to_string)
            .collect()
    };
    let expected = vec![
        row('a', "turns", &["down", "ra", "rb", "10", "5", "helped"]),
        row(
            'a',
            "breached_share:x-rule",
            &["down", "ra", "rb", "50", "25", "helped"],
        ),
        row('b', "turns", &["down", "rb", "rc", "5", "8", "hurt"]),
        row('c', "turns", &["down", "rc", "rd", "8", "3", "confounded"]),
        row('d', "turns", &["down", "", "", "", "", "no data"]),
        row('e', "turns", &["up", "", "ra", "", "10", "no data"]),
    ];
    let one = |extra: &[&str], sessions: Vec<S<'static>>, changes: &[String]| {
        verdict(&verdicts(dir, tmp, &sessions, changes, extra))
    };
    let models: Vec<S> = pair(
        two("p1", "ra", 1, 10),
        vec![
            S {
                models: "m2",
                ..base("p1", "rb", 2, 5)
            },
            S {
                models: "m2",
                ..base("p1", "rb", 2, 5)
            },
        ],
    );
    let anonymous = verdicts(
        dir,
        tmp,
        &pair(two("p1", "ra", 1, 10), two("secret-project", "rb", 2, 5)),
        &[change('a', "turns", "rb")],
        &["--anonymous"],
    );
    let mut mixed = two("p1", "ra", 1, 10);
    mixed.extend(two("p1", "ra", 1, 10));
    mixed.extend(two("p1", "rb", 2, 5));
    mixed.extend(two("p1", "rc", 3, 5));
    vec![
        (
            "the verdicts table comes last with a row per change and metric",
            main == expected,
        ),
        (
            "a change of the most frequent model confounds",
            one(&[], models, &[change('a', "turns", "rb")]) == ["confounded"],
        ),
        (
            "a change of the most frequent project confounds, and --anonymous names none",
            verdict(&anonymous) == ["confounded"]
                && !anonymous.concat().iter().any(|c| c.contains("secret")),
        ),
        (
            "an after group holding a revision without the change confounds",
            one(
                &["--min-sessions", "4"],
                mixed,
                &[change('a', "turns", "rc")],
            ) == ["confounded"],
        ),
    ]
}

// spec: VRD-13
fn waste_verdict_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let rows = verdicts(
        dir,
        tmp,
        &pair(two("p1", "ra", 1, 10), two("p1", "rb", 2, 10)),
        &[
            change('a', "bytes_per_session:oversized-result", "rb"),
            change('a', "breached_share:oversized-result", "rb"),
        ],
        &[],
    );
    let values: Vec<String> = rows.iter().map(|r| r[5..].join(" ")).collect();
    vec![(
        "the bytes per session of a waste rule come from its row of the rules table",
        values == ["30000 12000 helped", "50 50 no observed effect"],
    )]
}

// spec: REP-17
fn last_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    // Groups `ra`, `rb` and `rc` of two sessions each; a change held first by `rb`.
    let three: Vec<S> = [
        two("p1", "ra", 1, 10),
        two("p1", "rb", 2, 5),
        two("p1", "rc", 3, 8),
    ]
    .into_iter()
    .flatten()
    .collect();
    let rows: Vec<String> = three.iter().enumerate().map(|(i, s)| s.row(i)).collect();
    let sessions = write(&tmp.join("last-sessions.tsv"), SESSIONS, &rows);
    let breaches = write(
        &tmp.join("last-breaches.tsv"),
        BREACHES,
        &[
            "x-rule\tra\t2\t1\t1\t0\t".into(),
            "x-rule\trc\t2\t1\t1\t0\t".into(),
        ],
    );
    let base_args = |last: &'static str| -> Vec<&std::ffi::OsStr> {
        vec![
            "--sessions".as_ref(),
            sessions.as_os_str(),
            "--breaches".as_ref(),
            breaches.as_os_str(),
            "--min-sessions".as_ref(),
            "2".as_ref(),
            "--last".as_ref(),
            last.as_ref(),
        ]
    };
    let (ok, out, _) = run(dir, &base_args("2"));
    let shown = ["Efficiency", "Rules", "Skills"]
        .iter()
        .all(|t| group_names(&out, t) == ["rb", "rc"]);
    let (_, more, _) = run(dir, &base_args("5"));
    let all = group_names(&more, "Efficiency") == ["ra", "rb", "rc"];
    let verdict_rows = verdicts(
        dir,
        tmp,
        &three,
        &[change('a', "turns", "rb,rc")],
        &["--last", "2"],
    );
    let before = verdict_rows.first().map(|r| r[3..7].join(" "));
    let refused = |extra: &[&str]| {
        let mut args = vec!["--sessions".as_ref(), sessions.as_os_str()];
        args.extend(extra.iter().map(std::ffi::OsStr::new));
        !run(dir, &args).0
    };
    vec![
        ("--last succeeds", ok),
        (
            "--last <n> prints the rows of the last n groups only",
            shown,
        ),
        ("--last over the count of groups prints them all", all),
        (
            "--last keeps the verdict of a change from the group before it",
            before.as_deref() == Some("ra rb 10 5"),
        ),
        (
            "a --last out of range or missing, or with --by setup, is refused",
            refused(&["--last", "0"])
                && refused(&["--last", "x"])
                && refused(&["--last"])
                && refused(&["--last", "2", "--by", "setup"]),
        ),
    ]
}

// spec: VRD-1, VRD-5
fn token_verdict_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let quality = write(
        &tmp.join("v-quality.tsv"),
        QUALITY,
        &[
            "ra\t\t\t2\t1\t1\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t1\t0\t0\t0\t1000\t0\t10\t2".into(),
            "rb\t\t\t2\t1\t1\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t1\t0\t0\t0\t600\t0\t30\t2".into(),
        ],
    );
    let tokens = verdicts(
        dir,
        tmp,
        &pair(two("p1", "ra", 1, 10), two("p1", "rb", 2, 5)),
        &[
            change('a', "input_tokens_per_issue", "rb"),
            change('a', "output_tokens_per_issue", "rb"),
        ],
        &["--quality", quality.to_str().unwrap(), "--min-issues", "2"],
    );
    let token_values: Vec<String> = tokens.iter().map(|r| r[5..].join(" ")).collect();
    vec![(
        "the tokens per issue come from the quality table",
        token_values == ["500 300 helped", "5 15 hurt"],
    )]
}

// spec: VRD-5, VRD-6, VRD-7, VRD-9
fn verdict_edge_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let one = |sessions: Vec<S<'static>>, changes: &[String]| {
        verdict(&verdicts(dir, tmp, &sessions, changes, &[]))
    };
    let unpeaked = |s: S<'static>| S { peak: None, ..s };
    let peak_after: Vec<S> = pair(
        two("p1", "ra", 1, 10),
        vec![base("p1", "rb", 2, 5), unpeaked(base("p1", "rb", 2, 5))],
    );
    let peak_before: Vec<S> = pair(
        vec![unpeaked(base("p1", "ra", 1, 10)), base("p1", "ra", 1, 10)],
        two("p1", "rb", 2, 5),
    );
    let no_peak_before: Vec<S> = pair(
        vec![
            unpeaked(base("p1", "ra", 1, 10)),
            unpeaked(base("p1", "ra", 1, 10)),
        ],
        two("p1", "rb", 2, 5),
    );
    vec![
        (
            "two changes with the same after group confound each other",
            one(
                pair(two("p1", "ra", 1, 10), two("p1", "rb", 2, 5)),
                &[change('a', "turns", "rb"), change('b', "validations", "rb")],
            ) == ["confounded", "confounded"],
        ),
        (
            "a main context peak held by fewer sessions than the threshold confounds, before or after",
            one(peak_after, &[change('a', "main_context_peak", "rb")]) == ["confounded"]
                && one(peak_before, &[change('a', "main_context_peak", "rb")]) == ["confounded"],
        ),
        (
            "a main context peak every session holds gets a verdict",
            one(
                pair(two("p1", "ra", 1, 10), two("p1", "rb", 2, 5)),
                &[change('a', "main_context_peak", "rb")],
            ) == ["helped"],
        ),
        (
            "a group with no main context peak at all gives no data",
            one(no_peak_before, &[change('a', "main_context_peak", "rb")]) == ["no data"],
        ),
        (
            "the cache read share gets a verdict, its `%` dropped",
            one(
                pair(
                    two("p1", "ra", 1, 5),
                    vec![
                        S {
                            input: [90, 10, 0],
                            ..base("p1", "rb", 2, 5)
                        };
                        2
                    ],
                ),
                &[change('a', "cache_read_share", "rb")],
            ) == ["helped"],
        ),
        (
            "an unchanged value is no observed effect",
            one(
                pair(two("p1", "ra", 1, 5), two("p1", "rb", 2, 5)),
                &[change('a', "turns", "rb")],
            ) == ["no observed effect"],
        ),
    ]
}

// spec: VRD-11
fn issue_floor_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    // 6 issues in `ra`, 8 in `rb`, the tokens of only 2 of `ra` known
    let quality = write(
        &tmp.join("f-quality.tsv"),
        QUALITY,
        &[
            "ra\t\t\t6\t1\t1\t0\t3\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t1\t0\t0\t0\t1000\t0\t10\t2".into(),
            "rb\t\t\t8\t1\t1\t0\t1\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t1\t0\t0\t0\t600\t0\t30\t6".into(),
        ],
    );
    let quality = quality.to_str().unwrap();
    let one = |extra: &[&str]| {
        verdict(&verdicts(
            dir,
            tmp,
            &pair(two("p1", "ra", 1, 10), two("p1", "rb", 2, 5)),
            &[
                change('a', "fixes_after_per_issue", "rb"),
                change('a', "input_tokens_per_issue", "rb"),
            ],
            &[&["--quality", quality], extra].concat(),
        ))
    };
    vec![
        (
            "a group with fewer issues of known tokens than --min-issues (5) confounds a token verdict",
            one(&[]) == ["helped", "confounded"],
        ),
        (
            "a before group alone with fewer issues than --min-issues confounds a per-issue verdict",
            one(&["--min-issues", "7"]) == ["confounded", "confounded"],
        ),
        (
            "enough issues in both groups gives a verdict",
            one(&["--min-issues", "2"]) == ["helped", "helped"],
        ),
    ]
}

// spec: VRD-12
fn ratio_floor_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    // `ra`: 6 issues, 12 reviews of which `ra_reviews` give their 24 findings, 200 doc lines on 100 spec lines; `rb`: `rb_issues`, 12 reviews
    // with 6 findings, 100 doc lines on 100 spec lines
    let one = |ra_reviews: u64, rb_issues: u64, extra: &[&str]| {
        let quality = write(
            &tmp.join("r-quality.tsv"),
            QUALITY,
            &[
                format!("ra\t\t\t6\t1\t1\t0\t0\t0\t12\t{ra_reviews}\t24\t0\t0\t100\t200\t0\t0\t0\t0\t0\t1\t0\t0\t0\t0\t0\t0\t0"),
                format!(
                    "rb\t\t\t{rb_issues}\t1\t1\t0\t0\t0\t12\t12\t6\t0\t0\t100\t100\t0\t0\t0\t0\t0\t1\t0\t0\t0\t0\t0\t0\t0"
                ),
            ],
        );
        verdict(&verdicts(
            dir,
            tmp,
            &pair(two("p1", "ra", 1, 10), two("p1", "rb", 2, 5)),
            &[
                change('a', "findings_per_review", "rb"),
                change('a', "doc_lines_per_spec_line", "rb"),
            ],
            &[&["--quality", quality.to_str().unwrap()], extra].concat(),
        ))
    };
    vec![
        (
            "a before group with fewer reviews than --min-issues confounds a findings verdict",
            one(2, 6, &[]) == ["confounded", "helped"],
        ),
        (
            "an after group with fewer issues but enough reviews confounds both ratio verdicts",
            one(12, 3, &["--min-issues", "2"]) == ["helped", "helped"]
                && one(12, 3, &["--min-issues", "4"]) == ["confounded", "confounded"],
        ),
    ]
}

// spec: VRD-1, VRD-14
fn shape_floor_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    // `ra` measures 3 specs with 3 happy paths, `rb` 8 specs with 16: expected down, the happy paths per spec go up
    // from 1.00 to 2.00 and the bytes per spec down from 1000 to 500
    let one = |ra_issues: u64, extra: &[&str]| {
        let quality = write(
            &tmp.join("s-quality.tsv"),
            QUALITY,
            &[
                shape_row("ra", ra_issues, [3, 3_000, 30, 300, 3]),
                shape_row("rb", 6, [8, 4_000, 40, 400, 16]),
            ],
        );
        let quality = quality.to_str().unwrap();
        verdict(&verdicts(
            dir,
            tmp,
            &pair(two("p1", "ra", 1, 10), two("p1", "rb", 2, 5)),
            &[
                change('a', "happy_paths_per_spec", "rb"),
                change('a', "bytes_per_spec", "rb"),
            ],
            &[&["--quality", quality], extra].concat(),
        ))
    };
    vec![
        (
            "a group with fewer specs than --min-issues confounds a per-spec verdict",
            one(6, &[]) == ["confounded", "confounded"],
        ),
        (
            "enough specs in both groups gives a per-spec verdict",
            one(6, &["--min-issues", "3"]) == ["hurt", "helped"],
        ),
        (
            "a group of fewer issues than --min-issues confounds a per-spec verdict, whatever its specs",
            one(2, &["--min-issues", "3"]) == ["confounded", "confounded"],
        ),
    ]
}

// spec: VRD-10
fn advisor_verdict_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let one = |sessions: Vec<S<'static>>, changes: &[String]| {
        verdict(&verdicts(dir, tmp, &sessions, changes, &[]))
    };
    let advised = |revision: &'static str, calls: u64| -> Vec<S<'static>> {
        let day = if revision == "ra" { 1 } else { 2 };
        vec![
            S {
                advisor: "claude-opus-5-5",
                calls,
                ..base("p1", revision, day, 5)
            };
            2
        ]
    };
    let advisor_after = pair(
        advised("ra", 6),
        vec![advised("rb", 2).remove(0), base("p1", "rb", 2, 5)],
    );
    let advisor_before = pair(
        vec![advised("ra", 6).remove(0), base("p1", "ra", 1, 5)],
        advised("rb", 2),
    );
    vec![
        (
            "advisor metrics set by fewer sessions than the threshold confound, before or after",
            one(advisor_after, &[change('a', "advisor_calls", "rb")]) == ["confounded"]
                && one(
                    advisor_before,
                    &[change('a', "advisor_output_tokens", "rb")],
                ) == ["confounded"],
        ),
        (
            "a change of the most frequent advisor confounds an advisor metric, not another one",
            one(
                pair(
                    advised("ra", 6),
                    advised("rb", 2)
                        .into_iter()
                        .map(|s| S {
                            advisor: "claude-haiku-5-5",
                            ..s
                        })
                        .collect(),
                ),
                &[change('a', "advisor_calls", "rb")],
            ) == ["confounded"]
                && one(
                    pair(
                        advised("ra", 6),
                        vec![
                            S {
                                advisor: "claude-haiku-5-5",
                                turns: 4,
                                ..advised("rb", 2).remove(0)
                            };
                            2
                        ],
                    ),
                    &[change('a', "turns", "rb")],
                ) == ["helped"],
        ),
        (
            "advisor metrics every session sets get a verdict",
            one(
                pair(advised("ra", 6), advised("rb", 2)),
                &[change('a', "advisor_input_tokens", "rb")],
            ) == ["helped"],
        ),
    ]
}

// spec: REP-18
fn project_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let tmp = tmp.join("project");
    std::fs::create_dir_all(&tmp).unwrap();
    let (sessions, ..) = axes_inputs(&tmp);
    let context = write(
        &tmp.join("context.tsv"),
        CONTEXT,
        &[
            "p1\tr-b\tc2\t100\t0\t0\ta".into(),
            "p2\tr-b\tc3\t900\t0\t0\ta".into(),
        ],
    );
    let args = |project: &[&'static str]| -> Vec<std::ffi::OsString> {
        let mut args: Vec<std::ffi::OsString> = vec![
            "--sessions".into(),
            sessions.clone().into(),
            "--context".into(),
            context.clone().into(),
            "--min-sessions".into(),
            "2".into(),
        ];
        args.extend(project.iter().map(Into::into));
        args
    };
    let report = |project: &[&'static str]| {
        let args = args(project);
        let refs: Vec<&std::ffi::OsStr> = args.iter().map(std::ffi::OsString::as_os_str).collect();
        run(dir, &refs)
    };
    let (ok, out, _) = report(&["--project", "p1"]);
    let t = tables(&out);
    let eff = t.get("Efficiency");
    let (all_ok, all, _) = report(&[]);
    let all = tables(&all);
    let (refused, _, error) = report(&["--project"]);
    let (flag_refused, _, _) = report(&["--project", "--anonymous"]);
    let (unknown_ok, _, warning) = report(&["--project", "nope"]);
    vec![
        (
            "--project keeps the sessions of that project only, its groups cut over them",
            ok && cell(eff, "r-b", "sessions") == "2"
                && cell(eff, "r-b", "projects") == "p1=2"
                && group_names(&out, "Efficiency") == ["self:1111111", "r-b", "r-c"],
        ),
        (
            "--project keeps the fixed context rows of that project only",
            cell(t.get("Fixed context"), "r-b", "total_bytes") == "100"
                && all_ok
                && cell(all.get("Fixed context"), "r-b", "total_bytes") == "500",
        ),
        (
            "--project with no value, or an option as its value, is refused",
            !refused && error.contains("--project") && !flag_refused,
        ),
        (
            "--project naming no project of the sessions warns",
            unknown_ok && warning.contains("nope"),
        ),
    ]
}

#[test]
fn cardinal_report() {
    let dir = std::path::PathBuf::from("/tools");
    let temp = tempfile::tempdir().unwrap();
    let checks: Vec<_> = efficiency_checks(&dir, temp.path())
        .into_iter()
        .chain(option_checks(&dir, temp.path()))
        .chain(axes_checks(&dir, temp.path()))
        .chain(waste_checks(&dir, temp.path()))
        .chain(axes_threshold_checks(&dir, temp.path()))
        .chain(refusal_checks(&dir, temp.path()))
        .chain(verdict_checks(&dir, temp.path()))
        .chain(verdict_edge_checks(&dir, temp.path()))
        .chain(waste_verdict_checks(&dir, temp.path()))
        .chain(token_verdict_checks(&dir, temp.path()))
        .chain(token_quality_checks(&dir, temp.path()))
        .chain(advisor_verdict_checks(&dir, temp.path()))
        .chain(issue_floor_checks(&dir, temp.path()))
        .chain(ratio_floor_checks(&dir, temp.path()))
        .chain(shape_quality_checks(&dir, temp.path()))
        .chain(shape_floor_checks(&dir, temp.path()))
        .chain(setup_checks(&dir, temp.path()))
        .chain(last_checks(&dir, temp.path()))
        .chain(project_checks(&dir, temp.path()))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
