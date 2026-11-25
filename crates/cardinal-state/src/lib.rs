//! The rules `cardinal` and `cardinal-ui` both apply to the state of a worktree on disk (ADR 0033): the `key value`
//! lines of a lock, a claim or a mark, and whether the session that wrote a lock or a claim still runs (WSL-1, CLM-11,
//! STU-7). Runs no git: each binary runs it through its own runner. Depends on nothing but `std`.

pub mod worktree;

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

/// A process that started more than this after a lock or a claim was written took the number of its session.
const PID_REUSE_MARGIN: Duration = Duration::from_secs(2);

/// The value of the first line `<key> <value>` of `text`.
#[must_use]
pub fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .find_map(|l| l.strip_prefix(key)?.strip_prefix(' '))
}

/// The holder of the lock of a worktree (WSL-9): the file `cardinal-session` of its git directory.
#[derive(Clone, Debug, PartialEq)]
pub struct Lock {
    pub session: String,
    pub pid: String,
}

impl Lock {
    /// The lock of `text`, `None` without its `session` or its `pid` line.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Some(Lock {
            session: field(text, "session")?.to_string(),
            pid: field(text, "pid")?.to_string(),
        })
    }

    /// The lock read from the file `path`, `None` when it cannot be read or parsed.
    #[must_use]
    pub fn read(path: &Path) -> Option<Self> {
        Lock::parse(&std::fs::read_to_string(path).ok()?)
    }

    /// The text of the lock file.
    #[must_use]
    pub fn text(&self) -> String {
        format!("session {}\npid {}\n", self.session, self.pid)
    }

    /// Whether the session of this lock, read from the file `path`, still runs: a process of its pid runs and did not
    /// start more than 2 seconds after the file was last written; else the lock is stale (WSL-1, STU-7).
    #[must_use]
    pub fn runs(&self, path: &Path) -> bool {
        alive(&self.pid) && !lock_taken(&self.pid, path)
    }
}

/// Whether the process `pid` runs (`kill -0`); false for an empty or invalid id.
#[must_use]
pub fn alive(pid: &str) -> bool {
    // A pid_t is an i32: a larger number wraps to a negative one, which `kill` reads as a process group (-1: all).
    pid.parse::<i32>().is_ok_and(|p| p > 0)
        && Command::new("kill")
            .args(["-0", pid])
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
}

/// Whether the process `pid` started more than 2 seconds after the file `lock` was last written: another process that
/// took the number of the session that wrote it, a stale lock (WSL-1, RBI-4, STU-7); false when its mtime cannot be
/// read.
#[must_use]
pub fn lock_taken(pid: &str, lock: &Path) -> bool {
    std::fs::metadata(lock)
        .and_then(|m| m.modified())
        .is_ok_and(|written| started_after(pid, written))
}

/// Whether the process `pid` started more than 2 seconds after `written`: another process that took the number of
/// the session that wrote a lock or a claim then; false when its start cannot be read. The start is read from
/// `ps -o etimes=` (procps) against the wall clock: a `ps` without that field gives false, and a wall clock stepped
/// forward after `written` can give true for the session that wrote it.
#[must_use]
pub fn started_after(pid: &str, written: SystemTime) -> bool {
    let Ok(out) = Command::new("ps")
        .args(["-o", "etimes=", "-p", pid])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
    else {
        return false;
    };
    let Ok(elapsed) = String::from_utf8_lossy(&out.stdout).trim().parse::<u64>() else {
        return false;
    };
    let Some(started) = SystemTime::now().checked_sub(Duration::from_secs(elapsed)) else {
        return false;
    };
    started
        .duration_since(written)
        .is_ok_and(|d| d > PID_REUSE_MARGIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether `ps -o etimes=` reads the start of a process here (procps): without it no process counts as started
    /// after a lock, by design.
    fn ps_reads_starts() -> bool {
        Command::new("ps")
            .args(["-o", "etimes=", "-p", &std::process::id().to_string()])
            .output()
            .is_ok_and(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<u64>()
                    .is_ok()
            })
    }

    #[test]
    fn a_field_is_the_value_after_its_key_and_one_space() {
        let text = "session s1\npidx 9\npid 42\npid 43\n";
        assert_eq!(field(text, "session"), Some("s1"));
        assert_eq!(field(text, "pid"), Some("42"), "the first line of the key");
        assert_eq!(field(text, "since"), None);
        assert_eq!(field("pid\n", "pid"), None, "a key with no value");
    }

    #[test]
    fn a_lock_needs_its_session_and_its_pid_and_round_trips() {
        let lock = Lock::parse("session s1\npid 42\n").unwrap();
        assert_eq!(
            lock,
            Lock {
                session: "s1".into(),
                pid: "42".into()
            }
        );
        assert_eq!(Lock::parse(&lock.text()), Some(lock));
        assert_eq!(Lock::parse("session s1\n"), None);
        assert_eq!(Lock::parse("pid 42\n"), None);
    }

    #[test]
    fn a_pid_that_runs_is_alive_and_a_bad_one_is_not() {
        assert!(alive(&std::process::id().to_string()));
        assert!(!alive(""));
        assert!(!alive("0"));
        assert!(!alive("x"));
        assert!(!alive(&u32::MAX.to_string()));
    }

    // spec: WSL-1 (internal: the rule both binaries share, proven by the binaries' own tests of WSL-1 and STU-7 too)
    #[test]
    fn a_lock_written_before_its_process_started_is_stale() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cardinal-session");
        let lock = Lock {
            session: "s".into(),
            pid: std::process::id().to_string(),
        };
        std::fs::write(&path, lock.text()).unwrap();
        assert!(lock.runs(&path), "written after the process started");
        if !ps_reads_starts() {
            return;
        }
        let file = std::fs::File::options().write(true).open(&path).unwrap();
        file.set_modified(SystemTime::UNIX_EPOCH).unwrap();
        assert!(lock_taken(&lock.pid, &path));
        assert!(
            !lock.runs(&path),
            "the process took the number of the session that wrote it"
        );
        assert!(
            !lock_taken(&lock.pid, &dir.path().join("none")),
            "no mtime: not taken"
        );
        let dead = Lock {
            session: "s".into(),
            pid: String::new(),
        };
        assert!(!dead.runs(&path));
    }

    #[test]
    fn a_process_does_not_start_after_now() {
        assert!(!started_after(
            &std::process::id().to_string(),
            SystemTime::now()
        ));
        assert!(!started_after("", SystemTime::UNIX_EPOCH));
        if ps_reads_starts() {
            let me = std::process::id().to_string();
            assert!(started_after(&me, SystemTime::UNIX_EPOCH));
        }
    }
}
