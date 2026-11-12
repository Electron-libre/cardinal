//! `read-hint`, a `PreToolUse` hook on `Read`: before a whole read whose result would be over the threshold of rule
//! `oversized-result` (BRE-19), it denies the call once per file and session with a reason naming the size and the
//! range options, so the agent reads a range; the same call repeated goes through, the file being needed whole. An
//! output Claude Code saved under `tool-results/` (most often a reviewer's diff) is denied on every read too large,
//! with the parts to read instead. It lets go whenever it cannot judge or record, so it never blocks for its own
//! failure. Spec: `specs/read-hint.md`.

use super::rule_breaches::OVERSIZED;
use serde_json::{json, Value};
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// The start of the reason of a deny (RDH-1), by which `session-metrics` names its refusals (EVA-5): Claude Code
/// writes a deny of the JSON output without the command of the hook.
pub(super) const REASON_START: &str = "A whole read of this file returns about";

/// The size above which Claude Code refuses a whole read itself (RDH-1).
const READ_CAP: u64 = 262_144;

/// The lines a whole read returns, the characters it keeps of each, and the bytes of the number it prefixes (RDH-7).
const READ_LINES: usize = 2_000;
const LINE_CHARS: usize = 2_000;
const NUMBER_BYTES: usize = 7;

/// The extensions Claude Code renders rather than reads as text (RDH-2).
const RENDERED: [&str; 7] = ["png", "jpg", "jpeg", "gif", "webp", "pdf", "ipynb"];

/// The estimate of a part of a saved result (RDH-9), a margin under the threshold since RDH-7 is an estimate.
const PART_BYTES: usize = 20_000;

/// The line by which a part of a saved diff would rather start (RDH-9).
const FILE_DIFF: &str = "diff --git ";

/// The bytes a read returns of `line` (RDH-7).
fn line_bytes(line: &str) -> usize {
    let kept = line
        .char_indices()
        .nth(LINE_CHARS)
        .map_or(line.len(), |(at, _)| at);
    kept + 1 + NUMBER_BYTES
}

/// The bytes a read of `lines` returns (RDH-7), all of them: the caller takes the lines read.
fn estimate<'a>(lines: impl Iterator<Item = &'a str>) -> usize {
    lines.map(line_bytes).sum()
}

/// Whether `path` is an output Claude Code saved because it was too large (RDH-8).
fn saved(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "txt")
        && path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|d| d == "tool-results")
}

/// The text of `path` when it is a regular file Claude Code reads as text, and whole unless it is a saved result
/// (RDH-1, RDH-2, RDH-8).
fn text_of(path: &Path, saved: bool) -> Option<String> {
    let rendered = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| RENDERED.contains(&e.to_ascii_lowercase().as_str()));
    let mut file = File::open(path).ok()?;
    let meta = file.metadata().ok().filter(fs::Metadata::is_file)?;
    if rendered || (!saved && meta.len() > READ_CAP) {
        return None;
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(
        String::from_utf8(bytes)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()),
    )
}

/// The parts of `lines` as 1-based `(offset, limit)`: runs of at most `PART_BYTES`, ended before the last file diff
/// of the run when the run stops before the end and not on a file diff (RDH-9).
fn parts(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut parts = Vec::new();
    let mut start = 0;
    while start < lines.len() {
        let (mut end, mut bytes, mut last_diff) = (start, 0, None);
        while end < lines.len() {
            let more = line_bytes(lines[end]);
            if end > start && bytes + more > PART_BYTES {
                break;
            }
            if end > start && lines[end].starts_with(FILE_DIFF) {
                last_diff = Some(end);
            }
            bytes += more;
            end += 1;
        }
        if end < lines.len() && !lines[end].starts_with(FILE_DIFF) {
            end = last_diff.unwrap_or(end);
        }
        parts.push((start + 1, end - start));
        start = end;
    }
    parts
}

/// The reason of a deny of a read of a saved result over the threshold, `None` when the read is not (RDH-8).
fn saved_reason(input: &Value, text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let number = |key: &str, absent: usize| {
        input[key]
            .as_u64()
            .map_or(absent, |n| usize::try_from(n).unwrap_or(usize::MAX))
    };
    let from = number("offset", 1).saturating_sub(1);
    let range = lines
        .iter()
        .copied()
        .skip(from)
        .take(number("limit", READ_LINES));
    let bytes = estimate(range);
    if bytes <= OVERSIZED {
        return None;
    }
    let whole = estimate(lines.iter().copied().take(READ_LINES));
    let this = if input["offset"].is_null() && input["limit"].is_null() {
        String::new()
    } else {
        format!(", this read about {bytes} bytes")
    };
    let parts: Vec<String> = parts(&lines)
        .iter()
        .map(|(offset, limit)| format!("`offset` {offset} `limit` {limit}"))
        .collect();
    Some(format!(
        "{REASON_START} {whole} bytes{this}: it is an output saved because it was too large; read it in these parts, \
         each under the threshold, the ones you need or all of them in turn: {}.",
        parts.join("; ")
    ))
}

/// The estimate of the result of a whole read of `text`, when it is over the threshold (RDH-1, RDH-2).
fn oversized(text: &str) -> Option<usize> {
    let bytes = estimate(text.lines().take(READ_LINES));
    (bytes > OVERSIZED).then_some(bytes)
}

/// The directory of the records: `cardinal-read-hint` of `XDG_RUNTIME_DIR`, else `cardinal-read-hint-<USER>` of the
/// temporary directory, created with mode 0700; `None` when its mode is another, such as one that lets its group or others in (RDH-5).
fn records() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|d| !d.is_empty())
        .map_or_else(
            || {
                let user = std::env::var("USER").unwrap_or_default();
                std::env::temp_dir().join(format!("cardinal-read-hint-{user}"))
            },
            |runtime| PathBuf::from(runtime).join("cardinal-read-hint"),
        );
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .ok()?;
    let meta = fs::symlink_metadata(&dir).ok()?;
    (meta.is_dir() && meta.permissions().mode() & 0o777 == 0o700).then_some(dir)
}

/// The record of the files `session` was sent back from, its name kept to the characters of a file name.
fn record_of(dir: &Path, session: &str) -> PathBuf {
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
    dir.join(name)
}

/// Record `path` for the session, `false` when it was already there (RDH-3) or cannot be written (RDH-4).
fn first_time(record: &Path, path: &str) -> bool {
    if fs::read_to_string(record).is_ok_and(|text| text.lines().any(|line| line == path)) {
        return false;
    }
    OpenOptions::new()
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
    match reason(&event["tool_input"], session) {
        Some(reason) => deny(&reason),
        None => 0,
    }
}

/// The reason to deny the read of `input` by `session`, `None` to let it through (RDH-1 to RDH-3, RDH-8).
fn reason(input: &Value, session: &str) -> Option<String> {
    let path = input["file_path"].as_str()?;
    let is_saved = saved(Path::new(path));
    if !is_saved && (!input["offset"].is_null() || !input["limit"].is_null()) {
        return None;
    }
    let text = text_of(Path::new(path), is_saved)?;
    if is_saved {
        return saved_reason(input, &text);
    }
    let bytes = oversized(&text)?;
    let dir = records()?;
    first_time(&record_of(&dir, session), path).then(|| {
        format!(
            "{REASON_START} {bytes} bytes: read the range you need (`offset` and `limit`) or search it with `Grep`; \
             if you need it whole, repeat the same call, which then goes through."
        )
    })
}

/// Print the deny of RDH-1 with `reason`.
fn deny(reason: &str) -> u8 {
    let output = json!({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "deny",
        "permissionDecisionReason": reason,
    }});
    println!("{output}");
    0
}
