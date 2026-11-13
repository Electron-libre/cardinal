//! The files a change pushes over the file size limit of `skill:issue-workflow`, or grows while over it.
//! Usage: `cardinal run file-size [<rev> | <a>..<b> | --all]`: no argument compares the working tree and its untracked
//! files with `HEAD` (FSZ-1), `<rev>` with that revision, a range the files at `<b>` with those at `<a>` (FSZ-2),
//! `--all` lists every file over the limit (FSZ-3). Prints `<lines>\t<path>` per file, with ` (was <before>)` when it
//! existed before, then the count against the limit; exits 0 when none is listed, 1 otherwise, 3 on an error (FSZ-7,
//! FSZ-8). git counts the lines, as a diff from the empty tree, so a binary file counts 0 (FSZ-4, FSZ-5); the
//! working tree is read through the copy of the index of `deliverable-size` (FSZ-9).

use super::deliverable_size::{empty_tree, git, left_out, limit, working_tree_git};
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

pub fn main(args: &[String]) -> u8 {
    let target = match args {
        [] => Ok(Target::WorkingTree(None)),
        [all] if all == "--all" => Ok(Target::All),
        // git would read it as one of its options (FSZ-8)
        [option] if option.starts_with('-') => {
            Err(format!("`{option}` is not a revision, a range or --all"))
        }
        [range] if range.contains("..") => {
            let (range, merge_base) = match range.split_once("...") {
                Some(sides) => (sides, true),
                None => (range.split_once("..").unwrap_or_default(), false),
            };
            Ok(Target::Range(range.0, range.1, merge_base))
        }
        [rev] => Ok(Target::WorkingTree(Some(rev.as_str()))),
        _ => Err("usage: file-size [<rev> | <a>..<b> | --all]".to_string()),
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

/// The files over `limit`: each with its lines and, for a change, its lines before when it existed.
fn listed<'a>(
    root: &Path,
    target: &Target<'a>,
    limit: u64,
) -> Result<Vec<(u64, String, Option<u64>)>, String> {
    let empty = empty_tree(root)?;
    let (base, head) = match target {
        Target::All => {
            let after = lines(root, &empty, None, &[])?;
            return Ok(after
                .into_iter()
                .filter(|(_, n)| *n > limit)
                .map(|(path, n)| (n, path, None))
                .collect());
        }
        Target::WorkingTree(Some(rev)) => ((*rev).to_string(), None),
        Target::WorkingTree(None) => {
            let head = git(root, &["rev-parse", "--verify", "-q", "HEAD"], None, None);
            // no commit yet: every file is new (FSZ-1)
            let base = if head.is_ok() {
                "HEAD".to_string()
            } else {
                empty.clone()
            };
            (base, None)
        }
        Target::Range(a, b, merge_base) => {
            let or_head = |rev: &'a str| if rev.is_empty() { "HEAD" } else { rev };
            let (a, b) = (or_head(a), or_head(b));
            let base = if *merge_base {
                let id = git(root, &["merge-base", a, b], None, None)?;
                String::from_utf8_lossy(&id).trim().to_string()
            } else {
                a.to_string()
            };
            (base, Some(b))
        }
    };
    let changes = changes(root, &base, head)?;
    if changes.is_empty() {
        return Ok(Vec::new());
    }
    let news: Vec<&str> = changes.iter().map(|(_, new)| new.as_str()).collect();
    let after = lines(root, &empty, head, &news)?;
    let olds: Vec<&str> = changes
        .iter()
        .filter_map(|(old, _)| old.as_deref())
        .collect();
    let before = if olds.is_empty() {
        BTreeMap::new()
    } else {
        lines(root, &empty, Some(&base), &olds)?
    };
    Ok(changes
        .into_iter()
        .filter_map(|(old, new)| {
            let n = *after.get(&new)?;
            let was = old.map(|old| before.get(&old).copied().unwrap_or(0));
            (n > limit && n > was.unwrap_or(0)).then_some((n, new, was))
        })
        .collect())
}

/// The files changed from `base` to `head` (the working tree when `None`), deleted ones left out: each with the path
/// it had at `base` when it existed there, a rename paired with its old path (FSZ-1, FSZ-2).
fn changes(
    root: &Path,
    base: &str,
    head: Option<&str>,
) -> Result<Vec<(Option<String>, String)>, String> {
    let mut args = vec!["diff", "--name-status", "-z", "-M", "--no-ext-diff", base];
    args.extend(head);
    args.push("--");
    let out = match head {
        Some(_) => git(root, &args, None, None)?,
        None => working_tree_git(root, &args)?,
    };
    let text = String::from_utf8_lossy(&out);
    let mut fields = text.split('\0').filter(|f| !f.is_empty());
    let mut changes = Vec::new();
    while let Some(status) = fields.next() {
        let path = fields.next().unwrap_or_default().to_string();
        match status.as_bytes().first() {
            Some(b'R' | b'C') => {
                let new = fields.next().unwrap_or_default().to_string();
                changes.push((Some(path), new));
            }
            Some(b'A') => changes.push((None, path)),
            Some(b'D') => {}
            _ => changes.push((Some(path.clone()), path)),
        }
    }
    Ok(changes)
}

/// The lines of each file at `rev` (the working tree when `None`), restricted to `paths` unless empty, as git counts
/// them in a diff from the empty tree: a last line with no newline counts, a binary file counts 0 (FSZ-4, FSZ-5).
fn lines(
    root: &Path,
    empty: &str,
    rev: Option<&str>,
    paths: &[&str],
) -> Result<BTreeMap<String, u64>, String> {
    let mut args = vec![
        "--literal-pathspecs",
        "diff",
        "--numstat",
        "-z",
        "--no-renames",
        "--no-ext-diff",
        empty,
    ];
    args.extend(rev);
    args.push("--");
    args.extend(paths);
    let out = match rev {
        Some(_) => git(root, &args, None, None)?,
        None => working_tree_git(root, &args)?,
    };
    let text = String::from_utf8_lossy(&out);
    Ok(text
        .split('\0')
        .filter_map(|record| {
            let mut parts = record.splitn(3, '\t');
            let added = parts.next()?.parse::<u64>().unwrap_or(0);
            let _removed = parts.next()?;
            Some((parts.next()?.to_string(), added))
        })
        .collect())
}
