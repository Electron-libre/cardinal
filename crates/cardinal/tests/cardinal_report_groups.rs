//! Behavior test for the options of `cardinal tool cardinal-report` that choose its groups and sessions, on fixture TSV
//! files: `--by setup`, `--last` and `--project`.

mod cardinal_report_common;

use std::path::{Path, PathBuf};

use cardinal_report_common::*;

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
fn cardinal_report_groups() {
    let dir = std::path::PathBuf::from("/tools");
    let temp = tempfile::tempdir().unwrap();
    let checks: Vec<_> = setup_checks(&dir, temp.path())
        .into_iter()
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
