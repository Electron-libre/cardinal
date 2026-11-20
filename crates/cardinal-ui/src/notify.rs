//! The desktop notification of a new wait (STU-29, STU-30): between two reads of the screen, a worktree that waits
//! for the user with a mark (its session and its time) the read before did not show sends one notification, through
//! `CARDINAL_UI_NOTIFY` or `notify-send`, run on a thread of its own; the first failure is kept for the status line.

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use crate::source::{self, Worktree};

/// A worktree waiting for the user (STU-13), as one read saw it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Wait {
    pub path: String,
    pub branch: Option<String>,
    pub session: String,
    /// Since when it waits (seconds since the Unix epoch): with the session, what tells a new wait from the same one.
    pub since: i64,
    pub reason: String,
}

impl Wait {
    /// Whether `other` is the same wait: the same worktree and the same mark, whatever its reason.
    fn same(&self, other: &Self) -> bool {
        self.path == other.path && self.session == other.session && self.since == other.since
    }

    /// The title and the body of its notification (STU-29).
    pub fn notification(&self) -> (String, String) {
        let name = self.branch.as_deref().unwrap_or(&self.path);
        (format!("{name} waits"), self.reason.clone())
    }
}

/// The waits of the worktrees of a read (STU-13).
pub fn waits(worktrees: &[Worktree]) -> Vec<Wait> {
    worktrees
        .iter()
        .filter_map(|w| {
            let s = w.session.as_ref()?;
            let (since, reason) = s.waiting.clone()?;
            Some(Wait {
                path: w.path.clone(),
                branch: w.branch.clone(),
                session: s.id.clone(),
                since,
                reason,
            })
        })
        .collect()
}

/// The program that notifies: `CARDINAL_UI_NOTIFY`, else `notify-send` on the `PATH` (STU-30).
pub fn program() -> PathBuf {
    std::env::var_os("CARDINAL_UI_NOTIFY")
        .map_or_else(|| PathBuf::from("notify-send"), PathBuf::from)
}

/// Runs `program title body`; the line of STU-6 prefixed `notify` when it cannot run or exits other than 0.
fn send(program: &Path, title: &str, body: &str) -> Result<(), String> {
    match source::run("notify", program, &[title, body], Path::new("."))? {
        (_, _, 0) => Ok(()),
        (_, err, code) => Err(source::failure("notify", code, &err)),
    }
}

/// Sends the notifications of the new waits between the reads it is given (STU-29, STU-30).
pub struct Notifier {
    program: PathBuf,
    /// The waits of the last read the worktrees of which could be read; `None` before the first.
    last: Option<Vec<Wait>>,
    failures: mpsc::Sender<String>,
    failed: mpsc::Receiver<String>,
    /// Whether a failure was already given to the status line.
    shown: bool,
}

impl Notifier {
    pub fn new(program: PathBuf) -> Self {
        let (failures, failed) = mpsc::channel();
        Self {
            program,
            last: None,
            failures,
            failed,
            shown: false,
        }
    }

    /// Takes the waits of a read, `None` when its worktrees could not be read (then the last ones are kept), and
    /// notifies, each on a thread, those the last read did not show; none on the first read.
    pub fn read(&mut self, waits: Option<&[Wait]>) {
        let Some(now) = waits else { return };
        if let Some(last) = &self.last {
            for wait in now.iter().filter(|w| !last.iter().any(|l| l.same(w))) {
                let (title, body) = wait.notification();
                let program = self.program.clone();
                let failures = self.failures.clone();
                std::thread::spawn(move || {
                    if let Err(e) = send(&program, &title, &body) {
                        let _ = failures.send(e);
                    }
                });
            }
        }
        self.last = Some(now.to_vec());
    }

    /// The line of the first failure of a notification, once (STU-30).
    pub fn failure(&mut self) -> Option<String> {
        let line = self.failed.try_recv().ok()?;
        if self.shown {
            return None;
        }
        self.shown = true;
        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};

    fn wait(path: &str, session: &str, since: i64, reason: &str) -> Wait {
        Wait {
            path: path.to_string(),
            branch: Some(path.trim_start_matches('/').to_string()),
            session: session.to_string(),
            since,
            reason: reason.to_string(),
        }
    }

    /// A program that appends its two arguments as one line to `log`, then exits `code` with `err` on stderr.
    fn fake(dir: &Path, code: i32, err: &str) -> (PathBuf, PathBuf) {
        let log = dir.join("log");
        let program = dir.join("notify");
        let script = format!(
            "#!/bin/sh\nprintf '%s|%s\\n' \"$1\" \"$2\" >> '{}'\necho '{err}' >&2\nexit {code}\n",
            log.display()
        );
        std::fs::write(&program, script).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        (program, log)
    }

    /// The lines of `log` once it holds `n` of them, or after 5 s.
    fn lines(log: &Path, n: usize) -> Vec<String> {
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            let text = std::fs::read_to_string(log).unwrap_or_default();
            let lines: Vec<String> = text.lines().map(str::to_string).collect();
            if lines.len() >= n || Instant::now() > end {
                return lines;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    // spec: STU-H5 (internal: the screen of the flow needs a real terminal)
    // spec: STU-29 (internal: the reads of a screen that needs a real terminal)
    #[test]
    fn a_new_wait_is_notified_once_and_not_those_of_the_first_read() {
        let dir = tempfile::tempdir().unwrap();
        let (program, log) = fake(dir.path(), 0, "");
        let mut n = Notifier::new(program);
        n.read(Some(&[wait("/old", "a", 1, "stop")]));
        n.read(Some(&[
            wait("/old", "a", 1, "stop"),
            wait("/foo", "b", 2, "stop"),
        ]));
        assert_eq!(lines(&log, 1), ["foo waits|stop"]);
        // the reason changed by the idle notification, the worktrees unreadable, a mark of another session: nothing
        n.read(Some(&[
            wait("/old", "a", 1, "idle"),
            wait("/foo", "b", 2, "idle"),
        ]));
        n.read(None);
        n.read(Some(&[
            wait("/old", "a", 1, "idle"),
            wait("/foo", "b", 2, "idle"),
        ]));
        // the session worked, then waits anew
        n.read(Some(&[wait("/old", "a", 1, "idle")]));
        n.read(Some(&[
            wait("/old", "a", 1, "idle"),
            wait("/foo", "b", 9, "Claude needs your permission"),
        ]));
        assert_eq!(
            lines(&log, 2),
            ["foo waits|stop", "foo waits|Claude needs your permission"]
        );
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(
            std::fs::read_to_string(&log).unwrap().lines().count(),
            2,
            "no other notification"
        );
        let mut detached = wait("/ws", "c", 3, "stop");
        detached.branch = None;
        assert_eq!(detached.notification().0, "/ws waits");
    }

    // spec: STU-30 (internal: the status line of a screen that needs a real terminal)
    #[test]
    fn a_failing_program_is_shown_once_on_the_status_line() {
        let dir = tempfile::tempdir().unwrap();
        let (program, log) = fake(dir.path(), 3, "no bus");
        let mut n = Notifier::new(program);
        n.read(Some(&[]));
        n.read(Some(&[wait("/foo", "a", 1, "stop")]));
        n.read(Some(&[
            wait("/foo", "a", 1, "stop"),
            wait("/bar", "b", 2, "stop"),
        ]));
        assert_eq!(lines(&log, 2).len(), 2);
        let end = Instant::now() + Duration::from_secs(5);
        let first = loop {
            if let Some(line) = n.failure() {
                break line;
            }
            assert!(Instant::now() < end, "no failure shown");
            std::thread::sleep(Duration::from_millis(20));
        };
        assert_eq!(first, "notify: exit 3: no bus");
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(n.failure(), None, "the first time only");
        let mut missing = Notifier::new(dir.path().join("none"));
        missing.read(Some(&[]));
        missing.read(Some(&[wait("/foo", "a", 1, "stop")]));
        let end = Instant::now() + Duration::from_secs(5);
        let line = loop {
            if let Some(line) = missing.failure() {
                break line;
            }
            assert!(Instant::now() < end, "no failure shown");
            std::thread::sleep(Duration::from_millis(20));
        };
        assert!(line.starts_with("notify: cannot run "), "{line}");
    }

    #[test]
    fn the_program_defaults_to_notify_send() {
        if std::env::var_os("CARDINAL_UI_NOTIFY").is_none() {
            assert_eq!(program(), PathBuf::from("notify-send"));
        }
    }
}
