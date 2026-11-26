//! Helpers of the tests of `cardinal tool cardinal-report` (`cardinal_report.rs`, `cardinal_verdicts.rs`,
//! `cardinal_report_groups.rs`): the fixture TSV files, the runs of the tool and the tables of its report.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const SESSIONS: &str = "session\tproject\tissues\trevision\tmodels\tversions\tstart\tduration_s\tturns\ttool_calls\tinput_tokens\toutput_tokens\tthinking_tokens\tcache_read_tokens\tcache_creation_tokens\tmain_context_peak\tadvisor_models\tadvisor_calls\tadvisor_input_tokens\tadvisor_output_tokens\thook_refusals\tskills\tvalidations\tother_messages\tunreadable_lines\tunknown_lines\tmissing_fields";
pub const BREACHES: &str = "rule\trevision\tapplied\tbreached\tbreaches\tblocked\tbytes";
pub const QUALITY: &str = "revision\tsetup\tmilestone\tissues\tdeliverables\tcommits\tfixes_in_issue\tfixes_after\treverts_after\treviews\tcounted_reviews\treview_findings\ttraced\ttraced_without_red\tspec_lines\tdoc_lines\tspecs\tspec_bytes\tspec_requirements\tspec_intent_chars\tspec_happy_paths\tsessions\tfailed_checks\tvalidations\tother_messages\tinput_tokens\tcache_read_tokens\toutput_tokens\ttoken_issues";
pub const CONTEXT: &str =
    "project\trevision\tcommit\tclaude_md_bytes\tagents_md_bytes\tskill_description_bytes\tskills";
pub const CHANGES: &str = "change\tmetric\tdirection\trevisions";
pub const SELF: &str = "self:1111111111111111111111111111111111111111";

/// A session row: the columns that vary, the others fixed (output 10); `input` holds the uncached input, cache read and
/// cache creation tokens; the main context peak is 100 per turn unless `peak` is `None` (an archived row written before
/// the column); with an `advisor` set, `calls` calls of 1,000 input and 10 output tokens each, else empty advisor cells.
#[derive(Clone)]
pub struct S<'a> {
    pub project: &'a str,
    pub issues: &'a str,
    pub revision: &'a str,
    pub models: &'a str,
    pub day: u32,
    pub turns: u64,
    pub input: [u64; 3],
    pub peak: Option<u64>,
    pub advisor: &'a str,
    pub calls: u64,
    pub others: u64,
    pub hooks: &'a str,
    pub skills: &'a str,
}

impl S<'_> {
    pub fn row(&self, id: usize) -> String {
        let start = format!("2026-01-{:02}T00:00:{:02}.000Z", self.day, id % 60);
        let advised = |n: u64| {
            if self.advisor.is_empty() {
                String::new()
            } else {
                n.to_string()
            }
        };
        let cells = [
            format!("s{id}"),
            self.project.into(),
            self.issues.into(),
            self.revision.into(),
            self.models.into(),
            "v1".into(),
            start,
            "60".into(),
            self.turns.to_string(),
            "2".into(),
            self.input[0].to_string(),
            "10".into(),
            "0".into(),
            self.input[1].to_string(),
            self.input[2].to_string(),
            self.peak.map_or_default(|p| p.to_string()),
            self.advisor.into(),
            advised(self.calls),
            advised(self.calls * 1000),
            advised(self.calls * 10),
            self.hooks.into(),
            self.skills.into(),
            "1".into(),
            self.others.to_string(),
            "0".into(),
            "0".into(),
            "0".into(),
        ];
        cells.join("\t")
    }
}

pub fn base<'a>(project: &'a str, revision: &'a str, day: u32, turns: u64) -> S<'a> {
    S {
        project,
        issues: "",
        revision,
        models: "m1",
        day,
        turns,
        input: [1, 2, 3],
        peak: Some(turns * 100),
        advisor: "",
        calls: 0,
        others: 0,
        hooks: "",
        skills: "",
    }
}

/// The sessions: `SELF` 6 (p1, issue i1, turns 1 to 6), `r-b` 5 (3 of p2 on i1 with two models, 2 of p1; turns 7 to
/// 11, the 2 of p1 with no peak, none with input; the 4 first of `SELF` with an advisor called 0, 1, 3 and 6 times), `r-c` 4
/// (turns 1 to 4, other messages 0, 0, 1, 1, no peak, input sums 0, 10, 100 and 100 whose cache reads are 0, 9, 10
/// and 50), and one with no revision, the earliest, left out. `r-c` is written first: the order is the order of use.
/// The other sessions read 2 of their 6 input tokens from the cache.
pub fn sessions() -> Vec<S<'static>> {
    let inputs = [[0, 0, 0], [1, 9, 0], [90, 10, 0], [40, 50, 10]];
    let mut list: Vec<S> = (1..=4)
        .zip(inputs)
        .map(|(t, input)| S {
            others: u64::from(t > 2),
            input,
            peak: None,
            ..base("p1", "r-c", 20, t)
        })
        .collect();
    list.push(base("p0", "", 1, 99));
    list.extend((1..=6).map(|t| S {
        issues: "i1",
        skills: "code-review=2",
        hooks: if t == 1 { "require_code_review=1" } else { "" },
        calls: match t {
            2 => 1,
            3 => 3,
            4 => 6,
            _ => 0,
        },
        advisor: if t <= 4 { "claude-opus-5-5" } else { "" },
        ..base("p1", SELF, 2, t)
    }));
    list.extend((7..=9).map(|t| S {
        issues: "i1",
        models: "m1,m2",
        skills: "files:commit-message=1",
        input: [0, 0, 0],
        ..base("p2", "r-b", 10, t)
    }));
    list.extend((10..=11).map(|t| S {
        peak: None,
        input: [0, 0, 0],
        ..base("p1", "r-b", 11, t)
    }));
    list
}

pub fn write(path: &Path, header: &str, rows: &[String]) -> PathBuf {
    std::fs::write(
        path,
        std::iter::once(header.to_string())
            .chain(rows.iter().cloned())
            .map(|l| l + "\n")
            .collect::<String>(),
    )
    .unwrap();
    path.to_path_buf()
}

pub fn run(dir: &Path, args: &[&std::ffi::OsStr]) -> (bool, String, String) {
    let out = tool_command(dir.join("cardinal_report.rs"))
        .args(args)
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The tables of a markdown report by heading: each a list of rows of trimmed cells, the header row first.
pub fn tables(out: &str) -> HashMap<String, Vec<Vec<String>>> {
    let mut tables: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    let mut heading = String::new();
    for line in out.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            heading = h.trim().to_string();
        } else if line.starts_with('|') && !line.starts_with("|-") && !line.starts_with("| -") {
            let cells = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect();
            tables.entry(heading.clone()).or_default().push(cells);
        }
    }
    tables
}

/// The cell of `column` in the row of `group` of a table, empty when absent.
pub fn cell(table: Option<&Vec<Vec<String>>>, group: &str, column: &str) -> String {
    let Some(rows) = table else {
        return String::new();
    };
    let Some(index) = rows
        .first()
        .and_then(|h| h.iter().position(|c| c == column))
    else {
        return String::new();
    };
    rows.iter()
        .skip(1)
        .find(|r| r.first().is_some_and(|g| g == group))
        .and_then(|r| r.get(index))
        .cloned()
        .unwrap_or_default()
}

/// The sessions, breaches, two quality and context files of the axes checks.
pub fn axes_inputs(tmp: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    let rows: Vec<String> = sessions()
        .iter()
        .enumerate()
        .map(|(i, s)| s.row(i))
        .collect();
    let sessions = write(&tmp.join("axes-sessions.tsv"), SESSIONS, &rows);
    let breaches = write(
        &tmp.join("breaches.tsv"),
        BREACHES,
        &[
            format!("x-rule\t{SELF}\t8\t1\t3\t0\t"),
            "x-rule\tr-b\t4\t1\t1\t2\t".into(),
            "a-rule\tr-b\t2\t0\t0\t0\t".into(),
            "x-rule\tr-c\t1\t1\t1\t0\t".into(),
            format!("oversized-result\t{SELF}\t3\t1\t2\t0\t70000"),
            "oversized-result\tr-b\t2\t1\t1\t0\t30001".into(),
        ],
    );
    let quality_a = write(
        &tmp.join("quality-a.tsv"),
        QUALITY,
        &[
            format!("{SELF}\t\t\t2\t4\t6\t1\t1\t0\t2\t2\t3\t5\t2\t150\t30\t0\t0\t0\t0\t0\t2\t1\t3\t4\t3000000\t2000000\t10000\t1"),
            "zzz\t\t\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9\t9".into(),
        ],
    );
    let quality_b = write(
        &tmp.join("quality-b.tsv"),
        QUALITY,
        &[
            "r-b\t\t\t1\t1\t2\t0\t0\t0\t1\t\t\t1\t1\t50\t20\t0\t0\t0\t0\t0\t1\t0\t1\t0\t1\t0\t2\t1"
                .into(),
        ],
    );
    let context = write(
        &tmp.join("context.tsv"),
        CONTEXT,
        &[
            format!("p1\t{SELF}\tc1\t100\t200\t300\tcode-review,commit-message,write-spec"),
            "p1\tr-b\tc2\t201\t400\t500\taudit,code-review,commit-message".into(),
            "p2\tr-b\t\t\t\t\t".into(),
        ],
    );
    (sessions, breaches, quality_a, quality_b, context)
}

/// A row of the quality summary of `revision` with `issues` issues and the spec shape `shape` (QUA-18: `specs`,
/// `spec_bytes`, `spec_requirements`, `spec_intent_chars`, `spec_happy_paths`), every other count 0.
pub fn shape_row(revision: &str, issues: u64, shape: [u64; 5]) -> String {
    let shape: Vec<String> = shape.iter().map(u64::to_string).collect();
    format!(
        "{revision}\t\t\t{issues}{}\t{}{}",
        "\t0".repeat(12),
        shape.join("\t"),
        "\t0".repeat(8)
    )
}

/// A 40-digit commit hash made of `c`.
pub fn hash(c: char) -> String {
    c.to_string().repeat(40)
}

/// The rows of the `Verdicts` table, header apart, for these sessions and changes (and the extra arguments).
pub fn verdicts(
    dir: &Path,
    tmp: &Path,
    sessions: &[S],
    changes: &[String],
    extra: &[&str],
) -> Vec<Vec<String>> {
    static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    let rows: Vec<String> = sessions.iter().enumerate().map(|(i, s)| s.row(i)).collect();
    let sessions = write(&tmp.join(format!("v-sessions-{n}.tsv")), SESSIONS, &rows);
    let changes = write(&tmp.join(format!("v-changes-{n}.tsv")), CHANGES, changes);
    let breaches = write(
        &tmp.join(format!("v-breaches-{n}.tsv")),
        BREACHES,
        &[
            "x-rule\tra\t4\t2\t2\t0\t".into(),
            "x-rule\trb\t4\t1\t1\t0\t".into(),
            "oversized-result\tra\t4\t2\t3\t0\t120000".into(),
            "oversized-result\trb\t4\t2\t2\t0\t48000".into(),
        ],
    );
    let mut args: Vec<&std::ffi::OsStr> = vec![
        "--sessions".as_ref(),
        sessions.as_os_str(),
        "--changes".as_ref(),
        changes.as_os_str(),
        "--breaches".as_ref(),
        breaches.as_os_str(),
        "--min-sessions".as_ref(),
        "2".as_ref(),
    ];
    args.extend(extra.iter().map(std::ffi::OsStr::new));
    let (ok, out, err) = run(dir, &args);
    if !ok {
        println!("{err}");
    }
    let last = out.rfind("## ").map_or("", |i| &out[i..]);
    tables(&out)
        .remove("Verdicts")
        .filter(|_| last.starts_with("## Verdicts"))
        .unwrap_or_default()
        .into_iter()
        .skip(1)
        .collect()
}

/// Two identical sessions.
pub fn two(project: &'static str, revision: &'static str, day: u32, turns: u64) -> Vec<S<'static>> {
    vec![
        base(project, revision, day, turns),
        base(project, revision, day, turns),
    ]
}

pub fn pair(a: Vec<S<'static>>, b: Vec<S<'static>>) -> Vec<S<'static>> {
    a.into_iter().chain(b).collect()
}

/// A change `c` of `metric` expected down, held by `revisions`.
pub fn change(c: char, metric: &str, revisions: &str) -> String {
    format!("{}\t{metric}\tdown\t{revisions}", hash(c))
}

/// The verdict column of verdict rows.
pub fn verdict(rows: &[Vec<String>]) -> Vec<String> {
    rows.iter().map(|r| r[7].clone()).collect()
}

/// The group of each row of the table `title` of a report.
pub fn group_names(out: &str, title: &str) -> Vec<String> {
    tables(out)
        .remove(title)
        .map_or_default(|rows| rows.into_iter().skip(1).map(|r| r[0].clone()).collect())
}

/// `cardinal tool <name>` for the script path the checks were written against (`next_issue.rs` is `next-issue`).
pub fn tool_command(script: impl AsRef<std::path::Path>) -> std::process::Command {
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
