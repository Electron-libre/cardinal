//! The lock of a worktree (ADR 0022): the first Claude Code session that acts in a worktree holds it while its process
//! runs, and the others are refused their edits and commands, so two sessions never share its files, index and
//! review marker. The lock is the file `cardinal-session` of the git directory of the worktree, holding the
//! `session_id` of the hook event and the process of the session (`CLAUDE_PID`); a process that no longer runs frees
//! it, and so does a holder with no work in progress (ADR 0024): a clean worktree and no issue it claims, or a holder
//! whose transcript says Claude Code continued it in the calling session (ADR 0032), or a fork the user left to go
//! back to the calling session, which then also takes the `working` claims of that holder. Every change
//! of the lock happens under a mutex directory beside it, created atomically.
//! A gate for cooperative agents (ADR 0005): a broken input or environment lets the call through.
//! Usage: hooks `cardinal run workspace-lock check` (`PreToolUse`), `start` (`SessionStart`), `release` (`SessionEnd`).

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use super::issue_claim::{self, alive};

/// A mutex older than this was left by a process that died holding it.
const STALE_MUTEX: Duration = Duration::from_secs(10);

/// The paths of the work in progress a refusal lists at most.
const MAX_PATHS: usize = 10;

/// The holder of a lock.
#[derive(PartialEq)]
struct Holder {
    session: String,
    pid: String,
}

impl Holder {
    fn parse(text: &str) -> Option<Self> {
        let field = |key: &str| {
            text.lines()
                .find_map(|l| l.strip_prefix(key)?.strip_prefix(' '))
                .map(str::to_string)
        };
        Some(Holder {
            session: field("session")?,
            pid: field("pid")?,
        })
    }

    fn text(&self) -> String {
        format!("session {}\npid {}\n", self.session, self.pid)
    }
}

/// The path of the lock of the worktree of `cwd` (WSL-9: per worktree).
fn lock_path(cwd: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .args([
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "cardinal-session",
        ])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    Some(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

fn read(path: &Path) -> Option<Holder> {
    Holder::parse(&std::fs::read_to_string(path).ok()?)
}

/// The mutex of `lock`, removed when dropped.
struct Mutex(PathBuf);

impl Mutex {
    /// Wait up to two seconds for the mutex of `lock`, breaking one left by a dead process.
    fn take(lock: &Path) -> Option<Self> {
        let dir = lock.with_extension("mutex");
        for _ in 0..200 {
            if std::fs::create_dir(&dir).is_ok() {
                return Some(Mutex(dir));
            }
            let old = std::fs::metadata(&dir)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| SystemTime::now().duration_since(t).ok())
                .is_some_and(|age| age > STALE_MUTEX);
            if old {
                let _ = std::fs::remove_dir(&dir);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.0);
    }
}

/// The calling session: its holder line and the `transcript_path` of its hook input.
struct Caller<'a> {
    me: Holder,
    transcript: Option<&'a Path>,
}

/// Whether a holder lets the caller take its lock.
enum Verdict {
    /// The caller takes it (WSL-1, WSL-2, WSL-10).
    Free,
    /// The caller takes it, with its claims, as the session the holder was continued in or the user went back to from
    /// the holder (WSL-11, WSL-12, WSL-13).
    Continued,
    /// The holder keeps it for this work in progress (WSL-3).
    Kept(String),
}

/// Whether `holder` keeps the lock of the worktree of `cwd` from the caller: only another session of another process
/// that runs, has work in progress, was not continued in the caller and is not a session the user left to go back to
/// the caller (WSL-3, WSL-10, WSL-11, WSL-13).
fn keeps(holder: &Holder, caller: &Caller, cwd: &Path, lock: &Path) -> Verdict {
    let me = &caller.me;
    if holder.session == me.session || holder.pid == me.pid || !alive(&holder.pid) {
        return Verdict::Free;
    }
    let Some(work) = in_progress(holder, cwd) else {
        return Verdict::Free;
    };
    if continued_in(holder, caller) || returned_from(holder, caller, lock) {
        return Verdict::Continued;
    }
    Verdict::Kept(work)
}

/// Rewrite the `working` claims of the worktree of `cwd` that name the session or the process of `holder` (the claims
/// `in_progress` counts) to the caller's session and process, each by compare-and-swap with the value read (WSL-12).
fn hand_claims(holder: &Holder, caller: &Caller, cwd: &Path) {
    let canonical = |p: &Path| std::fs::canonicalize(p).ok();
    let Some(top) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| canonical(Path::new(String::from_utf8_lossy(&o.stdout).trim())))
    else {
        return;
    };
    for (stem, old, claim) in issue_claim::all(cwd) {
        let ours = claim.session == holder.session || claim.pid == holder.pid;
        let here = canonical(Path::new(&claim.worktree)).as_ref() == Some(&top);
        if claim.state == "working" && ours && here {
            let moved = issue_claim::Claim {
                session: caller.me.session.clone(),
                pid: caller.me.pid.clone(),
                ..claim
            };
            issue_claim::write(cwd, &stem, &moved, Some(&old));
        }
    }
}

/// What a transcript says of the sessions Claude Code continued it in (ADR 0032).
#[derive(Default)]
struct Continuations {
    /// The sessions its records `continued-in` name.
    sessions: Vec<String>,
    /// The timestamp of the first `user` or `assistant` record stamped after its last record `continued-in`: when the
    /// user went back to it.
    returned: Option<String>,
}

impl Continuations {
    /// Read the transcript at `path`; an unreadable one names no session. Lines that are not JSON are skipped.
    fn read(path: &Path) -> Self {
        let mut found = Continuations::default();
        let Ok(text) = std::fs::read(path) else {
            return found;
        };
        // The timestamp of the last record `continued-in`.
        let mut last: Option<String> = None;
        for line in text.split(|&b| b == b'\n') {
            // Before a `continued-in` record and once a return is found, only such a record matters: skip the others.
            let pending = last.is_some() && found.returned.is_none();
            if !pending && !line.windows(12).any(|w| w == b"continued-in") {
                continue;
            }
            let Ok(record) = serde_json::from_slice::<serde_json::Value>(line) else {
                continue;
            };
            let stamp = record["timestamp"].as_str().unwrap_or_default();
            match record["type"].as_str() {
                Some("continued-in") => {
                    // A record naming no session still ends the continuation before it: it matches no session.
                    let next = record["continuedInSessionId"].as_str().unwrap_or_default();
                    found.sessions.push(next.to_string());
                    last = Some(stamp.to_string());
                    found.returned = None;
                }
                Some("user" | "assistant")
                    if found.returned.is_none() && last.as_deref().is_some_and(|at| stamp > at) =>
                {
                    found.returned = Some(stamp.to_string());
                }
                _ => {}
            }
        }
        found
    }
}

/// Whether the transcript of `holder`, beside the caller's, says Claude Code continued it in the caller's session
/// (WSL-11): its last record `continued-in` names that session and no `user` or `assistant` record is stamped after
/// it (a record stamped later means the user went back to the holder).
fn continued_in(holder: &Holder, caller: &Caller) -> bool {
    let name = &holder.session;
    if name.is_empty() || name.contains(['/', '\\']) || name.contains("..") {
        return false;
    }
    let Some(dir) = caller.transcript.and_then(Path::parent) else {
        return false;
    };
    let found = Continuations::read(&dir.join(format!("{name}.jsonl")));
    let me = &caller.me.session;
    found.returned.is_none() && found.sessions.last() == Some(me) && !me.is_empty()
}

/// Whether the caller's own transcript says the user went back to it from `holder`, a session it was continued in
/// (WSL-13): a record `continued-in` names the holder, and the first `user` or `assistant` record stamped after the
/// last one is stamped after the holder took the lock at `lock` (its last write), so a holder that took it later,
/// once the caller had ended, keeps it.
fn returned_from(holder: &Holder, caller: &Caller, lock: &Path) -> bool {
    let Some(path) = caller.transcript else {
        return false;
    };
    let found = Continuations::read(path);
    let Some(returned) = found.returned.as_deref() else {
        return false;
    };
    let Ok(returned) = chrono::DateTime::parse_from_rfc3339(returned) else {
        return false;
    };
    let Ok(taken) = std::fs::metadata(lock).and_then(|m| m.modified()) else {
        return false;
    };
    found.sessions.contains(&holder.session)
        && returned > chrono::DateTime::<chrono::Utc>::from(taken)
}

/// The work in progress of `holder` (WSL-10), described for the refusal: the changes and untracked files of the
/// worktree of `cwd`, or an issue claimed `working` by its session or process; an unreadable status counts as work in
/// progress. Any change counts, whoever made it (ADR 0027).
fn in_progress(holder: &Holder, cwd: &Path) -> Option<String> {
    let status = Command::new("git")
        .args([
            "--no-optional-locks",
            "status",
            "--porcelain",
            "--untracked-files=normal",
        ])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success());
    let Some(status) = status else {
        return Some("its git status, which could not be read".to_string());
    };
    if !status.stdout.is_empty() {
        let lines: Vec<&[u8]> = status
            .stdout
            .trim_ascii_end()
            .split(|&b| b == b'\n')
            .collect();
        let mut paths: Vec<String> = lines
            .iter()
            .take(MAX_PATHS)
            .map(|l| format!("    {}", String::from_utf8_lossy(l)))
            .collect();
        if lines.len() > MAX_PATHS {
            paths.push(format!("    and {} more", lines.len() - MAX_PATHS));
        }
        return Some(format!("these uncommitted changes, whoever made them (commit them, `git stash -u` them or delete them):\n{}", paths.join("\n")));
    }
    issue_claim::all(cwd)
        .into_iter()
        .find(|(_, _, c)| {
            c.state == "working" && (c.session == holder.session || c.pid == holder.pid)
        })
        .map(|(stem, _, _)| {
            format!("the issue {stem} it claims, until that session closes it or ends")
        })
}

/// Take the lock at `path` for `me` (WSL-1, WSL-2, WSL-4, WSL-10, WSL-11, WSL-13), with the claims of a holder
/// continued in the caller or left for it (WSL-12): `Ok` when it holds it, else the holder that keeps it and its
/// work in progress.
fn take(path: &Path, caller: &Caller, cwd: &Path) -> Result<(), (Holder, String)> {
    let before = read(path);
    if before.as_ref() == Some(&caller.me) {
        return Ok(());
    }
    // Read git before the mutex, so a slow status never holds it; a holder that changed meanwhile is judged under it.
    let verdict = before.as_ref().map(|h| keeps(h, caller, cwd, path));
    let Some(_mutex) = Mutex::take(path) else {
        return Ok(());
    };
    let holder = read(path);
    let verdict = match (&holder, verdict) {
        (None, _) => Verdict::Free,
        (Some(h), Some(v)) if before.as_ref() == Some(h) => v,
        (Some(h), _) => keeps(h, caller, cwd, path),
    };
    match (holder, verdict) {
        (Some(h), Verdict::Kept(work)) => Err((h, work)),
        (holder, verdict) => {
            // The claims move first, under the mutex: until the lock names the caller, no other session sees a holder
            // without them, and a move cut short is done again by the next call (WSL-12).
            if let (Some(h), Verdict::Continued) = (&holder, verdict) {
                hand_claims(h, caller, cwd);
            }
            let tmp = path.with_extension(format!("tmp{}", std::process::id()));
            if std::fs::write(&tmp, caller.me.text()).is_ok() {
                let _ = std::fs::rename(&tmp, path);
            }
            Ok(())
        }
    }
}

/// Delete the lock at `path` when `session` holds it (WSL-6).
fn release(path: &Path, session: &str) {
    let Some(_mutex) = Mutex::take(path) else {
        return;
    };
    if read(path).is_some_and(|h| h.session == session) {
        let _ = std::fs::remove_file(path);
    }
}

pub(super) fn main(args: &[String]) -> u8 {
    let mode = args.first().map_or("", String::as_str);
    if !["check", "start", "release"].contains(&mode) {
        eprintln!("usage: workspace-lock check | start | release");
        return 2;
    }
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).ok();
    let Ok(event) = serde_json::from_str::<serde_json::Value>(&input) else {
        return 0;
    };
    let (Some(session), Some(cwd)) = (event["session_id"].as_str(), event["cwd"].as_str()) else {
        return 0;
    };
    let Some(pid) = std::env::var("CLAUDE_PID").ok().filter(|p| !p.is_empty()) else {
        return 0;
    };
    let Some(path) = lock_path(Path::new(cwd)) else {
        return 0;
    };
    if mode == "release" {
        release(&path, session);
        return 0;
    }
    let caller = Caller {
        me: Holder {
            session: session.to_string(),
            pid,
        },
        transcript: event["transcript_path"].as_str().map(Path::new),
    };
    let Err((holder, work)) = take(&path, &caller, Path::new(cwd)) else {
        return 0;
    };
    let me = format!(
        "this session ({}) of process {}",
        caller.me.session, caller.me.pid
    );
    if mode == "start" {
        println!("This worktree is busy: the Claude Code session {} of process {} works in it and still runs, not {me}. Your edits and commands will be refused here: tell the user to open another worktree or to end that session. It keeps the worktree for {work}", holder.session, holder.pid);
        return 0;
    }
    eprintln!("Refused: another live session ({}) of process {} works in this worktree, not {me}; two sessions would share its files, index and review marker. Stop and let the user open another worktree or end that session. It keeps the worktree for {work}", holder.session, holder.pid);
    2
}
