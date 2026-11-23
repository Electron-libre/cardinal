//! A skill session in a zellij tab of the main worktree (SUA-4, SUA-12, SUA-16, SUA-18, SUA-23, SUA-25): an interactive `claude`
//! whose first message names the skill, in a tab of its own, never opened twice while its pane still runs.

use std::path::Path;

use serde_json::Value;

use crate::source;

/// Whether a tab `tab` holds a terminal pane still running `claude <message>`; `false` when the panes cannot be
/// listed (SUA-11, SUA-17).
pub fn running(dir: &Path, tab: &str, message: &str) -> bool {
    let args = ["action", "list-panes", "--json", "--all"];
    let Ok((out, _, 0)) = source::run("zellij", &source::zellij(), &args, dir) else {
        return false;
    };
    let command = format!("claude {message}");
    let panes = serde_json::from_str::<Value>(out.trim()).unwrap_or_default();
    source::array(&panes).iter().any(|p| {
        p["is_plugin"] == false
            && p["tab_name"] == tab
            && p["terminal_command"] == command.as_str()
            && p["exited"] != true
    })
}

/// Opens in `main` a zellij tab `tab` running `claude <message>`: the path it runs in, or the failure of SUA-11.
pub fn open(main: &Path, tab: &str, message: &str) -> Result<String, String> {
    let cwd = main.display().to_string();
    let name = "zellij action new-tab";
    let args = [
        "action", "new-tab", "--cwd", &cwd, "--name", tab, "--", "claude", message,
    ];
    let (_, err, code) = source::run(name, &source::zellij(), &args, main)?;
    if code != 0 {
        return Err(source::failure(name, code, &err));
    }
    Ok(cwd)
}
