//! The scratch directory of a session (spec `specs/session-scratch.md`): `start` creates
//! `<tmp>/claude-scratch-<session_id>/` and exports it as `TMPDIR` to the `Bash` commands of the session through the
//! env file of Claude Code (`CLAUDE_ENV_FILE`), so `mktemp`, `tempfile` and `tmpdir()` write in it; `end` removes it.
//! `start` also sweeps the directories of the sessions killed before their `SessionEnd`. A file `cardinal-keep` in a
//! directory keeps it. The hooks never block a session: any failure leaves things as they are, prints nothing, exits 0.
//!
//! Usage: hooks `cardinal run session-scratch start` (`SessionStart`), `end` (`SessionEnd`).

use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::issue_claim::alive;

/// The name of a scratch directory before its session id.
const PREFIX: &str = "claude-scratch-";

/// The file that keeps a scratch directory after its session (SCR-5).
const KEEP: &str = "cardinal-keep";

/// The file of a scratch directory that records the process of its session (SCR-7).
const OWNER: &str = ".cardinal-session";

/// How long a directory of a dead session stays unchanged before the sweep removes it (SCR-3).
const STALE: Duration = Duration::from_hours(24);

pub(super) fn main(args: &[String]) -> u8 {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).ok();
    let Some(session) = session_id(&input) else {
        return 0;
    };
    let root = temp_root();
    match args.first().map(String::as_str) {
        Some("start") => start(&root, &session),
        Some("end") => end(&root.join(format!("{PREFIX}{session}"))),
        _ => {}
    }
    0
}

/// The `session_id` of the hook event, when it is safe as a file name (SCR-4).
fn session_id(input: &str) -> Option<String> {
    let event: serde_json::Value = serde_json::from_str(input).ok()?;
    let id = event["session_id"].as_str()?;
    let safe = !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    safe.then(|| id.to_string())
}

/// The temporary directory of the hook's environment, or its parent when it is itself a scratch directory (SCR-6).
fn temp_root() -> PathBuf {
    let tmp = std::env::temp_dir();
    let nested = tmp
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with(PREFIX));
    match tmp.parent() {
        Some(parent) if nested => parent.to_path_buf(),
        _ => tmp,
    }
}

fn start(root: &Path, session: &str) {
    let Some(env_file) = std::env::var_os("CLAUDE_ENV_FILE") else {
        return;
    };
    // with no process recorded, the sweep could not tell the session alive (SCR-4)
    let Some(pid) = std::env::var("CLAUDE_PID").ok().filter(|p| !p.is_empty()) else {
        return;
    };
    let dir = root.join(format!("{PREFIX}{session}"));
    let created = fs::DirBuilder::new().mode(0o700).create(&dir);
    if created.is_err() && !dir.is_dir() {
        return;
    }
    fs::write(dir.join(OWNER), format!("{pid}\n")).ok();
    let quoted = dir.to_string_lossy().replace('\'', r"'\''");
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(env_file)
        .and_then(|mut f| writeln!(f, "export TMPDIR='{quoted}'"))
        .ok();
    sweep(root, &dir);
}

fn end(dir: &Path) {
    if dir.is_dir() && !dir.join(KEEP).exists() {
        fs::remove_dir_all(dir).ok();
    }
}

/// Removes each scratch directory of `root` but `own` whose session is dead and that nothing changed in for a day,
/// unless it is kept (SCR-3).
fn sweep(root: &Path, own: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let Some(cutoff) = SystemTime::now().checked_sub(STALE) else {
        return;
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        let scratch = entry.file_name().to_string_lossy().starts_with(PREFIX)
            && entry.file_type().is_ok_and(|t| t.is_dir())
            && dir != own;
        if !scratch || dir.join(KEEP).exists() {
            continue;
        }
        let pid = fs::read_to_string(dir.join(OWNER)).unwrap_or_default();
        if alive(pid.trim()) || changed_since(&dir, cutoff) {
            continue;
        }
        fs::remove_dir_all(&dir).ok();
    }
}

/// Whether `path`, or anything under it, changed after `cutoff`; true when one cannot be read, so it is kept.
fn changed_since(path: &Path, cutoff: SystemTime) -> bool {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return true;
    };
    if meta.modified().map_or(true, |m| m > cutoff) {
        return true;
    }
    if !meta.is_dir() {
        return false;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return true;
    };
    entries
        .into_iter()
        .any(|e| e.map_or(true, |e| changed_since(&e.path(), cutoff)))
}
