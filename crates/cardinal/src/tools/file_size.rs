//! The files a change pushes over the file size limit of `skill:issue-workflow`, or grows while over it.
//! Usage: `cardinal run file-size [<rev> | <a>..<b> | <a>...<b> | --all]`: no argument compares the working tree and
//! its untracked files with `HEAD` (FSZ-1), `<rev>` with that revision, a range the files at `<b>` with those at `<a>`
//! or at their merge base (FSZ-2), `--all` lists every file over the limit (FSZ-3). Prints `<lines>\t<path>` per file,
//! with ` (was <before>)` when it existed before, then the count against the limit; exits 0 when none is listed, 1
//! otherwise, 3 on an error (FSZ-7, FSZ-8). git counts the lines: those of a file before the change as a diff from
//! the empty tree, those after as that count plus the lines the change added minus those it removed, so a binary
//! file counts nothing (FSZ-4, FSZ-5) and the working tree is read once, through the copy of the index of
//! `git_diff` (FSZ-9).

use super::git_diff::{empty_tree, git, left_out, limit, working_tree_git};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

/// What the files are compared with.
enum Target<'a> {
    /// The working tree against a revision, `HEAD` when none is given.
    WorkingTree(Option<&'a str>),
    /// The files at `.1` against those at `.0`, or at their merge base when `.2` (`<a>...<b>`); an empty side is
    /// `HEAD`, as git reads it.
    Range(&'a str, &'a str, bool),
    /// Every file of the working tree.
    All,
}

/// A listed file: its lines, its path, and its lines before the change when it existed.
type Listed = (u64, String, Option<u64>);

pub fn main(args: &[String]) -> u8 {
    let target = match args {
        [] => Ok(Target::WorkingTree(None)),
        [all] if all == "--all" => Ok(Target::All),
        [range] if range.contains("..") => {
            let (sides, merge_base) = match range.split_once("...") {
                Some(sides) => (sides, true),
                None => (range.split_once("..").unwrap_or_default(), false),
            };
            Ok(Target::Range(sides.0, sides.1, merge_base))
        }
        [rev] => Ok(Target::WorkingTree(Some(rev.as_str()))),
        _ => Err("usage: file-size [<rev> | <a>..<b> | <a>...<b> | --all]".to_string()),
    };
    match target.and_then(|target| report(&target)) {
        Ok((report, over)) => {
            print!("{report}");
            u8::from(over)
        }
        Err(e) => {
            eprintln!("file-size: {e}");
            crate::NOT_A_REFUSAL
        }
    }
}

/// The printed report and whether it lists a file.
fn report(target: &Target) -> Result<(String, bool), String> {
    let root = super::repo_root()?;
    let root = Path::new(&root);
    let limit = limit(root, "file_size_limit", 800, 0)?;
    let mut files = if limit == 0 {
        Vec::new()
    } else {
        // with the issue files off, `issues/` counts as any other directory (FEA-37)
        let issue_files = !crate::features::is_off(root, crate::features::ISSUE_FILES);
        listed(root, target, limit)?
            .into_iter()
            .filter(|(_, path, _)| !left_out(path, issue_files))
            .collect()
    };
    files.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let mut out = String::new();
    for (n, path, before) in &files {
        let _ = write!(out, "{n}\t{path}");
        if let Some(before) = before {
            let _ = write!(out, " (was {before})");
        }
        out.push('\n');
    }
    let _ = writeln!(out, "{} files over the limit of {limit} lines", files.len());
    Ok((out, !files.is_empty()))
}

/// The files over `limit`: those the change of `target` grew over it, or every one for `--all`.
fn listed(root: &Path, target: &Target, limit: u64) -> Result<Vec<Listed>, String> {
    let empty = empty_tree(root)?;
    let (base, numstat) = match target {
        Target::All => {
            let numstat = working_tree_git(root, &diff(&empty, None))?;
            return Ok(grown(&numstat, &BTreeMap::new(), limit)
                .into_iter()
                .map(|(n, path, _)| (n, path, None))
                .collect());
        }
        Target::WorkingTree(rev) => {
            let base = match rev {
                Some(rev) => commit(root, rev)?,
                None => commit(root, "HEAD").or_else(|e| {
                    // no commit yet: every file is new (FSZ-1)
                    let unborn = git(root, &["rev-parse", "-q", "--verify", "HEAD"], None, None);
                    unborn.is_err().then(|| empty.clone()).ok_or(e)
                })?,
            };
            let numstat = working_tree_git(root, &diff(&base, None))?;
            (base, numstat)
        }
        Target::Range(a, b, merge_base) => {
            let (a, b) = (commit(root, a)?, commit(root, b)?);
            let base = if *merge_base {
                let id = git(root, &["merge-base", &a, &b], None, None)?;
                String::from_utf8_lossy(&id).trim().to_string()
            } else {
                a
            };
            let numstat = git(root, &diff(&base, Some(&b)), None, None)?;
            (base, numstat)
        }
    };
    let before = if base == empty {
        BTreeMap::new()
    } else {
        let numstat = git(root, &diff(&empty, Some(&base)), None, None)?;
        grown(&numstat, &BTreeMap::new(), 0)
            .into_iter()
            .map(|(n, path, _)| (path, n))
            .collect()
    };
    Ok(grown(&numstat, &before, limit))
}

/// The commit `rev` names, one revision only: an empty one is `HEAD`, an option or a range is refused (FSZ-8).
fn commit(root: &Path, rev: &str) -> Result<String, String> {
    let rev = if rev.is_empty() { "HEAD" } else { rev };
    if rev.starts_with('-') {
        // git would read it as one of its options
        return Err(format!("`{rev}` is not a revision"));
    }
    let id = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "-q",
            "--end-of-options",
            &format!("{rev}^{{commit}}"),
        ],
        None,
        None,
    )
    .map_err(|_| format!("`{rev}` is not a commit"))?;
    Ok(String::from_utf8_lossy(&id).trim().to_string())
}

/// The `git diff --numstat` from `base` to `head`, the working tree when `None`, a rename paired with its old path.
fn diff<'a>(base: &'a str, head: Option<&'a str>) -> Vec<&'a str> {
    let mut args = vec!["diff", "--numstat", "-z", "-M", "--no-ext-diff", base];
    args.extend(head);
    args.push("--");
    args
}

/// The files of a `git diff --numstat -z` whose lines after it exceed `limit` and their lines before it, read in
/// `before` by their old path: the lines after are those before plus the added minus the removed. A binary file, `-`
/// for both, and a deleted one are left out (FSZ-1, FSZ-5).
fn grown(numstat: &[u8], before: &BTreeMap<String, u64>, limit: u64) -> Vec<Listed> {
    let text = String::from_utf8_lossy(numstat);
    let mut fields = text.split('\0');
    let mut listed = Vec::new();
    while let Some(record) = fields.next() {
        let mut parts = record.splitn(3, '\t');
        let (Some(added), Some(removed), Some(path)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let (old, new) = if path.is_empty() {
            // a rename: its old path, then its new one
            let old = fields.next().unwrap_or_default();
            (old, fields.next().unwrap_or_default())
        } else {
            (path, path)
        };
        let (Ok(added), Ok(removed)) = (added.parse::<u64>(), removed.parse::<u64>()) else {
            continue;
        };
        let was = before.get(old).copied();
        let after = (was.unwrap_or(0) + added).saturating_sub(removed);
        if after > limit && after > was.unwrap_or(0) {
            listed.push((after, new.to_string(), was));
        }
    }
    listed
}
