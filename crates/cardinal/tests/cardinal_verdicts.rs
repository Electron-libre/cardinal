//! Behavior test for the verdicts of `cardinal tool cardinal-report` (`specs/cardinal-verdicts.md`) on fixture TSV
//! files: the changes of `--changes` and the metric each one should move, the verdict of each against the noise.

mod cardinal_report_common;

use std::path::Path;

use cardinal_report_common::*;

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

#[test]
fn cardinal_verdicts() {
    let dir = std::path::PathBuf::from("/tools");
    let temp = tempfile::tempdir().unwrap();
    let checks: Vec<_> = verdict_checks(&dir, temp.path())
        .into_iter()
        .chain(waste_verdict_checks(&dir, temp.path()))
        .chain(token_verdict_checks(&dir, temp.path()))
        .chain(verdict_edge_checks(&dir, temp.path()))
        .chain(issue_floor_checks(&dir, temp.path()))
        .chain(ratio_floor_checks(&dir, temp.path()))
        .chain(shape_floor_checks(&dir, temp.path()))
        .chain(advisor_verdict_checks(&dir, temp.path()))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
