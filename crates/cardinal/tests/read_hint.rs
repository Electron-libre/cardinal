//! Behavior test for the hook `cardinal tool read-hint` (`specs/read-hint.md`): a `PreToolUse` event of `Read` on
//! stdin, files of a temp dir, and a temp `TMPDIR` (no `XDG_RUNTIME_DIR`, `USER` `tester`) for the record of the
//! sessions sent back.

mod common;

use common::{output, put, Run};
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

/// The directory of the records under the test's `TMPDIR`.
const RECORDS: &str = "cardinal-read-hint-tester";

fn hook_raw(tmp: &Path, stdin: &str) -> Run {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.args(["tool", "read-hint"])
        .env("TMPDIR", tmp)
        .env("USER", "tester")
        .env_remove("XDG_RUNTIME_DIR");
    output(cmd, stdin)
}

fn hook(tmp: &Path, event: &Value) -> Run {
    hook_raw(tmp, &event.to_string())
}

/// A `PreToolUse` event of `Read` by `session` with `input`.
fn read(session: &str, input: &Value) -> Value {
    json!({"hook_event_name": "PreToolUse", "session_id": session, "tool_name": "Read", "tool_input": input})
}

/// A file named `name` in `dir` of `lines` lines of `width` bytes, newline included.
fn file(dir: &Path, name: &str, lines: usize, width: usize) -> PathBuf {
    put(
        dir,
        name,
        &format!("{}\n", "x".repeat(width - 1)).repeat(lines),
    );
    dir.join(name)
}

fn silent_pass(run: &Run) -> bool {
    run.code == Some(0) && run.stderr.is_empty() && run.stdout.is_empty()
}

/// The reason of a deny, or a failure when the run is not one.
fn denied(run: &Run) -> String {
    assert_eq!(run.code, Some(0), "{run:?}");
    let output: Value = serde_json::from_str(&run.stdout).unwrap_or_else(|_| panic!("{run:?}"));
    let specific = &output["hookSpecificOutput"];
    assert_eq!(specific["hookEventName"], "PreToolUse", "{run:?}");
    assert_eq!(specific["permissionDecision"], "deny", "{run:?}");
    specific["permissionDecisionReason"]
        .as_str()
        .unwrap()
        .to_string()
}

// spec: RDH-1, RDH-3, RDH-7
#[test]
fn a_whole_read_of_a_large_file_is_sent_back_once() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    // 300 lines of 100 bytes and their 7 bytes of line number
    let path = file(dir.path(), "big.rs", 300, 100);
    let event = read("s1", &json!({"file_path": path}));
    let reason = denied(&hook(tmp.path(), &event));
    for word in ["32100 bytes", "`offset`", "`limit`", "`Grep`", "same call"] {
        assert!(reason.contains(word), "{word}: {reason}");
    }
    assert!(silent_pass(&hook(tmp.path(), &event)), "the repeat");
}

// spec: RDH-2, RDH-7
#[test]
fn a_bounded_small_missing_or_rendered_read_passes() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let big = file(dir.path(), "big.md", 300, 100);
    // 233 lines of 107 bytes: 24,931, at most the threshold
    let small = file(dir.path(), "small.md", 233, 100);
    // one line cut at 2,000 characters, and 3,000 lines of which 2,000 are read
    let minified = file(dir.path(), "app.min.js", 1, 200_000);
    let long = file(dir.path(), "long.txt", 3_000, 5);
    // over the size Claude Code reads whole
    let journal = file(dir.path(), "big.log", 4_000, 100);
    let shot = file(dir.path(), "shot.PNG", 300, 100);
    let pdf = file(dir.path(), "doc.pdf", 300, 100);
    let notebook = file(dir.path(), "a.ipynb", 300, 100);
    let locked = file(dir.path(), "locked.md", 300, 100);
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    for (n, input) in [
        json!({"file_path": big, "limit": 200}),
        json!({"file_path": big, "offset": 10}),
        json!({"file_path": small}),
        json!({"file_path": minified}),
        json!({"file_path": long}),
        json!({"file_path": journal}),
        json!({"file_path": notebook}),
        json!({"file_path": locked}),
        json!({"file_path": dir.path().join("missing.md")}),
        json!({"file_path": dir.path()}),
        json!({"file_path": shot}),
        json!({"file_path": pdf}),
    ]
    .into_iter()
    .enumerate()
    {
        let run = hook(tmp.path(), &read(&format!("s{n}"), &input));
        assert!(silent_pass(&run), "{input}: {run:?}");
    }
}

// spec: RDH-3, RDH-5
#[test]
fn the_record_is_per_session_under_the_temporary_directory() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let a = file(dir.path(), "a.rs", 300, 100);
    let b = file(dir.path(), "b.rs", 300, 100);
    denied(&hook(tmp.path(), &read("s1", &json!({"file_path": a}))));
    denied(&hook(tmp.path(), &read("s1", &json!({"file_path": b}))));
    denied(&hook(tmp.path(), &read("s2", &json!({"file_path": a}))));
    assert!(silent_pass(&hook(
        tmp.path(),
        &read("s1", &json!({"file_path": a}))
    )));
    let records = tmp.path().join(RECORDS);
    assert_eq!(
        fs::read_dir(&records).unwrap().count(),
        2,
        "one file per session"
    );
    let mode = fs::metadata(&records).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
}

// spec: RDH-5
#[test]
fn the_record_goes_to_the_runtime_directory_when_set() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let runtime = tempdir().unwrap();
    let path = file(dir.path(), "a.rs", 300, 100);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.args(["tool", "read-hint"])
        .env("TMPDIR", tmp.path())
        .env("XDG_RUNTIME_DIR", runtime.path());
    denied(&output(
        cmd,
        &read("s1", &json!({"file_path": path})).to_string(),
    ));
    assert!(runtime.path().join("cardinal-read-hint/s1").is_file());
    assert!(!tmp.path().join(RECORDS).exists());
}

// spec: RDH-4, RDH-5
#[test]
fn a_record_directory_others_may_enter_is_not_used() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let path = file(dir.path(), "a.rs", 300, 100);
    let records = tmp.path().join(RECORDS);
    fs::create_dir(&records).unwrap();
    fs::set_permissions(&records, fs::Permissions::from_mode(0o777)).unwrap();
    let run = hook(tmp.path(), &read("s1", &json!({"file_path": path})));
    assert!(silent_pass(&run), "{run:?}");
}

// spec: RDH-4
#[test]
fn another_event_or_an_unrecordable_one_passes() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let path = file(dir.path(), "big.rs", 300, 100);
    let mut bash = read("s1", &json!({"file_path": path}));
    bash["tool_name"] = json!("Bash");
    let mut post = read("s1", &json!({"file_path": path}));
    post["hook_event_name"] = json!("PostToolUse");
    let mut anonymous = read("s1", &json!({"file_path": path}));
    anonymous.as_object_mut().unwrap().remove("session_id");
    for event in [bash, post, anonymous] {
        assert!(silent_pass(&hook(tmp.path(), &event)), "{event}");
    }
    assert!(silent_pass(&hook_raw(tmp.path(), "not json")));
    // a `TMPDIR` that is a file: the record cannot be written
    let blocked = file(dir.path(), "tmp-file", 1, 1);
    let run = hook(&blocked, &read("s1", &json!({"file_path": path})));
    assert!(silent_pass(&run), "{run:?}");
}

/// The parts a reason names, as `(offset, limit)` pairs in order.
fn parts(reason: &str) -> Vec<(usize, usize)> {
    reason
        .split("`offset` ")
        .skip(1)
        .filter_map(|part| {
            let (offset, rest) = part.split_once(" `limit` ")?;
            let limit: String = rest.chars().take_while(char::is_ascii_digit).collect();
            Some((offset.parse().ok()?, limit.parse().ok()?))
        })
        .collect()
}

/// A directory `tool-results` in `dir`, where Claude Code saves an output too large to return.
fn saved_dir(dir: &Path) -> PathBuf {
    let saved = dir.join("tool-results");
    fs::create_dir_all(&saved).unwrap();
    saved
}

// spec: RDH-8, RDH-9
#[test]
fn a_saved_result_is_sent_back_every_time_with_its_parts() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    // 600 lines of 99 bytes, 106 bytes each with the line number: 63,600, parts of 188 lines
    let path = file(&saved_dir(dir.path()), "b1.txt", 600, 99);
    let whole = read("s1", &json!({"file_path": path}));
    for _ in 0..2 {
        let reason = denied(&hook(tmp.path(), &whole));
        assert!(reason.contains("63600 bytes"), "{reason}");
        assert_eq!(
            parts(&reason),
            [(1, 188), (189, 188), (377, 188), (565, 36)],
            "{reason}"
        );
    }
    let part = read(
        "s1",
        &json!({"file_path": path, "offset": 189, "limit": 188}),
    );
    assert!(silent_pass(&hook(tmp.path(), &part)));
    let open_ended = read("s1", &json!({"file_path": path, "offset": 189}));
    let reason = denied(&hook(tmp.path(), &open_ended));
    // lines 189 to 600: 412 lines of 106 bytes
    assert!(reason.contains("43672 bytes"), "{reason}");
    assert!(
        !tmp.path().join(RECORDS).join("s1").exists(),
        "nothing recorded"
    );
}

// spec: RDH-9
#[test]
fn the_parts_of_a_saved_diff_keep_whole_file_diffs_where_they_fit() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    // file diffs of a header and lines of 92 characters, 100 bytes with the line number: 8,000, 9,000 and 10,000
    let diff = |name: &str, lines: usize| {
        let header = format!("diff --git a/{name} b/{name}");
        let header = format!("{header}{}\n", " ".repeat(92 - header.len()));
        header + &format!("+{}\n", "x".repeat(91)).repeat(lines - 1)
    };
    let text = diff("a.rs", 80) + &diff("b.rs", 90) + &diff("c.rs", 100);
    let saved = saved_dir(dir.path());
    put(&saved, "d1.txt", &text);
    let reason = denied(&hook(
        tmp.path(),
        &read("s1", &json!({"file_path": saved.join("d1.txt")})),
    ));
    assert_eq!(parts(&reason), [(1, 170), (171, 100)], "{reason}");
    // two file diffs that fill a part exactly, then a third: the part ends on the file boundary
    let text = diff("a.rs", 100) + &diff("b.rs", 100) + &diff("c.rs", 100);
    put(&saved, "d4.txt", &text);
    let reason = denied(&hook(
        tmp.path(),
        &read("s1", &json!({"file_path": saved.join("d4.txt")})),
    ));
    assert_eq!(parts(&reason), [(1, 200), (201, 100)], "{reason}");
    // one file diff of 300 lines of 100 bytes: cut where its lines reach 20,000 bytes
    put(&saved, "d2.txt", &diff("big.rs", 300));
    let reason = denied(&hook(
        tmp.path(),
        &read("s1", &json!({"file_path": saved.join("d2.txt")})),
    ));
    assert_eq!(parts(&reason), [(1, 200), (201, 100)], "{reason}");
    // lines of 3,000 characters cut at 2,000, 2,008 bytes with the line number: 9 lines a part
    let long = format!("{}\n", "y".repeat(3_000));
    put(&saved, "d3.txt", &long.repeat(30));
    let reason = denied(&hook(
        tmp.path(),
        &read("s1", &json!({"file_path": saved.join("d3.txt")})),
    ));
    assert_eq!(
        parts(&reason),
        [(1, 9), (10, 9), (19, 9), (28, 3)],
        "{reason}"
    );
}

// spec: RDH-8
#[test]
fn a_text_file_outside_a_saved_result_directory_is_sent_back_once() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let path = file(dir.path(), "notes.txt", 300, 100);
    let not_txt = file(&saved_dir(dir.path()), "b2.json", 300, 100);
    for path in [path, not_txt] {
        let event = read("s1", &json!({"file_path": path}));
        let reason = denied(&hook(tmp.path(), &event));
        assert!(parts(&reason).is_empty(), "{reason}");
        assert!(silent_pass(&hook(tmp.path(), &event)), "the repeat");
        let ranged = read("s1", &json!({"file_path": path, "offset": 1}));
        assert!(silent_pass(&hook(tmp.path(), &ranged)));
    }
}

// spec: RDH-8
#[test]
fn a_large_saved_result_and_a_long_range_are_sent_back() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let saved = saved_dir(dir.path());
    // 4,000 lines of 100 bytes: over the size Claude Code reads whole, 2,000 lines of it read with `limit`
    let large = file(&saved, "b3.txt", 4_000, 100);
    let reason = denied(&hook(
        tmp.path(),
        &read("s1", &json!({"file_path": large, "limit": 2000})),
    ));
    assert_eq!(parts(&reason).last(), Some(&(3_907, 94)), "{reason}");
    // 6,000 lines of 11 bytes: 2,000 of them hold 22,000 bytes, 6,000 hold 66,000
    let long = file(&saved, "b4.txt", 6_000, 4);
    assert!(silent_pass(&hook(
        tmp.path(),
        &read("s1", &json!({"file_path": long}))
    )));
    let reason = denied(&hook(
        tmp.path(),
        &read("s1", &json!({"file_path": long, "limit": 6000})),
    ));
    assert!(reason.contains("this read about 66000 bytes"), "{reason}");
}
