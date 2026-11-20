//! The end of what a merged issue left (STU-22, STU-24, STU-25): once the trunk holds the close of the issue `<stem>`,
//! the session of the linked worktree on the branch `<stem>` ends, the zellij tab the UI opened for it closes (only
//! the session's pane when the tab holds others), then `cardinal run worktree close <stem>` removes the worktree and
//! the branch from the main worktree. Closing a tab sends `SIGHUP` to its process, on which `claude` runs its
//! `SessionEnd` hooks; a session with no pane of the UI gets that signal directly. Nothing is ended while the session
//! is at work (no wait mark names it), nor from a dirty worktree.

use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::merge::{git, main_worktree};
use crate::source;

/// How long the session may take to end once signalled (STU-25).
const WAIT: Duration = Duration::from_secs(10);

/// The pane of the session of `stem` in the panes `list-panes --json` printed: the tab id and the pane id, and
/// whether it is the only terminal pane of its tab (STU-25).
fn session_pane(panes: &Value, stem: &str) -> Option<(u64, u64, bool)> {
    let panes = panes.as_array()?;
    let terminal = |p: &&Value| p["is_plugin"] == false;
    let command = format!("claude /issue-workflow issues/{stem}.md");
    let pane = panes.iter().filter(terminal).find(|p| {
        p["tab_name"].as_str() == Some(stem) && p["terminal_command"].as_str() == Some(&command)
    })?;
    let tab = pane["tab_id"].as_u64()?;
    let alone = panes
        .iter()
        .filter(terminal)
        .filter(|p| p["tab_id"].as_u64() == Some(tab))
        .count()
        == 1;
    Some((tab, pane["id"].as_u64()?, alone))
}

/// Runs `zellij args` in `dir`, failing with the line of STU-25.
fn zellij(dir: &Path, args: &[&str]) -> Result<String, String> {
    let name = format!("zellij {}", args.join(" "));
    let (out, err, code) = source::run(&name, &source::zellij(), args, dir)?;
    if code != 0 {
        return Err(source::failure(&name, code, &err));
    }
    Ok(out)
}

/// Closes the tab, or the pane alone, of the session of `stem`: what was closed, `None` when the UI opened none.
fn close_pane(dir: &Path, stem: &str) -> Result<Option<String>, String> {
    let out = zellij(dir, &["action", "list-panes", "--json", "--all"])?;
    let panes = serde_json::from_str::<Value>(out.trim()).unwrap_or_default();
    let Some((tab, pane, alone)) = session_pane(&panes, stem) else {
        return Ok(None);
    };
    if alone {
        zellij(dir, &["action", "close-tab-by-id", &tab.to_string()])?;
        Ok(Some(format!("closed tab {stem}")))
    } else {
        let pane = format!("terminal_{pane}");
        zellij(dir, &["action", "close-pane", "--pane-id", &pane])?;
        Ok(Some(format!("closed pane {pane}")))
    }
}

/// Ends the session `id` of process `pid`, its pane closed or not, then waits for it to end (STU-25).
fn end_session(id: &str, pid: &str, signalled: bool, dir: &Path) -> Result<(), String> {
    if !signalled {
        let kill = "kill -HUP";
        let (_, err, code) = source::run(kill, Path::new("kill"), &["-HUP", pid], dir)?;
        if code != 0 {
            return Err(source::failure(kill, code, &err));
        }
    }
    let start = Instant::now();
    while source::alive(pid) {
        if start.elapsed() >= WAIT {
            return Err(format!(
                "session {id} still runs after {} s",
                WAIT.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

/// Ends what the merged issue `stem` of the project at `root` left: `ended <stem>`, or why nothing was ended, or the
/// step that failed after what the steps before it did (STU-24, STU-25).
pub fn end(root: &Path, stem: &str) -> Result<String, String> {
    let (main, trunk) = main_worktree(root)?;
    let worktrees = source::worktrees(root)?;
    let target = worktrees
        .iter()
        .skip(1)
        .find(|w| w.branch.as_deref() == Some(stem))
        .ok_or_else(|| format!("{stem}: no linked worktree on branch {stem}"))?;
    let path = Path::new(&target.path);
    let trunk = trunk.ok_or("main worktree not on a branch")?;
    if !source::holds_close(&main, &trunk, stem)? {
        return Err(format!(
            "{trunk} does not hold the close of {stem}: merge it first"
        ));
    }
    if !git(path, &["status", "--porcelain"])?.trim().is_empty() {
        return Err(format!(
            "{} has uncommitted changes or untracked files",
            path.display()
        ));
    }
    let inside = std::fs::canonicalize(root)
        .ok()
        .zip(std::fs::canonicalize(path).ok())
        .is_some_and(|(here, target)| here.starts_with(target));
    if inside {
        return Err(format!(
            "cardinal-ui runs in {}: run it from another worktree",
            path.display()
        ));
    }
    let session = target.session.as_ref().filter(|s| s.alive);
    if let Some(s) = session.filter(|s| s.waiting.is_none()) {
        return Err(format!("session {} is at work in {}", s.id, path.display()));
    }
    let mut done: Vec<String> = Vec::new();
    let failed = |done: &[String], e: String| {
        if done.is_empty() {
            e
        } else {
            format!("{}; {e}", done.join("; "))
        }
    };
    if source::in_zellij() {
        if let Some(closed) = close_pane(root, stem)? {
            done.push(closed);
        }
    }
    if let Some(s) = session {
        end_session(&s.id, &s.pid, !done.is_empty(), root).map_err(|e| failed(&done, e))?;
        done.push(format!("ended session {}", s.id));
    }
    let name = "worktree close";
    let (_, err, code) = source::run(
        name,
        &source::cardinal(),
        &["run", "worktree", "close", stem],
        &main,
    )
    .map_err(|e| failed(&done, e))?;
    if code != 0 {
        return Err(failed(&done, source::failure(name, code, &err)));
    }
    Ok(format!("ended {stem}"))
}

/// The result of the merge of STU-20 followed, once its `cardinal check` passed, by the end of its issue (STU-22).
pub fn after_merge(
    merged: Result<String, String>,
    end: impl FnOnce() -> Result<String, String>,
) -> Result<String, String> {
    let merged = merged?;
    end()
        .map(|e| format!("{merged}; {e}"))
        .map_err(|e| format!("{merged}; {e}"))
}

#[cfg(test)]
mod tests {
    use super::after_merge;

    // spec: STU-22 (internal: the key `f` runs in the terminal loop, which no binary test drives)
    #[test]
    fn a_merge_whose_check_passed_chains_the_end_and_a_failed_one_does_not() {
        let merged = || Ok("merged foo (foo) into trunk".to_string());
        assert_eq!(
            after_merge(merged(), || Ok("ended foo".to_string())),
            Ok("merged foo (foo) into trunk; ended foo".to_string())
        );
        assert_eq!(
            after_merge(merged(), || Err("session s1 is at work in ws".to_string())),
            Err("merged foo (foo) into trunk; session s1 is at work in ws".to_string())
        );
        let failed = Err("merged foo (foo) into trunk; cardinal check: exit 1".to_string());
        assert_eq!(
            after_merge(failed.clone(), || panic!("no end after a failed check")),
            failed
        );
    }
}
