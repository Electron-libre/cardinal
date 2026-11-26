//! Behavior test for `cardinal tool status-line` (`specs/status-line.md`): a status line input of Claude Code on
//! stdin, the usage file written under a throwaway `XDG_CACHE_HOME`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Runs `cardinal tool status-line` with `input` on stdin and `cache` as `XDG_CACHE_HOME`; returns (exit code,
/// stdout, stderr).
fn run(cache: &Path, input: &str) -> (Option<i32>, String, String) {
    run_in(cache, Path::new("."), input)
}

/// [`run`] with `dir` as the current directory.
fn run_in(cache: &Path, dir: &Path, input: &str) -> (Option<i32>, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "status-line"])
        .current_dir(dir)
        .env("XDG_CACHE_HOME", cache)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn usage_file(cache: &Path) -> PathBuf {
    cache.join("cardinal-ui").join("usage.json")
}

fn usage(cache: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(usage_file(cache)).unwrap()).unwrap()
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

const BOTH: &str = r#"{"session_id":"a","model":{"id":"m"},"rate_limits":{
    "five_hour":{"used_percentage":41.6,"resets_at":1790000000},
    "seven_day":{"used_percentage":13,"resets_at":1790400000}}}"#;

// spec: STL-H1
// spec: STL-1
// spec: STL-3
#[test]
fn both_limits_are_printed_and_recorded_with_the_time_they_were_written() {
    let dir = tempfile::tempdir().unwrap();
    let before = now();
    let (code, out, err) = run(dir.path(), BOTH);
    assert_eq!(code, Some(0), "{err}");
    assert_eq!(out, "5h 42% · 7d 13%\n");
    assert_eq!(err, "");
    let file = usage(dir.path());
    assert_eq!(file["version"], 1);
    assert_eq!(file["five_hour"]["used_percentage"], 41.6);
    assert_eq!(file["five_hour"]["resets_at"], 1_790_000_000);
    assert_eq!(file["seven_day"]["used_percentage"], 13);
    assert_eq!(file["seven_day"]["resets_at"], 1_790_400_000);
    let written = file["written_at"].as_u64().unwrap();
    assert!((before..=now()).contains(&written), "{written}");
    let left: Vec<_> = std::fs::read_dir(dir.path().join("cardinal-ui"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(left, ["usage.json"], "no temporary file is left");
}

// spec: STL-1
#[test]
fn a_limit_the_input_lacks_is_left_out_of_the_line_and_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let input = r#"{"rate_limits":{"five_hour":{"used_percentage":41.6,"resets_at":1790000000}}}"#;
    let (code, out, _) = run(dir.path(), input);
    assert_eq!(code, Some(0));
    assert_eq!(out, "5h 42%\n");
    assert!(usage(dir.path()).get("seven_day").is_none());
}

// spec: STL-2
#[test]
fn an_input_with_no_limit_leaves_the_file_and_prints_an_empty_line() {
    let dir = tempfile::tempdir().unwrap();
    run(dir.path(), BOTH);
    let before = std::fs::read_to_string(usage_file(dir.path())).unwrap();
    for input in [r#"{"session_id":"a"}"#, r#"{"rate_limits":{}}"#] {
        let (code, out, _) = run(dir.path(), input);
        assert_eq!(code, Some(0));
        assert_eq!(out, "\n", "{input}");
    }
    assert_eq!(
        std::fs::read_to_string(usage_file(dir.path())).unwrap(),
        before
    );
}

// spec: STL-3
#[test]
fn the_cardinal_ui_directory_is_created_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("deep").join("cache");
    let (code, _, _) = run(&cache, BOTH);
    assert_eq!(code, Some(0));
    assert!(usage_file(&cache).is_file());
}

// spec: STL-4
#[test]
fn an_input_that_is_not_json_prints_an_empty_line_and_exits_0() {
    let dir = tempfile::tempdir().unwrap();
    let (code, out, err) = run(dir.path(), "not json");
    assert_eq!((code, out.as_str(), err.as_str()), (Some(0), "\n", ""));
    assert!(!usage_file(dir.path()).exists());
}

// spec: STL-4
#[test]
fn a_limit_whose_fields_are_not_numbers_counts_as_missing() {
    let dir = tempfile::tempdir().unwrap();
    let input = r#"{"rate_limits":{
        "five_hour":{"used_percentage":"41","resets_at":1790000000},
        "seven_day":{"used_percentage":13,"resets_at":1790400000}}}"#;
    let (code, out, _) = run(dir.path(), input);
    assert_eq!(code, Some(0));
    assert_eq!(out, "7d 13%\n");
    assert!(usage(dir.path()).get("five_hour").is_none());
}

// spec: STL-4
#[test]
fn a_cache_that_cannot_be_written_still_prints_the_line() {
    let dir = tempfile::tempdir().unwrap();
    // a file where the cache directory should be: nothing can be created under it
    let cache = dir.path().join("cache");
    std::fs::write(&cache, "").unwrap();
    let (code, out, err) = run(&cache, BOTH);
    assert_eq!(
        (code, out.as_str(), err.as_str()),
        (Some(0), "5h 42% · 7d 13%\n", "")
    );
}

// spec: STL-1
#[test]
fn a_directory_outside_any_project_records_the_limits() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let (code, out, err) = run_in(dir.path(), elsewhere.path(), BOTH);
    assert_eq!(
        (code, out.as_str(), err.as_str()),
        (Some(0), "5h 42% · 7d 13%\n", "")
    );
    assert_eq!(usage(dir.path())["five_hour"]["used_percentage"], 41.6);
}

/// An input whose 5-hour limit is `used` resetting at `resets`, and no weekly one.
fn five_hour(used: f64, resets: u64) -> String {
    format!(
        r#"{{"rate_limits":{{"five_hour":{{"used_percentage":{used},"resets_at":{resets}}}}}}}"#
    )
}

// spec: STL-5
#[test]
fn an_older_answer_of_the_same_window_never_moves_the_file_back() {
    let dir = tempfile::tempdir().unwrap();
    run(dir.path(), &five_hour(55.0, 1_790_000_000));
    let (_, out, _) = run(dir.path(), &five_hour(40.0, 1_790_000_000));
    assert_eq!(out, "5h 40%\n");
    assert_eq!(usage(dir.path())["five_hour"]["used_percentage"], 55.0);
    // an answer of the window before also leaves it
    run(dir.path(), &five_hour(90.0, 1_789_982_000));
    assert_eq!(usage(dir.path())["five_hour"]["resets_at"], 1_790_000_000);
}

// spec: STL-5
#[test]
fn a_later_window_replaces_the_recorded_one() {
    let dir = tempfile::tempdir().unwrap();
    run(dir.path(), &five_hour(55.0, 1_790_000_000));
    run(dir.path(), &five_hour(3.0, 1_790_018_000));
    let file = usage(dir.path());
    assert_eq!(file["five_hour"]["used_percentage"], 3.0);
    assert_eq!(file["five_hour"]["resets_at"], 1_790_018_000);
}

// spec: STL-5
#[test]
fn a_limit_the_input_lacks_keeps_the_recorded_one() {
    let dir = tempfile::tempdir().unwrap();
    run(dir.path(), BOTH);
    run(dir.path(), &five_hour(50.0, 1_790_000_000));
    let file = usage(dir.path());
    assert_eq!(file["five_hour"]["used_percentage"], 50.0);
    assert_eq!(file["seven_day"]["used_percentage"], 13);
}
