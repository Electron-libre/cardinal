//! Behavior test for the hook `cardinal tool read-hint` (`specs/read-hint.md`): a `PreToolUse` event of `Read` on
//! stdin, files of a temp dir, and a temp `TMPDIR` for the record of the sessions sent back.

mod common;

use common::{output, put, Run};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

/// The threshold of rule `oversized-result` (BRE-19).
const THRESHOLD: usize = 25_000;

fn hook_raw(tmp: &Path, stdin: &str) -> Run {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.args(["tool", "read-hint"]).env("TMPDIR", tmp);
    output(cmd, stdin)
}

fn hook(tmp: &Path, event: &Value) -> Run {
    hook_raw(tmp, &event.to_string())
}

/// A `PreToolUse` event of `Read` by `session` with `input`.
fn read(session: &str, input: &Value) -> Value {
    json!({"hook_event_name": "PreToolUse", "session_id": session, "tool_name": "Read", "tool_input": input})
}

/// A file of `bytes` bytes named `name` in `dir`.
fn file(dir: &Path, name: &str, bytes: usize) -> PathBuf {
    put(dir, name, &"x".repeat(bytes));
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

// spec: RDH-1, RDH-3
#[test]
fn a_whole_read_of_a_large_file_is_sent_back_once() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let path = file(dir.path(), "big.rs", 30_000);
    let event = read("s1", &json!({"file_path": path}));
    let reason = denied(&hook(tmp.path(), &event));
    for word in ["30000 bytes", "`offset`", "`limit`", "`Grep`", "same call"] {
        assert!(reason.contains(word), "{word}: {reason}");
    }
    assert!(silent_pass(&hook(tmp.path(), &event)), "the repeat");
}

// spec: RDH-2
#[test]
fn a_bounded_small_missing_or_rendered_read_passes() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let big = file(dir.path(), "big.md", 30_000);
    let small = file(dir.path(), "small.md", THRESHOLD);
    let shot = file(dir.path(), "shot.PNG", 200_000);
    let pdf = file(dir.path(), "doc.pdf", 200_000);
    for (n, input) in [
        json!({"file_path": big, "limit": 200}),
        json!({"file_path": big, "offset": 10}),
        json!({"file_path": small}),
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
    let a = file(dir.path(), "a.rs", 30_000);
    let b = file(dir.path(), "b.rs", 30_000);
    denied(&hook(tmp.path(), &read("s1", &json!({"file_path": a}))));
    denied(&hook(tmp.path(), &read("s1", &json!({"file_path": b}))));
    denied(&hook(tmp.path(), &read("s2", &json!({"file_path": a}))));
    assert!(silent_pass(&hook(
        tmp.path(),
        &read("s1", &json!({"file_path": a}))
    )));
    let records = fs::read_dir(tmp.path().join("cardinal-read-hint"))
        .unwrap()
        .count();
    assert_eq!(records, 2, "one file per session");
}

// spec: RDH-4
#[test]
fn another_event_or_an_unrecordable_one_passes() {
    let dir = tempdir().unwrap();
    let tmp = tempdir().unwrap();
    let path = file(dir.path(), "big.rs", 30_000);
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
    let blocked = file(dir.path(), "tmp-file", 1);
    let run = hook(&blocked, &read("s1", &json!({"file_path": path})));
    assert!(silent_pass(&run), "{run:?}");
}

// spec: RDH-6
#[test]
fn the_source_declares_the_hook_and_its_feature() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let settings: Value = serde_json::from_str(
        &fs::read_to_string(root.join("entries/.claude/settings.json")).unwrap(),
    )
    .unwrap();
    let declared = settings["hooks"]["PreToolUse"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|group| group["matcher"] == "Read")
        .flat_map(|group| group["hooks"].as_array().unwrap())
        .any(|hook| hook["command"] == "cardinal run read-hint");
    assert!(declared, "{settings}");
    let features: toml::Table = fs::read_to_string(root.join("features.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let feature = features["read-hint"].as_table().unwrap();
    assert_eq!(
        feature["entries"],
        toml::Value::Array(vec!["hook PreToolUse Read cardinal run read-hint".into()])
    );
    assert!(feature.get("files").is_none(), "{feature:?}");
}
