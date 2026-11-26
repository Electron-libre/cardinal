//! Behavior test for the archive of `cardinal tool session-metrics` (EVA-11, EVA-14, EVA-17, EVA-20): created with its
//! parent directories, merged by session and sorted by start, upgraded from older headers, refused under another one.

mod session_metrics_common;

use session_metrics_common::*;
use std::path::Path;

/// The columns before `main_context_peak` (EVA-17): an archive written then is upgraded.
const OLD_COLUMNS: &str = "session\tproject\tissues\trevision\tmodels\tversions\tstart\tduration_s\tturns\ttool_calls\tinput_tokens\toutput_tokens\tthinking_tokens\tcache_read_tokens\tcache_creation_tokens\thook_refusals\tskills\tvalidations\tother_messages\tunreadable_lines\tunknown_lines\tmissing_fields";
/// The columns before the advisor columns (EVA-20): an archive written then is upgraded.
const PRE_ADVISOR_COLUMNS: &str = "session\tproject\tissues\trevision\tmodels\tversions\tstart\tduration_s\tturns\ttool_calls\tinput_tokens\toutput_tokens\tthinking_tokens\tcache_read_tokens\tcache_creation_tokens\tmain_context_peak\thook_refusals\tskills\tvalidations\tother_messages\tunreadable_lines\tunknown_lines\tmissing_fields";

/// The sessions `b` (two validations) and `c` (one) of the project `p` under `tmp/projects`.
fn archived_sessions(tmp: &Path) {
    let projects = tmp.join("projects/-p");
    std::fs::create_dir_all(&projects).unwrap();
    let cwd = Path::new("/nonexistent/p");
    std::fs::write(
        projects.join("b.jsonl"),
        [
            typed(cwd, "2026-01-02T00:00:00.000Z", "ok"),
            typed(cwd, "2026-01-02T00:00:01.000Z", "ok"),
        ]
        .concat(),
    )
    .unwrap();
    std::fs::write(
        projects.join("c.jsonl"),
        typed(cwd, "2026-01-01T12:00:00.000Z", "ok"),
    )
    .unwrap();
}

// spec: EVA-11, EVA-14, EVA-17
fn archive_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let tmp = tmp.join("archive");
    archived_sessions(&tmp);
    let archive = tmp.join("state/deep/sessions.tsv");
    let run = || {
        tool_command(dir.join("session_metrics.rs"))
            .arg(tmp.join("projects"))
            .arg("--archive")
            .arg(&archive)
            .output()
            .unwrap()
    };
    let first = run();
    let created = std::fs::read_to_string(&archive).unwrap_or_default();
    let old_row = |id: &str, start: &str| {
        format!("{id}\tp\t\t\t\t\t{start}\t0\t0\t0\t0\t0\t0\t0\t0\t0\t\t\t\t\t\t\t9\t0\t0\t0\t0\n")
    };
    std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
    let resolved =
        "c\tkeep\t\trrr\t\t\t2026-01-01T12:00:00.000Z\t0\t0\t0\t0\t0\t0\t0\t0\t0\t\t\t\t\t\t\t9\t0\t0\t0\t0\n";
    std::fs::write(
        &archive,
        format!(
            "{COLUMNS}\n{}{}{resolved}",
            old_row("b", "2026-01-02T00:00:00.000Z"),
            old_row("a", "2026-01-01T00:00:00.000Z")
        ),
    )
    .unwrap();
    let merged = run();
    let text = std::fs::read_to_string(&archive).unwrap_or_default();
    let ids: Vec<&str> = text
        .lines()
        .skip(1)
        .map(|l| l.split('\t').next().unwrap_or_default())
        .collect();
    let merged_rows = rows(&text).unwrap_or_default();
    let cell = |id: &str, c: &str| {
        merged_rows
            .get(id)
            .and_then(|r| r.get(c))
            .cloned()
            .unwrap_or_default()
    };
    let b_validations = cell("b", "validations");
    let before_peak = format!(
        "{OLD_COLUMNS}\nold\tp\t\t\t\t\t2026-01-01T00:00:00.000Z\t0\t0\t0\t0\t0\t0\t0\t0\t\t\t9\t0\t0\t0\t0\n"
    );
    std::fs::write(&archive, before_peak).unwrap();
    let upgraded = run();
    let upgraded_text = std::fs::read_to_string(&archive).unwrap_or_default();
    let upgraded_rows = rows(&upgraded_text).unwrap_or_default();
    let old = upgraded_rows.get("old");
    let foreign = "session\tsomething_else\na\tx\n";
    std::fs::write(&archive, foreign).unwrap();
    let refused = run();
    vec![
        (
            "the archive and its missing parent directories are created",
            first.status.success() && created.lines().count() == 3,
        ),
        (
            "the rows are still printed with --archive",
            String::from_utf8_lossy(&merged.stdout).lines().count() == 3,
        ),
        (
            "one header, sessions kept, replaced or added, sorted by start",
            merged.status.success() && text.starts_with(COLUMNS) && ids == ["a", "c", "b"],
        ),
        (
            "an archived session is replaced by its new row",
            b_validations == "2",
        ),
        (
            "a resolved project and revision survive a new row that lost them",
            cell("c", "project") == "keep"
                && cell("c", "revision") == "rrr"
                && cell("c", "validations") == "1",
        ),
        (
            "an archive written before main_context_peak is upgraded, its rows given an empty peak",
            upgraded.status.success()
                && upgraded_rows.len() == 3
                && old
                    .and_then(|r| r.get("main_context_peak"))
                    .is_some_and(std::string::String::is_empty)
                && old
                    .and_then(|r| r.get("validations"))
                    .is_some_and(|v| v == "9"),
        ),
        (
            "an archive with another header is refused and left unchanged",
            !refused.status.success()
                && std::fs::read_to_string(&archive).unwrap_or_default() == foreign,
        ),
    ]
}

// spec: EVA-20
fn advisor_archive_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let tmp = tmp.join("advisor_archive");
    archived_sessions(&tmp);
    let archive = tmp.join("sessions.tsv");
    let before_advisor = format!(
        "{PRE_ADVISOR_COLUMNS}\nold\tp\t\t\t\t\t2026-01-01T00:00:00.000Z\t0\t0\t0\t0\t0\t0\t0\t0\t7\t\t\t9\t0\t0\t0\t0\n"
    );
    std::fs::write(&archive, before_advisor).unwrap();
    let upgraded = tool_command(dir.join("session_metrics.rs"))
        .arg(tmp.join("projects"))
        .arg("--archive")
        .arg(&archive)
        .output()
        .unwrap();
    let rows = rows(&std::fs::read_to_string(&archive).unwrap_or_default()).unwrap_or_default();
    let old = rows.get("old");
    vec![(
        "an archive written before the advisor columns is upgraded, its rows given empty advisor cells",
        upgraded.status.success()
            && rows.len() == 3
            && old.is_some_and(|r| {
                ["advisor_models", "advisor_calls", "advisor_input_tokens", "advisor_output_tokens"]
                    .iter()
                    .all(|c| r.get(*c).is_some_and(String::is_empty))
                    && r.get("main_context_peak").is_some_and(|v| v == "7")
                    && r.get("validations").is_some_and(|v| v == "9")
            }),
    )]
}

#[test]
fn session_metrics_archive() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    assert_checks(
        archive_checks(&dir, &tmp)
            .into_iter()
            .chain(advisor_archive_checks(&dir, &tmp)),
    );
}
