//! The end of what a merged issue left (SUA-3, SUA-8, SUA-9): once the trunk holds the close of the issue `<stem>`,
//! the session of the linked worktree on the branch `<stem>` ends, the zellij tab the UI opened for it closes (only
//! the session's pane when the tab holds others), then `cardinal run worktree close <stem>` removes the worktree and
//! the branch from the main worktree. Closing a tab sends `SIGHUP` to its process, on which `claude` runs its
//! `SessionEnd` hooks; a session with no pane of the UI gets that signal directly. Nothing is ended while the session
//! is at work (no wait mark names it), nor from a dirty or locked worktree, nor one holding another; one whose
//! directory is gone skips the checks that run in it, its session read from the lock its git directory keeps.

use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::merge::{git, main_worktree};
use crate::source;

/// How long the session may take to end once signalled (SUA-9).
const WAIT: Duration = Duration::from_secs(10);

/// The pane of the session of `stem` in the panes `list-panes --json` printed: the tab id and the pane id, and
/// whether it is the only terminal pane of its tab (SUA-9).
fn session_pane(panes: &Value, stem: &str) -> Option<(u64, u64, bool)> {
    let panes = panes.as_array()?;
    let terminal = |p: &&Value| p["is_plugin"] == false;
    let command = format!("claude /issue-workflow issues/{stem}.md");
    let pane = panes.iter().filter(terminal).find(|p| {
        p["tab_name"].as_str() == Some(stem)
            && p["terminal_command"].as_str() == Some(&command)
            && p["exited"] != true
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

/// Runs `zellij args` in `dir`, failing with the line of SUA-9.
fn zellij(dir: &Path, args: &[&str]) -> Result<String, String> {
    let name = format!("zellij {}", args.join(" "));
    let (out, err, code) = source::run(&name, &source::zellij(), args, dir)?;
    if code != 0 {
        return Err(source::failure(&name, code, &err));
    }
    Ok(out)
}

/// Closes the tab, or the pane alone, of the session of `stem`: what was closed, `None` when the UI opened none or
/// the panes cannot be listed (a zellij without `list-panes`).
fn close_pane(dir: &Path, stem: &str) -> Result<Option<String>, String> {
    let Ok(out) = zellij(dir, &["action", "list-panes", "--json", "--all"]) else {
        return Ok(None);
    };
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

/// Ends the session `id` of process `pid`, its pane closed or not, then waits for it to end (SUA-9).
fn end_session(id: &str, pid: &str, signalled: bool, dir: &Path) -> Result<(), String> {
    if !signalled {
        let kill = "kill -HUP";
        let (_, err, code) = source::run(kill, Path::new("kill"), &["-HUP", pid], dir)?;
        // a session that ended on its own meanwhile needs no signal
        if code != 0 && source::alive(pid) {
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

/// Whether the process `pid` started more than 2 seconds after the lock `lock` was last written: another process
/// that took the number of the session (SUA-9).
fn started_after(pid: &str, lock: &Path) -> bool {
    let Some(written) = std::fs::metadata(lock).and_then(|m| m.modified()).ok() else {
        return false;
    };
    let Ok((out, _, 0)) = source::run(
        "ps",
        Path::new("ps"),
        &["-o", "etimes=", "-p", pid],
        lock.parent().unwrap_or(lock),
    ) else {
        return false;
    };
    let Ok(elapsed) = out.trim().parse::<u64>() else {
        return false;
    };
    let started = std::time::SystemTime::now() - Duration::from_secs(elapsed);
    started
        .duration_since(written)
        .is_ok_and(|d| d > Duration::from_secs(2))
}

/// The checks that run in the directory of the worktree at `path`, which a directory removed by hand skips (SUA-8).
fn refuse_in(root: &Path, path: &Path, worktrees: &[source::Worktree]) -> Result<(), String> {
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
    // ignored by `git status` (`.claude/worktrees/`), refused by `worktree close` after the tab and the session
    let nested = worktrees
        .iter()
        .map(|w| Path::new(&w.path))
        .find(|w| *w != path && w.starts_with(path));
    if let Some(inner) = nested {
        return Err(format!(
            "{} holds the worktree {}: remove that one first",
            path.display(),
            inner.display()
        ));
    }
    Ok(())
}

/// Ends what the merged issue `stem` of the project at `root` left, on the linked worktree of the branch `stem`
/// (SUA-8).
pub fn end(root: &Path, stem: &str) -> Result<String, String> {
    end_on(root, stem, stem)
}

/// Ends what the merged issue `stem` left on the linked worktree of the branch `branch`: `ended <stem>`, or why
/// nothing was ended, or the step that failed after what the steps before it did (SUA-3, SUA-8, SUA-9).
pub fn end_on(root: &Path, stem: &str, branch: &str) -> Result<String, String> {
    let (main, trunk) = main_worktree(root)?;
    let worktrees = source::worktrees(root)?;
    let target = worktrees
        .iter()
        .skip(1)
        .find(|w| w.branch.as_deref() == Some(branch))
        .ok_or_else(|| format!("{stem}: no linked worktree on branch {branch}"))?;
    let path = Path::new(&target.path);
    let trunk = trunk.ok_or("main worktree not on a branch")?;
    if !source::holds_close(&main, &trunk, stem)? {
        return Err(format!(
            "{trunk} does not hold the close of {stem}: merge it first"
        ));
    }
    let ancestor = [
        "merge-base",
        "--is-ancestor",
        &format!("refs/heads/{branch}"),
        &format!("refs/heads/{trunk}"),
    ];
    if source::run("git merge-base", Path::new("git"), &ancestor, &main)?.2 != 0 {
        return Err(format!(
            "{branch} has commits {trunk} lacks: merge them first"
        ));
    }
    // `worktree close` refuses it too, but only once the tab and the session are ended (SUA-8)
    if target.locked {
        return Err(format!(
            "{0} is locked: run `git worktree unlock '{0}'` first",
            path.display()
        ));
    }
    // a directory removed by hand: nothing to check in it, `worktree close` removes that worktree alone (SUA-8)
    // only a confirmed missing directory: one that cannot be reached keeps its checks
    let gone = !path.try_exists().unwrap_or(true);
    if !gone {
        refuse_in(root, path, &worktrees)?;
    }
    let lock = source::worktree_git_path(root, path, "cardinal-session");
    let session = target
        .session
        .as_ref()
        .filter(|s| s.alive && !lock.as_deref().is_some_and(|l| started_after(&s.pid, l)));
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
        &["run", "worktree", "close", &target.path],
        &main,
    )
    .map_err(|e| failed(&done, e))?;
    if code != 0 {
        return Err(failed(&done, source::failure(name, code, &err)));
    }
    Ok(format!("ended {stem}"))
}

/// Ends what each merged issue left, on every linked worktree whose branch has its close on the trunk, going on
/// after one it refuses or fails on: one line per such worktree, `ended <branch>` or `kept <branch>: <why>`, and
/// whether it kept none; why it ends nothing when the main worktree cannot be read or is not on a branch (SUA-13,
/// SUA-14).
pub fn end_merged(root: &Path) -> Result<(Vec<String>, bool), String> {
    let (_, trunk) = main_worktree(root)?;
    trunk.ok_or("main worktree not on a branch")?;
    let mut all_ended = true;
    let lines = source::worktrees(root)?
        .iter()
        .skip(1)
        .filter(|w| w.merged)
        .filter_map(|w| w.branch.as_deref())
        .map(|branch| match end_on(root, branch, branch) {
            Ok(line) => line,
            Err(why) => {
                all_ended = false;
                format!("kept {branch}: {why}")
            }
        })
        .collect();
    Ok((lines, all_ended))
}

/// The status line of `E`: the lines of `end_merged` joined by ` | `, an error when it kept one (SUA-13).
pub fn merged_status(lines: &[String], all_ended: bool) -> Result<String, String> {
    let line = merged_lines(lines).join(" | ");
    if all_ended {
        Ok(line)
    } else {
        Err(line)
    }
}

/// The lines of `end_merged`, `no merged worktree` when it has none (SUA-14).
pub fn merged_lines(lines: &[String]) -> Vec<String> {
    if lines.is_empty() {
        vec!["no merged worktree".to_string()]
    } else {
        lines.to_vec()
    }
}

/// The result of the merge of SUA-2 followed, once its `cardinal check` passed, by the end of its issue (SUA-3).
pub fn after_merge(
    merged: Result<(String, String), String>,
    end: impl FnOnce(&str) -> Result<String, String>,
) -> Result<String, String> {
    let (branch, merged) = merged?;
    end(&branch)
        .map(|e| format!("{merged}; {e}"))
        .map_err(|e| format!("{merged}; {e}"))
}

#[cfg(test)]
mod tests {
    use super::{after_merge, merged_status};

    // spec: SUA-13, SUA-H6 (internal: the status line of the key `E`, drawn in the terminal loop)
    #[test]
    fn the_status_line_of_e_parts_each_worktree_and_fails_when_one_is_kept() {
        let ended = |s: &str| s.to_string();
        assert_eq!(
            merged_status(&[ended("ended foo"), ended("ended bar")], true),
            Ok("ended foo | ended bar".to_string())
        );
        assert_eq!(
            merged_status(
                &[
                    ended("kept foo: closed tab foo; worktree close: exit 1: no"),
                    ended("ended bar")
                ],
                false
            ),
            Err("kept foo: closed tab foo; worktree close: exit 1: no | ended bar".to_string())
        );
        assert_eq!(
            merged_status(&[], true),
            Ok("no merged worktree".to_string())
        );
    }

    // spec: SUA-3 (internal: the key `f` runs in the terminal loop, which no binary test drives)
    #[test]
    fn a_merge_whose_check_passed_chains_the_end_on_its_branch_and_a_failed_one_does_not() {
        let merged = || {
            Ok((
                "feature-foo".to_string(),
                "merged foo (feature-foo) into trunk".to_string(),
            ))
        };
        assert_eq!(
            after_merge(merged(), |branch| Ok(format!("ended on {branch}"))),
            Ok("merged foo (feature-foo) into trunk; ended on feature-foo".to_string())
        );
        assert_eq!(
            after_merge(merged(), |_| Err("session s1 is at work in ws".to_string())),
            Err("merged foo (feature-foo) into trunk; session s1 is at work in ws".to_string())
        );
        let failed = "merged foo (foo) into trunk; cardinal check: exit 1".to_string();
        assert_eq!(
            after_merge(Err(failed.clone()), |_| panic!(
                "no end after a failed check"
            )),
            Err(failed)
        );
    }
}
