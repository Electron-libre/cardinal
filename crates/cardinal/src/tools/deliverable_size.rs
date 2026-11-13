//! The changed lines of a deliverable, or of a whole issue, against the size limit of `skill:issue-workflow`.
//! Usage: `cardinal run deliverable-size [<rev> | <a>..<b>]`: no argument counts the working tree and its untracked
//! files against `HEAD` (SIZ-1), `<rev>` against that revision (SIZ-2), a range its commits alone (SIZ-3). Prints
//! `<lines>\t<path>` per file, largest first, then the total against the limit; exits 0 within it, 1 over it, 3 on an
//! error (SIZ-9, SIZ-10). git does all the counting: the untracked files are marked intent-to-add in a copy of the
//! index, so `git diff` pairs a plain `mv` as a rename and reads their binary attributes (SIZ-5, SIZ-7, SIZ-11).

use super::git_diff::{empty_tree, git, left_out, limit, working_tree_git};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

pub fn main(args: &[String]) -> u8 {
    let target = match args {
        [] => None,
        [target] => Some(target.as_str()),
        _ => {
            eprintln!("usage: deliverable-size [<rev> | <a>..<b>]");
            return crate::NOT_A_REFUSAL;
        }
    };
    match report(target) {
        Ok((report, over)) => {
            print!("{report}");
            u8::from(over)
        }
        Err(e) => {
            eprintln!("deliverable-size: {e}");
            crate::NOT_A_REFUSAL
        }
    }
}

/// The printed report and whether its total exceeds the limit.
fn report(target: Option<&str>) -> Result<(String, bool), String> {
    if let Some(option) = target.filter(|t| t.starts_with('-')) {
        // git would read it as one of its options (SIZ-10)
        return Err(format!("`{option}` is not a revision or a range"));
    }
    let root = super::repo_root()?;
    let root = Path::new(&root);
    let (key, default) = match target {
        None => ("deliverable_size_limit", 1000),
        Some(_) => ("issue_size_limit", 3000),
    };
    let limit = limit(root, key, default, 1)?;
    let numstat = match target {
        Some(range) if range.contains("..") => git(root, &diff_args(range), None, None)?,
        Some(rev) => working_tree_diff(root, rev)?,
        None if git(root, &["rev-parse", "--verify", "-q", "HEAD"], None, None).is_ok() => {
            working_tree_diff(root, "HEAD")?
        }
        // no commit yet: everything counts as added (SIZ-1)
        None => working_tree_diff(root, &empty_tree(root)?)?,
    };
    // with the issue files off, `issues/` counts as any other directory (FEA-37)
    let issue_files = !crate::features::is_off(root, crate::features::ISSUE_FILES);
    let mut files: Vec<(u64, String)> = counts(&numstat)
        .into_iter()
        .filter(|(path, n)| *n > 0 && !left_out(path, issue_files))
        .map(|(path, n)| (n, path))
        .collect();
    files.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let total: u64 = files.iter().map(|(n, _)| n).sum();
    let over = total > limit;
    let verdict = if over { "over" } else { "within" };
    let mut out = String::new();
    for (n, path) in &files {
        let _ = writeln!(out, "{n}\t{path}");
    }
    let _ = writeln!(out, "{total} changed lines, limit {limit}: {verdict}");
    Ok((out, over))
}

/// `git diff --numstat` of `target`, renames detected only when exact, so a renamed and edited file counts its whole
/// old and new content (SIZ-5, SIZ-6).
fn diff_args(target: &str) -> [&str; 7] {
    [
        "diff",
        "--numstat",
        "-z",
        "-M100%",
        "--no-ext-diff",
        target,
        "--",
    ]
}

/// The diff of the working tree against `rev`, the untracked files included as new ones (SIZ-11).
fn working_tree_diff(root: &Path, rev: &str) -> Result<Vec<u8>, String> {
    working_tree_git(root, &diff_args(rev))
}

/// The added plus removed lines of each path of a `git diff --numstat -z`; a binary file, `-` for both, counts 0
/// (SIZ-7).
fn counts(numstat: &[u8]) -> BTreeMap<String, u64> {
    let text = String::from_utf8_lossy(numstat);
    let mut counts = BTreeMap::new();
    let mut fields = text.split('\0');
    while let Some(record) = fields.next() {
        let mut parts = record.splitn(3, '\t');
        let (Some(added), Some(removed), Some(path)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let path = if path.is_empty() {
            // an exact rename: its old path, then its new one
            fields.next();
            fields.next().unwrap_or_default()
        } else {
            path
        };
        let n = added.parse::<u64>().unwrap_or(0) + removed.parse::<u64>().unwrap_or(0);
        *counts.entry(path.to_string()).or_default() += n;
    }
    counts
}
