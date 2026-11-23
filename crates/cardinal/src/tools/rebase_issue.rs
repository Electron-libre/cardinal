//! `rebase-issue`: brings the branch of an issue closed on its branch (its claim `merge pending`, CLM-7) back onto the
//! trunk when another issue reached the trunk first, so the fast-forward of its merge can run. Git runs in the worktree
//! that has the branch checked out, never another one; a dirty worktree, one held by another running session (ADR 0022,
//! a `pid` another process took counting as ended) or a pushed branch is refused before anything is rewritten. A
//! conflict whose every hunk only adds lines on both sides is settled (the trunk's lines, then the branch's); any other
//! aborts the rebase. The replayed commits run no hook: they passed theirs when they were made.
//! Spec: `specs/rebase-issue.md`.
//! Usage: `cardinal run rebase-issue <stem>`, from any worktree.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::issue_claim;

const USAGE: &str = "usage: cardinal run rebase-issue <stem>";

/// Runs `git args` in `dir` with no hook nor editor, the conflicts in `zdiff3` style (the lines both sides added
/// alike stay out of the hunk): its exit code, stdout and stderr.
fn run(dir: &Path, args: &[&str]) -> Result<(i32, String, String), String> {
    let out = Command::new("git")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "merge.conflictStyle=zdiff3",
        ])
        .args(args)
        .current_dir(dir)
        .env("GIT_EDITOR", "true")
        .env("GIT_SEQUENCE_EDITOR", "true")
        // a conflicted file named `[d].md` is that file, never a glob
        .env("GIT_LITERAL_PATHSPECS", "1")
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    Ok((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    ))
}

/// The failure line of RBI-4: the command, its exit code and the first line of its stderr.
fn failure(args: &[&str], code: i32, err: &str) -> String {
    let name = format!("git {}", args.join(" "));
    match err.lines().find(|l| !l.trim().is_empty()) {
        Some(first) => format!("{name}: exit {code}: {}", first.trim()),
        None => format!("{name}: exit {code}"),
    }
}

/// The stdout of `git args` in `dir`, or its failure line.
fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    match run(dir, args)? {
        (0, out, _) => Ok(out.trim_end().to_string()),
        (code, _, err) => Err(failure(args, code, &err)),
    }
}

/// The trunk and the worktree that has `branch` checked out (RBI-1, RBI-4).
fn locate(dir: &Path, branch: &str) -> Result<(String, PathBuf), String> {
    let list = git(dir, &["worktree", "list", "--porcelain"])?;
    let mut records = list.split("\n\n").filter(|r| !r.trim().is_empty());
    let main = records.next().unwrap_or_default();
    if main.lines().any(|l| l == "bare") {
        return Err("no main worktree: the repository is bare".to_string());
    }
    let branch_of = |record: &str| {
        record
            .lines()
            .find_map(|l| l.strip_prefix("branch refs/heads/"))
            .map(str::to_string)
    };
    let trunk = branch_of(main).ok_or("main worktree not on a branch")?;
    if trunk == branch {
        return Err(format!("{branch} is the trunk"));
    }
    let worktree = list
        .split("\n\n")
        .find(|r| branch_of(r).as_deref() == Some(branch))
        .and_then(|r| r.lines().find_map(|l| l.strip_prefix("worktree ")))
        .ok_or_else(|| format!("{branch} is checked out in no worktree"))?;
    Ok((trunk, PathBuf::from(worktree)))
}

/// Why the worktree `wt` of `branch` must not be rewritten, if it must not (RBI-4).
fn refusal(wt: &Path, branch: &str, trunk: &str) -> Result<Option<String>, String> {
    if !git(wt, &["status", "--porcelain"])?.is_empty() {
        return Ok(Some(format!(
            "worktree {} has uncommitted changes or untracked files",
            wt.display()
        )));
    }
    let gitdir = git(wt, &["rev-parse", "--absolute-git-dir"])?;
    let path = Path::new(&gitdir).join("cardinal-session");
    if let Ok(lock) = std::fs::read_to_string(&path) {
        let field = |key: &str| {
            lock.lines()
                .find_map(|l| l.strip_prefix(key)?.strip_prefix(' '))
                .map(str::trim)
        };
        if let (Some(session), Some(pid)) = (field("session"), field("pid")) {
            let caller = std::env::var("CLAUDE_CODE_SESSION_ID").unwrap_or_default();
            let runs = issue_claim::alive(pid) && !issue_claim::started_after(pid, &path);
            if session != caller && runs {
                return Ok(Some(format!("session {session} works in {}", wt.display())));
            }
        }
    }
    let head = format!("refs/heads/{branch}");
    let base = format!("^refs/heads/{trunk}");
    let own = git(wt, &["rev-list", "--count", &head, &base])?;
    let unpushed = git(
        wt,
        &["rev-list", "--count", &head, &base, "--not", "--remotes"],
    )?;
    if own != unpushed {
        return Ok(Some(format!("{branch} was pushed: rebase it by hand")));
    }
    Ok(None)
}

/// The text of a file left in conflict with zdiff3 markers, settled when every hunk adds lines on both sides over an
/// empty base (RBI-2): the lines of the trunk (ours, in a rebase), then those of the branch.
fn settle(text: &str) -> Option<String> {
    enum Part {
        Outside,
        Ours,
        Base,
        Theirs,
    }
    let mut part = Part::Outside;
    let mut out = String::new();
    let mut hunks = 0;
    let (mut ours, mut base_lines, mut theirs) = (String::new(), 0, String::new());
    for line in text.split_inclusive('\n') {
        let marker = line.trim_end_matches(['\n', '\r']);
        match part {
            Part::Outside if marker.starts_with("<<<<<<<") => part = Part::Ours,
            Part::Outside => out.push_str(line),
            Part::Ours if marker.starts_with("|||||||") => part = Part::Base,
            Part::Ours if marker.starts_with("=======") => return None,
            Part::Ours => ours.push_str(line),
            Part::Base if marker.starts_with("=======") => part = Part::Theirs,
            Part::Base => base_lines += 1,
            Part::Theirs if marker.starts_with(">>>>>>>") => {
                if base_lines > 0 || ours.is_empty() || theirs.is_empty() {
                    return None;
                }
                out.push_str(&ours);
                out.push_str(&theirs);
                (ours, base_lines, theirs) = (String::new(), 0, String::new());
                hunks += 1;
                part = Part::Outside;
            }
            Part::Theirs => theirs.push_str(line),
        }
    }
    (matches!(part, Part::Outside) && hunks > 0).then_some(out)
}

/// Settles the conflicts of the commit the rebase stopped on: the files it settled, or those it could not.
fn settle_all(wt: &Path) -> Result<Result<Vec<String>, Vec<String>>, String> {
    let names = git(wt, &["diff", "--name-only", "--diff-filter=U", "-z"])?;
    let files: Vec<&str> = names.split('\0').filter(|f| !f.is_empty()).collect();
    let mut settled = Vec::new();
    let mut unsettled = Vec::new();
    for file in &files {
        let stages = git(wt, &["ls-files", "-u", "-z", "--", file])?;
        let count = stages.split('\0').filter(|s| !s.is_empty()).count();
        let text = std::fs::read(wt.join(file))
            .ok()
            .and_then(|b| String::from_utf8(b).ok());
        match text.filter(|_| count == 3).as_deref().and_then(settle) {
            Some(new) => settled.push((file.to_string(), new)),
            None => unsettled.push(file.to_string()),
        }
    }
    if files.is_empty() || !unsettled.is_empty() {
        return Ok(Err(unsettled));
    }
    for (file, new) in &settled {
        std::fs::write(wt.join(file), new).map_err(|e| format!("cannot write {file}: {e}"))?;
        git(wt, &["add", "--", file])?;
    }
    Ok(Ok(settled.into_iter().map(|(f, _)| f).collect()))
}

/// Rebases the branch of the `merge pending` claim of `stem`: the lines to print on stdout, or why it did not.
fn rebase(dir: &Path, stem: &str) -> Result<Vec<String>, String> {
    let branch = issue_claim::read(dir, stem)
        .filter(|(_, c)| c.state == "merge pending")
        .map(|(_, c)| c.branch)
        .ok_or_else(|| format!("{stem}: not merge pending"))?;
    let (trunk, wt) = locate(dir, &branch)?;
    let onto = format!("refs/heads/{trunk}");
    let ancestor = ["merge-base", "--is-ancestor", &onto, "HEAD"];
    match run(&wt, &ancestor)? {
        (0, _, _) => return Ok(vec![format!("{branch} already holds {trunk}")]),
        (1, _, _) => {}
        (code, _, err) => return Err(failure(&ancestor, code, &err)),
    }
    if let Some(why) = refusal(&wt, &branch, &trunk)? {
        return Err(why);
    }
    let mut lines = Vec::new();
    let mut git_step: Vec<&str> = vec!["rebase", "--quiet", &onto];
    loop {
        let (code, _, err) = run(&wt, &git_step)?;
        if code == 0 {
            lines.push(format!("rebased {branch} onto {trunk}"));
            return Ok(lines);
        }
        let in_progress = ["rebase-merge", "rebase-apply"]
            .iter()
            .any(|d| git(&wt, &["rev-parse", "--git-path", d]).is_ok_and(|p| wt.join(p).exists()));
        if !in_progress {
            return Err(failure(&git_step, code, &err));
        }
        match settle_all(&wt) {
            Ok(Ok(files)) => lines.extend(files.into_iter().map(|f| format!("settled {f}"))),
            other => {
                git(&wt, &["rebase", "--abort"])?;
                let files = match other {
                    Ok(Err(files)) if !files.is_empty() => files,
                    Ok(_) => return Err(failure(&git_step, code, &err)),
                    Err(e) => return Err(e),
                };
                return Err(format!(
                    "{branch}: conflict in {}: rebase aborted, settle it in {}",
                    files.join(", "),
                    wt.display()
                ));
            }
        }
        git_step = vec!["rebase", "--continue"];
    }
}

pub(super) fn main(args: &[String]) -> u8 {
    let [stem] = args else {
        eprintln!("{USAGE}");
        return 2;
    };
    match rebase(Path::new("."), stem) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            0
        }
        Err(why) => {
            eprintln!("{why}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::settle;

    #[test]
    fn keeps_both_sides_of_an_addition_over_an_empty_base() {
        let text = "a\n<<<<<<< HEAD\nx\n||||||| base\n=======\ny\n>>>>>>> c\nb\n";
        assert_eq!(settle(text).as_deref(), Some("a\nx\ny\nb\n"));
    }

    #[test]
    fn refuses_a_hunk_that_changes_a_line_of_the_base() {
        let text = "<<<<<<< HEAD\nx\n||||||| base\nold\n=======\ny\n>>>>>>> c\n";
        assert_eq!(settle(text), None);
    }

    #[test]
    fn refuses_a_hunk_without_its_base() {
        assert_eq!(settle("<<<<<<< HEAD\nx\n=======\ny\n>>>>>>> c\n"), None);
    }
}
