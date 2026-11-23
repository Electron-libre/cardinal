//! The calls of `cardinal-ui` to `zellij` (SUZ): each killed, with the processes it started, once it has not
//! answered within 5 seconds (SUZ-4), so no zellij that hangs blocks the UI (SUZ-2).

use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long a `zellij` call may run (SUZ-4).
const LIMIT: Duration = Duration::from_secs(5);

/// The output of `zellij args` in `dir`, as `source::run` gives it: stdout, stderr and exit code; the error line of
/// STU-6 when it cannot run, that of SUZ-4 when it does not answer within [`LIMIT`].
pub fn run(name: &str, args: &[&str], dir: &Path) -> Result<(String, String, i32), String> {
    let program = crate::source::zellij();
    let mut child = Command::new(&program)
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
