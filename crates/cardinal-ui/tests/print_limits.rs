//! Behavior test for `cardinal-ui --print`: the usage limits of the Claude plan on the top border of the worktrees
//! panel (STU-37), read from the usage file `cardinal run status-line` writes (STL-3) under a fake cache, in UTC.

mod common;

use common::{line, ok, project, ui, Project, QUEUE, ROADMAP};
use std::path::PathBuf;

fn now() -> i64 {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap()
}

/// The `HH:MM` (UTC) of `seconds`, its date before it when it is not today, as STU-13 words it.
fn clock(seconds: i64) -> String {
    let t = chrono::DateTime::from_timestamp(seconds, 0).unwrap();
    let today = chrono::Utc::now().date_naive();
    if t.date_naive() == today {
        t.format("%H:%M").to_string()
    } else {
        t.format("%Y-%m-%d %H:%M").to_string()
    }
}

/// The fake `cardinal` of `p` and the usage file of its cache (`XDG_CACHE_HOME` of `common::command`).
fn setup(p: &Project) -> (PathBuf, PathBuf) {
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let usage = cardinal
        .parent()
        .unwrap()
        .join("cache")
        .join("cardinal-ui")
        .join("usage.json");
    std::fs::create_dir_all(usage.parent().unwrap()).unwrap();
    (cardinal, usage)
}

/// The top border of the worktrees panel of `out`.
fn border(out: &str) -> &str {
    line(out, "┌ Worktrees ")
}

// spec: STU-H10
// spec: STU-37
#[test]
fn both_limits_their_resets_and_the_time_they_were_read_end_the_worktrees_border() {
    let p = project();
    let (cardinal, usage) = setup(&p);
    let (five, week, read) = (now() + 3600, now() + 4 * 86_400, now() - 60);
    std::fs::write(
        &usage,
        format!(
            r#"{{"version":1,"written_at":{read},
            "five_hour":{{"used_percentage":41.6,"resets_at":{five}}},
            "seven_day":{{"used_percentage":13,"resets_at":{week}}}}}"#
        ),
    )
    .unwrap();
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let expected = format!(
        " 5h used 42% · resets {} · 7d used 13% · resets {} · read {} ┐",
        clock(five),
        clock(week),
        clock(read)
    );
    assert!(border(&out).ends_with(&expected), "{out}");
}

// spec: STU-37
#[test]
fn no_usage_file_reads_limits_unknown() {
    let p = project();
    let (cardinal, _) = setup(&p);
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(border(&out).ends_with("─ limits unknown ┐"), "{out}");
}

// spec: STU-37
#[test]
fn a_usage_file_that_is_not_json_or_of_another_version_reads_limits_unknown() {
    let p = project();
    let (cardinal, usage) = setup(&p);
    for text in [
        "not json".to_string(),
        format!(r#"{{"version":2,"written_at":{}}}"#, now()),
    ] {
        std::fs::write(&usage, &text).unwrap();
        let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
        assert!(
            border(&out).ends_with("─ limits unknown ┐"),
            "{text}: {out}"
        );
    }
}

// spec: STU-37
#[test]
fn a_limit_that_reset_or_that_the_file_lacks_reads_unknown() {
    let p = project();
    let (cardinal, usage) = setup(&p);
    let (week, read) = (now() + 86_400, now() - 7200);
    std::fs::write(
        &usage,
        format!(
            r#"{{"version":1,"written_at":{read},
            "five_hour":{{"used_percentage":90,"resets_at":{}}},
            "seven_day":{{"used_percentage":13.5,"resets_at":{week}}}}}"#,
            now() - 3600
        ),
    )
    .unwrap();
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let expected = format!(
        " 5h unknown · 7d used 14% · resets {} · read {} ┐",
        clock(week),
        clock(read)
    );
    assert!(border(&out).ends_with(&expected), "{out}");

    std::fs::write(
        &usage,
        format!(r#"{{"version":1,"written_at":{read},"five_hour":{{"used_percentage":5,"resets_at":{week}}}}}"#),
    )
    .unwrap();
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(
        border(&out).ends_with(&format!(" · 7d unknown · read {} ┐", clock(read))),
        "{out}"
    );
}
