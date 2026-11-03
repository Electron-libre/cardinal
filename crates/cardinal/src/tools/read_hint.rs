//! `read-hint`, a `PreToolUse` hook on `Read`: before a whole read of a file over the threshold of rule
//! `oversized-result` (BRE-19), it denies the call once per file and session with a reason naming the size and the
//! range options, so the agent reads a range; the same call repeated goes through, the file being needed whole. It
//! lets go whenever it cannot judge or record, so it never blocks for its own failure. Spec: `specs/read-hint.md`.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// The threshold of rule `oversized-result` (BRE-19), in bytes.
const THRESHOLD: u64 = 25_000;

/// The extensions Claude Code renders rather than reads as text (RDH-2).
const RENDERED: [&str; 6] = ["png", "jpg", "jpeg", "gif", "webp", "pdf"];

/// The size of the file a whole read of `input` would return as text, when it is over the threshold (RDH-1, RDH-2).
fn oversized(input: &Value) -> Option<(&str, u64)> {
    if !input["offset"].is_null() || !input["limit"].is_null() {
        return None;
    }
    let path = input["file_path"].as_str()?;
    let rendered = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| RENDERED.contains(&e.to_ascii_lowercase().as_str()));
    let meta = fs::metadata(path).ok().filter(fs::Metadata::is_file)?;
    (!rendered && meta.len() > THRESHOLD).then_some((path, meta.len()))
}

/// The record of the files `session` was sent back from: one file per session under `TMPDIR` (RDH-5), its name
/// kept to the characters of a file name.
fn record_of(session: &str) -> PathBuf {
    let name: String = session
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    std::env::temp_dir().join("cardinal-read-hint").join(name)
}

/// Record `path` for the session, `false` when it was already there (RDH-3) or cannot be written (RDH-4).
fn first_time(record: &Path, path: &str) -> bool {
    if fs::read_to_string(record).is_ok_and(|text| text.lines().any(|line| line == path)) {
        return false;
    }
    record
        .parent()
        .is_some_and(|dir| fs::create_dir_all(dir).is_ok())
        && OpenOptions::new()
            .create(true)
            .append(true)
            .open(record)
            .and_then(|mut file| writeln!(file, "{path}"))
            .is_ok()
}

pub(super) fn main(_args: &[String]) -> u8 {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).ok();
    let Ok(event) = serde_json::from_str::<Value>(&input) else {
        return 0;
    };
    if event["hook_event_name"] != "PreToolUse" || event["tool_name"] != "Read" {
        return 0;
    }
    let Some(session) = event["session_id"].as_str().filter(|s| !s.is_empty()) else {
        return 0;
    };
    let Some((path, bytes)) = oversized(&event["tool_input"]) else {
        return 0;
    };
    if !first_time(&record_of(session), path) {
        return 0;
    }
    let reason = format!(
        "This file holds {bytes} bytes: read the range you need (`offset` and `limit`) or search it with `Grep`; \
         if you need it whole, repeat the same call, which then goes through."
    );
    let output = json!({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "deny",
        "permissionDecisionReason": reason,
    }});
    println!("{output}");
    0
}
