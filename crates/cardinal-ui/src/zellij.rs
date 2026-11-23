//! The calls of `cardinal-ui` to `zellij` (SUZ): each addressed to the session the UI runs in by the name zellij
//! records for it now, so a session renamed since the pane opened still answers (SUZ-1, SUZ-3), and each killed, with
//! the processes it started, once it has not answered within 5 seconds (SUZ-4), so no zellij that hangs blocks the
//! UI (SUZ-2).

use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// How long a `zellij` call may run (SUZ-4).
const LIMIT: Duration = Duration::from_secs(5);

/// The output of `zellij args` in `dir`, as `source::run` gives it: stdout, stderr and exit code; the error line of
/// STU-6 when it cannot run, that of SUZ-4 when it does not answer within [`LIMIT`].
pub fn run(name: &str, args: &[&str], dir: &Path) -> Result<(String, String, i32), String> {
    let program = crate::source::zellij();
    let mut command = Command::new(&program);
    if let Some(name) = current_name() {
        command.env("ZELLIJ_SESSION_NAME", name);
    }
    let mut child = command
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // a group of its own, so the kill reaches the processes it started
        .process_group(0)
        .spawn()
        .map_err(|e| format!("{name}: cannot run {}: {e}", program.display()))?;
    let out = drain(child.stdout.take());
    let err = drain(child.stderr.take());
    let deadline = Instant::now() + LIMIT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            Ok(None) => break None,
            Err(e) => {
                let _ = child.kill();
                return Err(format!(
                    "{name}: cannot wait for {}: {e}",
                    program.display()
                ));
            }
        }
    };
    let Some(status) = status else {
        let group = format!("-{}", child.id());
        let _ = Command::new("kill")
            .args(["-s", "KILL", "--", &group])
            .stderr(Stdio::null())
            .status();
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!(
            "{name}: no answer after {} s (zellij session renamed?)",
            LIMIT.as_secs()
        ));
    };
    Ok((
        out.join().unwrap_or_default(),
        err.join().unwrap_or_default(),
        status.code().unwrap_or(-1),
    ))
}

/// Reads `pipe` to its end in a thread of its own, so a child that fills it never blocks.
fn drain(pipe: Option<impl Read + Send + 'static>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

/// The name zellij records now for the session of the server the UI descends from, read at each call so a rename
/// while the UI runs is followed; `None` when it cannot be found: the call keeps the environment's (SUZ-3).
fn current_name() -> Option<String> {
    static SOCKET: OnceLock<Option<PathBuf>> = OnceLock::new();
    let socket = SOCKET.get_or_init(|| server_socket(std::os::unix::process::parent_id()));
    recorded_name(socket.as_deref()?, &cache_dir()?)
}

/// The directory of the caches: `$XDG_CACHE_HOME`, else `~/.cache`.
fn cache_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
}

/// The socket of the nearest process among `pid` and its ancestors run with `--server <socket>`, read from `/proc`.
fn server_socket(mut pid: u32) -> Option<PathBuf> {
    while pid > 1 {
        let cmdline = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
        let mut args = cmdline.split(|b| *b == 0);
        if args.any(|a| a == b"--server") {
            if let Some(socket) = args.next().filter(|s| !s.is_empty()) {
                return Some(PathBuf::from(String::from_utf8_lossy(socket).into_owned()));
            }
        }
        pid = parent(pid)?;
    }
    None
}

/// The parent of `pid`: the field after the state in `/proc/<pid>/stat`, read after the `)` that ends the command.
fn parent(pid: u32) -> Option<u32> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    stat.rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

/// The `name` of `<cache>/zellij/<socket directory>/session_info/<socket id>/session-metadata.kdl`.
fn recorded_name(socket: &Path, cache: &Path) -> Option<String> {
    let id = socket.file_name()?;
    let contract = socket.parent()?.file_name()?;
    let file = cache
        .join("zellij")
        .join(contract)
        .join("session_info")
        .join(id)
        .join("session-metadata.kdl");
    let text = std::fs::read_to_string(file).ok()?;
    // the top-level node only: the tabs and panes nested under it have their own `name`
    text.lines()
        .find_map(|l| kdl_string(l.strip_prefix("name ")?))
}

/// The value of the KDL string `"<text>"`, its `\\` and `\"` escapes read; `None` with any other escape, so the call
/// keeps the environment's name rather than address a name misread.
fn kdl_string(quoted: &str) -> Option<String> {
    let inner = quoted.trim().strip_prefix('"')?.strip_suffix('"')?;
    let mut value = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        value.push(match c {
            '\\' => chars.next().filter(|e| matches!(e, '\\' | '"'))?,
            c => c,
        });
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: SUZ-3 (internal: a rename while the binary runs would need a UI that keeps running under the test)
    #[test]
    fn the_name_is_read_again_at_each_call() {
        let cache = tempfile::tempdir().unwrap();
        let socket = Path::new("/run/zellij/contract_version_1/abc123");
        let info = cache
            .path()
            .join("zellij/contract_version_1/session_info/abc123");
        std::fs::create_dir_all(&info).unwrap();
        let file = info.join("session-metadata.kdl");
        assert_eq!(recorded_name(socket, cache.path()), None);
        std::fs::write(
            &file,
            "name \"b\"\ntabs {\n    tab {\n        name \"t\"\n    }\n}\n",
        )
        .unwrap();
        assert_eq!(recorded_name(socket, cache.path()).as_deref(), Some("b"));
        std::fs::write(&file, "name \"c \\\"q\\\"\"\n").unwrap();
        assert_eq!(
            recorded_name(socket, cache.path()).as_deref(),
            Some("c \"q\"")
        );
        std::fs::write(&file, "name \"d\\ne\"\n").unwrap();
        assert_eq!(recorded_name(socket, cache.path()), None);
    }
}
