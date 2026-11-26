//! Behavior test for `cardinal tool cardinal-report` on fixture TSV files: the groups of revisions, the tables per axis, the
//! confounders next to the numbers, the refusal of what it cannot read.

mod cardinal_report_common;

use std::path::Path;

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
        .chain(token_quality_checks(&dir, temp.path()))
        .chain(shape_quality_checks(&dir, temp.path()))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
