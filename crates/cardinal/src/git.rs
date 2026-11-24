//! The one git runner of the crate: every git command of `cardinal` is built by `command` and run by `exec`, which
//! fails with one text, the command, its exit code and the first line of its stderr (RBI-4).

use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A git command that did not succeed.
#[derive(Debug)]
pub(crate) struct Failure {
    /// Its exit code, `-1` when a signal ended it, `None` when it could not start.
    pub(crate) code: Option<i32>,
    /// Its whole stderr, for a caller that recognizes a message of git.
    pub(crate) stderr: String,
    text: String,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl From<Failure> for String {
    fn from(failure: Failure) -> Self {
        failure.text
    }
}

/// `git` run in `dir` with no input: the base of a command whose caller sets an environment or runs it with a deadline.
/// Its first argument is the caller's: a wrapper of git on the `PATH` reads the subcommand where git has it.
pub(crate) fn command(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir).stdin(Stdio::null());
    cmd
}

/// Runs `cmd`, built by `command`, with `input` written to it from a thread (so a large output never blocks the
/// write): its stdout, or its failure.
pub(crate) fn exec(mut cmd: Command, input: Option<&[u8]>) -> Result<Vec<u8>, Failure> {
    let name = name(&cmd);
    let cmd_dir = cmd.get_current_dir().map(Path::to_path_buf);
    let cannot_run = |e: std::io::Error| {
        let place = cmd_dir
            .as_deref()
            .map_or_default(|d| format!(" in {}", d.display()));
        Failure {
            code: None,
            stderr: String::new(),
            text: format!("cannot run git{place}: {e}"),
        }
    };
    if input.is_some() {
        cmd.stdin(Stdio::piped());
    }
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(cannot_run)?;
    let pipe = child.stdin.take();
    let out = std::thread::scope(|scope| {
        if let (Some(input), Some(mut pipe)) = (input, pipe) {
            // a command that stops reading early closes the pipe: its exit status tells what happened
            scope.spawn(move || pipe.write_all(input).ok());
        }
        child.wait_with_output()
    })
    .map_err(cannot_run)?;
    if out.status.success() {
        return Ok(out.stdout);
    }
    let code = out.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    Err(Failure {
        code: Some(code),
        text: failure(&name, code, &stderr),
        stderr,
    })
}

/// The failure line of RBI-4: `name` (`git <args>`), its exit code and the first line of `stderr`.
fn failure(name: &str, code: i32, stderr: &str) -> String {
    match stderr.lines().find(|l| !l.trim().is_empty()) {
        Some(first) => format!("{name}: exit {code}: {}", first.trim()),
        None => format!("{name}: exit {code}"),
    }
}

/// `git <args>` of `cmd`, without the leading `-c <setting>` that only say how it runs (a `-c` of the subcommand,
/// `git grep -c`, stays).
fn name(cmd: &Command) -> String {
    let mut words = vec!["git".to_string()];
    let mut args = cmd.get_args();
    while let Some(arg) = args.next() {
        if arg == "-c" {
            args.next();
        } else {
            words.push(arg.to_string_lossy().into_owned());
            break;
        }
    }
    words.extend(args.map(|arg| arg.to_string_lossy().into_owned()));
    words.join(" ")
}

/// The stdout of `git <args>` run in `dir`, as it is.
pub(crate) fn bytes(dir: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let mut cmd = command(dir);
    cmd.args(args);
    exec(cmd, None).map_err(String::from)
}

/// The stdout of `git <args>` run in `dir`, its trailing whitespace trimmed.
pub(crate) fn run(dir: &Path, args: &[&str]) -> Result<String, String> {
    bytes(dir, args).map(|out| String::from_utf8_lossy(&out).trim_end().to_string())
}

/// The stdout of `git <args>` run in `dir` with `input` as its stdin.
pub(crate) fn run_stdin(dir: &Path, args: &[&str], input: &[u8]) -> Result<Vec<u8>, String> {
    let mut cmd = command(dir);
    cmd.args(args);
    exec(cmd, Some(input)).map_err(String::from)
}

/// Whether `git <args>` succeeds in `dir`.
pub(crate) fn run_ok(dir: &Path, args: &[&str]) -> bool {
    bytes(dir, args).is_ok()
}

/// The root of the work tree holding `dir`, `None` outside one.
pub(crate) fn toplevel(dir: &Path) -> Option<PathBuf> {
    run(dir, &["rev-parse", "--show-toplevel"])
        .ok()
        .filter(|top| !top.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        run(dir.path(), &["init", "-q"]).unwrap();
        dir
    }

    #[test]
    fn a_failure_names_the_command_its_exit_code_and_the_first_stderr_line() {
        let dir = repo();
        let err = run(dir.path(), &["rev-parse", "--verify", "nope^{commit}"]).unwrap_err();
        assert!(
            err.starts_with("git rev-parse --verify nope^{commit}: exit 128: fatal: "),
            "{err}"
        );
        assert!(!err.contains('\n'), "{err}");
    }

    #[test]
    fn the_settings_of_a_command_stay_out_of_its_name() {
        let dir = repo();
        let mut cmd = command(dir.path());
        cmd.args([
            "-c",
            "core.hooksPath=/dev/null",
            "merge-base",
            "--is-ancestor",
            "x",
            "y",
        ]);
        let failure = exec(cmd, None).unwrap_err();
        assert_eq!(failure.code, Some(128));
        assert!(
            failure
                .to_string()
                .starts_with("git merge-base --is-ancestor x y: exit 128"),
            "{failure}"
        );
    }

    #[test]
    fn a_missing_directory_is_named_and_has_no_exit_code() {
        let gone = tempfile::tempdir().unwrap().path().join("gone");
        let failure = exec(command(&gone), None).unwrap_err();
        assert_eq!(failure.code, None);
        let text = failure.to_string();
        assert!(
            text.starts_with(&format!("cannot run git in {}: ", gone.display())),
            "{text}"
        );
    }

    #[test]
    fn a_setting_of_the_subcommand_stays_in_its_name() {
        let mut cmd = command(Path::new("."));
        cmd.args(["-c", "a=b", "grep", "-c", "x"]);
        assert_eq!(name(&cmd), "git grep -c x");
    }

    #[test]
    fn a_failure_with_a_silent_stderr_ends_on_its_exit_code() {
        assert_eq!(failure("git x", 1, "\n  \n"), "git x: exit 1");
    }

    #[test]
    fn the_input_reaches_git_whatever_its_size() {
        let dir = repo();
        // larger than a pipe buffer both ways: written from a thread, it never blocks the read
        let input = "line\n".repeat(100_000);
        let mut cmd = command(dir.path());
        cmd.arg("stripspace");
        let lines = exec(cmd, Some(input.as_bytes())).unwrap();
        assert_eq!(lines, input.as_bytes());
    }

    #[test]
    fn the_output_is_trimmed_at_its_end_only() {
        let dir = repo();
        std::fs::write(dir.path().join("a"), "").unwrap();
        assert_eq!(run(dir.path(), &["status", "--porcelain"]).unwrap(), "?? a");
        std::fs::write(dir.path().join("a"), "x").unwrap();
        run(dir.path(), &["add", "a"]).unwrap();
        run(
            dir.path(),
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "-qm",
                "a",
            ],
        )
        .unwrap();
        std::fs::write(dir.path().join("a"), "y").unwrap();
        assert_eq!(run(dir.path(), &["status", "--porcelain"]).unwrap(), " M a");
    }

    #[test]
    fn the_top_level_is_none_outside_a_work_tree() {
        let dir = repo();
        let top = toplevel(dir.path()).unwrap();
        assert_eq!(
            top.canonicalize().unwrap(),
            dir.path().canonicalize().unwrap()
        );
        assert!(run_ok(dir.path(), &["rev-parse", "--git-dir"]));
        let outside = tempfile::tempdir().unwrap();
        // a temporary directory under a repository would be inside its work tree
        if run_ok(outside.path(), &["rev-parse", "--git-dir"]) {
            return;
        }
        assert_eq!(toplevel(outside.path()), None);
    }
}
