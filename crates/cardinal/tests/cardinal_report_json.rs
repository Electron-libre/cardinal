//! Behavior test for `cardinal tool cardinal-report --json` and `--by milestone` (REP-20, REP-21): the tables as one
//! JSON document for a program, and the quality of the issues per milestone.

use std::path::{Path, PathBuf};
use std::process::Command;

const SESSIONS: &str = "session\tproject\tissues\trevision\tmodels\tversions\tstart\tduration_s\tturns\ttool_calls\tinput_tokens\toutput_tokens\tthinking_tokens\tcache_read_tokens\tcache_creation_tokens\tmain_context_peak\tadvisor_models\tadvisor_calls\tadvisor_input_tokens\tadvisor_output_tokens\thook_refusals\tskills\tvalidations\tother_messages\tunreadable_lines\tunknown_lines\tmissing_fields";
const QUALITY: &str = "revision\tsetup\tmilestone\tissues\tdeliverables\tcommits\tfixes_in_issue\tfixes_after\treverts_after\treviews\tcounted_reviews\treview_findings\ttraced\ttraced_without_red\tspec_lines\tdoc_lines\tspecs\tspec_bytes\tspec_requirements\tspec_intent_chars\tspec_happy_paths\tsessions\tfailed_checks\tvalidations\tother_messages\tinput_tokens\tcache_read_tokens\toutput_tokens\ttoken_issues";

/// A session `id` of project `p` on `revision`, started on day `day`.
fn session(id: usize, revision: &str, day: u32) -> String {
    format!(
        "s{id}\tp\t\t{revision}\tclaude-opus-5-5\tv1\t2026-01-{day:02}T00:00:{:02}.000Z\t60\t3\t2\t100\t10\t0\t50\t0\t200\t\t\t\t\t\t\t1\t0\t0\t0\t0",
        id % 60
    )
}

/// A quality summary row of `issues` issues of `milestone` on `revision`, each with one fix after its close.
fn quality(revision: &str, milestone: &str, issues: u64) -> String {
    format!(
        "{revision}\t\t{milestone}\t{issues}\t1\t1\t0\t{issues}{}\t1\t0\t0\t0\t100\t0\t10\t{issues}",
        "\t0".repeat(13)
    )
}

fn write(path: &Path, header: &str, rows: &[String]) -> PathBuf {
    let text: String = std::iter::once(header.to_string())
        .chain(rows.iter().cloned())
        .map(|l| l + "\n")
        .collect();
    std::fs::write(path, text).unwrap();
    path.to_path_buf()
}

/// The sessions of two groups of revisions: `r1` (10 sessions) then `r2` (3, too few).
fn sessions(dir: &Path) -> PathBuf {
    let rows: Vec<String> = (0..13)
        .map(|i| {
            session(
                i,
                if i < 10 { "r1" } else { "r2" },
                if i < 10 { 1 } else { 2 },
            )
        })
        .collect();
    write(&dir.join("sessions.tsv"), SESSIONS, &rows)
}

/// The quality summaries of the milestones `One` (3 issues over two revisions), none (1) and `Two` (2).
fn milestones(dir: &Path) -> PathBuf {
    write(
        &dir.join("quality.tsv"),
        QUALITY,
        &[
            quality("r1", "One", 2),
            quality("r1", "", 1),
            quality("r2", "Two", 2),
            quality("r2", "One", 1),
        ],
    )
}

/// The exit status, stdout and stderr of `cardinal-report` with `args`.
fn report(args: &[&Path]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "cardinal-report"])
        .args(args)
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn json(out: &str) -> serde_json::Value {
    assert_eq!(out.lines().count(), 1, "one line: {out}");
    serde_json::from_str(out).unwrap()
}

/// The table `title` of `doc`.
fn table<'a>(doc: &'a serde_json::Value, title: &str) -> &'a serde_json::Value {
    doc["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["title"] == title)
        .unwrap_or_else(|| panic!("no table {title}: {doc}"))
}

/// The cells of `column` in the rows of `table`.
fn cells(table: &serde_json::Value, column: &str) -> Vec<String> {
    let at = table["columns"]
        .as_array()
        .unwrap()
        .iter()
        .position(|c| c == column)
        .unwrap_or_else(|| panic!("no column {column}"));
    table["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r[at].as_str().unwrap().to_string())
        .collect()
}

fn titles(doc: &serde_json::Value) -> Vec<String> {
    doc["tables"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["title"].as_str().unwrap().to_string())
        .collect()
}

// spec: REP-21
#[test]
fn json_prints_the_tables_of_the_markdown_as_one_document() {
    let dir = tempfile::tempdir().unwrap();
    let file = sessions(dir.path());
    let (ok, out, _) = report(&[Path::new("--sessions"), &file, Path::new("--json")]);
    assert!(ok);
    let doc = json(&out);
    assert_eq!(doc["version"], 1);
    assert_eq!(titles(&doc), ["Efficiency", "Skills"]);
    let efficiency = table(&doc, "Efficiency");
    assert_eq!(&efficiency["columns"][0], "group");
    assert_eq!(&efficiency["columns"][1], "sessions");
    assert_eq!(cells(efficiency, "group"), ["r1", "r2"]);
    assert_eq!(cells(efficiency, "sessions"), ["10", "3"]);
    assert_eq!(cells(efficiency, "turns"), ["3", "too few"]);
    assert_eq!(cells(table(&doc, "Skills"), "calls")[1], "too few");
}

// spec: REP-21
#[test]
fn json_holds_the_quality_and_the_verdicts_when_given() {
    let dir = tempfile::tempdir().unwrap();
    let (s, q) = (sessions(dir.path()), milestones(dir.path()));
    let changes = write(
        &dir.path().join("changes.tsv"),
        "change\tmetric\tdirection\trevisions",
        &["c1\tturns\tdown\tr2".into()],
    );
    let args = [
        Path::new("--sessions"),
        &s,
        Path::new("--quality"),
        &q,
        Path::new("--changes"),
        &changes,
        Path::new("--json"),
    ];
    let (ok, out, _) = report(&args);
    assert!(ok);
    let doc = json(&out);
    assert_eq!(
        titles(&doc),
        ["Efficiency", "Quality", "Skills", "Verdicts"]
    );
    assert_eq!(cells(table(&doc, "Quality"), "issues"), ["3", "too few"]);
    assert_eq!(cells(table(&doc, "Verdicts"), "verdict"), ["confounded"]);
}

// spec: REP-20
#[test]
fn by_milestone_sums_the_quality_of_each_milestone() {
    let dir = tempfile::tempdir().unwrap();
    let (s, q) = (sessions(dir.path()), milestones(dir.path()));
    let args = [
        Path::new("--sessions"),
        &s,
        Path::new("--quality"),
        &q,
        Path::new("--by"),
        Path::new("milestone"),
        Path::new("--json"),
    ];
    let (ok, out, _) = report(&args);
    assert!(ok);
    let doc = json(&out);
    assert_eq!(titles(&doc), ["Quality"]);
    let quality = table(&doc, "Quality");
    assert_eq!(cells(quality, "group"), ["One", "(no milestone)", "Two"]);
    assert_eq!(cells(quality, "sessions"), ["", "", ""]);
    assert_eq!(cells(quality, "issues"), ["3", "1", "2"]);
    assert_eq!(
        cells(quality, "fixes_after_per_issue"),
        ["1.00", "1.00", "1.00"]
    );
}

// spec: REP-20
#[test]
fn by_milestone_prints_markdown_too() {
    let dir = tempfile::tempdir().unwrap();
    let (s, q) = (sessions(dir.path()), milestones(dir.path()));
    let args = [
        Path::new("--sessions"),
        &s,
        Path::new("--quality"),
        &q,
        Path::new("--by"),
        Path::new("milestone"),
    ];
    let (ok, out, _) = report(&args);
    assert!(ok);
    assert!(out.starts_with("## Quality\n"), "{out}");
    assert!(out.contains("\n| One |  | 3 |"), "{out}");
    assert!(!out.contains("## Efficiency"), "{out}");
}

// spec: REP-20
#[test]
fn by_milestone_needs_quality_and_refuses_last() {
    let dir = tempfile::tempdir().unwrap();
    let (s, q) = (sessions(dir.path()), milestones(dir.path()));
    let by = [Path::new("--by"), Path::new("milestone")];
    let (ok, _, err) = report(&[&[Path::new("--sessions"), &s], &by[..]].concat());
    assert!(!ok);
    assert!(err.contains("--quality"), "{err}");
    let (ok, _, _) = report(
        &[
            &[Path::new("--sessions"), &s, Path::new("--quality"), &q],
            &by[..],
            &[Path::new("--last"), Path::new("2")],
        ]
        .concat(),
    );
    assert!(!ok);
}
