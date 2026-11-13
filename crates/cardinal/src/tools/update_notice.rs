//! The hook that tells the user a newer release of the harness is out (spec `specs/update-notice.md`).
//!
//! Usage: hook `cardinal run update-notice` (`SessionStart`). It compares the latest `v<X.Y.Z>` tag of the git source
//! with the pinned `rev` and, when the pin does not contain it, prints a `systemMessage` the user reads and the agent
//! never sees. The query of the source runs at most once a day (its record lives in the cache, answered or not) and
//! under a deadline; any failure, CI, a resumed session, a linked worktree or uncommitted work print nothing. It always exits 0.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::run::project_root;
use crate::{cache_dir, clone_dir, hash, in_ci, is_full_hash, is_git_url, source_and_rev};

/// How long the hook may spend on its git commands, all of them together (UPN-8).
const DEADLINE: Duration = Duration::from_secs(3);
/// How long a query of the source stands (UPN-9).
const DAY: u64 = 24 * 3600;

pub(super) fn main(_args: &[String]) -> u8 {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).ok();
    if let Some(tag) = newer_release(&input) {
        let message = format!(
            "Cardinal {tag} is available: run `cardinal update --rev {tag}` to update this project."
        );
        println!("{}", serde_json::json!({ "systemMessage": message }));
    }
    0
}

/// The latest release of the source the pinned revision of the project of the hook's `cwd` does not contain, if any.
/// The cheap checks come first, so a project the hook skips pays for no `git status` nor query.
fn newer_release(input: &str) -> Option<String> {
    if in_ci() {
        return None;
    }
    let event: serde_json::Value = serde_json::from_str(input).ok()?;
    // a resumed, cleared or compacted session already had its notice (UPN-11)
    if event
        .get("source")
        .is_some_and(|source| source != "startup")
    {
        return None;
    }
    let cwd = Path::new(event["cwd"].as_str()?);
    let root = project_root(cwd, None).ok()?;
    let (source, rev) = source_and_rev(&root).ok()?;
    if !is_git_url(&source) {
        return None;
    }
    let end = Instant::now() + DEADLINE;
    let cache = cache_dir().ok()?;
    let clone = clone_dir(&cache, &source);
    let pinned = pinned_commit(&clone, &rev?, end)?;
    if !clean_main_worktree(cwd, end) {
        return None;
    }
    let (tag, commit) = latest_release(&cache, &source, end)?;
    (!contains(&clone, &pinned, &commit, end)?).then_some(tag)
}

/// Whether `cwd` is in the main worktree of its repository and `git status` lists nothing there (UPN-11). The status
/// takes no optional lock, so it never holds the index of the user's worktree.
fn clean_main_worktree(cwd: &Path, end: Instant) -> bool {
    let Some(dirs) = git_in(cwd, &["rev-parse", "--git-dir", "--git-common-dir"], end) else {
        return false;
    };
    let mut dirs = dirs.lines().map(|dir| fs::canonicalize(cwd.join(dir)).ok());
    let (Some(Some(git_dir)), Some(Some(common))) = (dirs.next(), dirs.next()) else {
        return false;
    };
    git_dir == common
        && git_in(cwd, &["--no-optional-locks", "status", "--porcelain"], end)
            .is_some_and(|status| status.trim().is_empty())
}

/// The commit `rev` resolves to in the clone of the source, `origin/<rev>` first as `sync` resolves it, so a branch
/// is read where the last fetch left it.
fn pinned_commit(clone: &Path, rev: &str, end: Instant) -> Option<String> {
    if rev.starts_with('-') {
        return None;
    }
    [format!("origin/{rev}"), rev.to_string()]
        .iter()
        .find_map(|candidate| commit_of(clone, candidate, end))
}

/// The commit `rev` names in the clone.
fn commit_of(clone: &Path, rev: &str, end: Instant) -> Option<String> {
    let rev = format!("{rev}^{{commit}}");
    git_in(clone, &["rev-parse", "--verify", "--quiet", &rev], end)
        .map(|commit| commit.trim().to_string())
}

/// Whether the pinned commit is the commit of the release or descends from it; `None` when it cannot be judged. The
/// clone holds every ancestor of the pinned commit, so a release commit it lacks is newer: nothing is fetched.
fn contains(clone: &Path, pinned: &str, commit: &str, end: Instant) -> Option<bool> {
    if pinned == commit {
        return Some(true);
    }
    if commit_of(clone, commit, end).is_none() {
        return Some(false);
    }
    let mut ancestor = git(clone);
    ancestor.args(["merge-base", "--is-ancestor", commit, pinned]);
    match within(ancestor, end)?.status.code()? {
        0 => Some(true),
        1 => Some(false),
        _ => None,
    }
}

/// The latest release of `source` and its commit: from the record of a query of the last day, else from a new query,
/// recorded whether it was answered or not (UPN-9, UPN-10).
fn latest_release(cache: &Path, source: &str, end: Instant) -> Option<(String, String)> {
    let record = record_path(cache, source);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    if let Some((time, release)) = fs::read_to_string(&record)
        .ok()
        .and_then(|text| parse_record(&text))
    {
        if now.saturating_sub(time) < DAY {
            return release;
        }
    }
    let mut query = Command::new("git");
    query.args(["ls-remote", "--tags", source]);
    if std::env::var_os("GIT_SSH_COMMAND").is_none() && std::env::var_os("GIT_SSH").is_none() {
        // an ssh source must never ask the terminal for a passphrase or a host key
        query.env(
            "GIT_SSH_COMMAND",
            "ssh -o BatchMode=yes -o ConnectTimeout=3",
        );
    }
    let release = succeeded(query, end).and_then(|listing| latest(&listing));
    let text = match &release {
        Some((tag, commit)) => format!("{now} {tag} {commit}\n"),
        None => format!("{now}\n"),
    };
    let _ = fs::create_dir_all(record.parent()?).and_then(|()| fs::write(&record, text));
    release
}

/// Where the record of the queries of `source` lives.
fn record_path(cache: &Path, source: &str) -> PathBuf {
    cache
        .join("cardinal")
        .join("update-notice")
        .join(&hash(source.as_bytes())[..12])
}

/// A record `<unix seconds> [<tag> <commit>]`: its time and the release it found; `None` when it has another shape.
fn parse_record(text: &str) -> Option<(u64, Option<(String, String)>)> {
    let mut words = text.split_whitespace();
    let time = words.next()?.parse().ok()?;
    let release = match (words.next(), words.next(), words.next()) {
        (Some(tag), Some(commit), None) if is_release(tag, commit) => {
            Some((tag.to_string(), commit.to_string()))
        }
        (None, None, None) => None,
        _ => return None,
    };
    Some((time, release))
}

/// Whether `tag` names a release and `commit` is a full commit hash.
fn is_release(tag: &str, commit: &str) -> bool {
    version(tag).is_some() && is_full_hash(commit)
}

/// The release of the greatest version in the output of `git ls-remote --tags`, with its commit (the peeled line of an
/// annotated tag).
fn latest(listing: &str) -> Option<(String, String)> {
    let mut best: Option<((u64, u64, u64), String, String)> = None;
    let mut peeled = std::collections::BTreeMap::new();
    let mut tags = Vec::new();
    for line in listing.lines() {
        let Some((object, name)) = line.split_once('\t') else {
            continue;
        };
        let Some(name) = name.strip_prefix("refs/tags/") else {
            continue;
        };
        match name.strip_suffix("^{}") {
            Some(tag) => {
                peeled.insert(tag.to_string(), object.to_string());
            }
            None => tags.push((name.to_string(), object.to_string())),
        }
    }
    for (tag, object) in tags {
        let Some(version) = version(&tag) else {
            continue;
        };
        if best
            .as_ref()
            .is_none_or(|(greatest, _, _)| version > *greatest)
        {
            let commit = peeled.get(&tag).cloned().unwrap_or(object);
            if is_full_hash(&commit) {
                best = Some((version, tag, commit));
            }
        }
    }
    best.map(|(_, tag, commit)| (tag, commit))
}

/// The `(X, Y, Z)` of a tag `v<X.Y.Z>` (UPN-5).
fn version(tag: &str) -> Option<(u64, u64, u64)> {
    let mut parts = tag.strip_prefix('v')?.split('.').map(|part| {
        (!part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
            .then(|| part.parse().ok())
            .flatten()
    });
    let version = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(version)
}

/// `git -C <dir>`.
fn git(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(dir);
    cmd
}

/// The stdout of `git -C <dir> <args>` when it succeeds before `end`.
fn git_in(dir: &Path, args: &[&str], end: Instant) -> Option<String> {
    let mut cmd = git(dir);
    cmd.args(args);
    succeeded(cmd, end)
}

/// The stdout of `cmd` when it succeeds before `end`.
fn succeeded(cmd: Command, end: Instant) -> Option<String> {
    let out = within(cmd, end)?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Run `cmd` with no input; `None` when it cannot start, or when it still runs or its output is still open at `end`
/// (a process it left behind may hold the pipe), then killed (UPN-8).
fn within(mut cmd: Command, end: Instant) -> Option<Output> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env("GIT_TERMINAL_PROMPT", "0")
        .spawn()
        .ok()?;
    // read in a thread, so a full pipe never stalls the child before the deadline
    let mut stdout = child.stdout.take()?;
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = sender.send(stdout.read_to_end(&mut bytes).map(|_| bytes));
    });
    let status = loop {
        if let Some(status) = child.try_wait().ok()? {
            break status;
        }
        if Instant::now() >= end {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let stdout = receiver
        .recv_timeout(end.saturating_duration_since(Instant::now()))
        .ok()?
        .ok()?;
    Some(Output {
        status,
        stdout,
        stderr: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: UPN-8 (internal: a source that never answers cannot be set up offline through the binary)
    #[test]
    fn a_query_that_does_not_answer_is_given_up_at_the_deadline() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "sleep 10"]);
        let start = Instant::now();
        assert!(within(cmd, Instant::now() + Duration::from_millis(200)).is_none());
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    // spec: UPN-8 (internal: a process that keeps the output open cannot be set up offline through the binary)
    #[test]
    fn an_output_left_open_by_a_child_is_given_up_at_the_deadline() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "sleep 10 & echo started"]);
        let start = Instant::now();
        assert!(within(cmd, Instant::now() + Duration::from_millis(200)).is_none());
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn a_version_is_three_decimal_numbers_after_a_v() {
        assert_eq!(version("v5.10.0"), Some((5, 10, 0)));
        for tag in [
            "v6.0.0-rc1",
            "latest",
            "v6.0",
            "v1.2.3.4",
            "5.1.0",
            "v1..2",
            "v+1.2.3",
        ] {
            assert_eq!(version(tag), None, "{tag}");
        }
    }
}
