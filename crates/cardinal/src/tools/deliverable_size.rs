//! The changed lines of a deliverable, or of a whole issue, against the size limit of `skill:issue-workflow`.
//! Usage: `cardinal run deliverable-size [<rev> | <a>..<b>]`: no argument counts the working tree and its untracked
//! files against `HEAD` (SIZ-1), `<rev>` against that revision (SIZ-2), a range its commits alone (SIZ-3). Prints
//! `<lines>\t<path>` per file, largest first, then the total against the limit; exits 0 within it, 1 over it, 3 on an
//! error (SIZ-9, SIZ-10).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

/// The lock files, left out with `issues/` (SIZ-4).
const LOCK_FILES: &[&str] = &[
    "*.lock",
    "package-lock.json",
    "npm-shrinkwrap.json",
    "pnpm-lock.yaml",
    "go.sum",
];

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
    let root = super::repo_root()?;
    let root = Path::new(&root);
    let (key, default) = match target {
        None => ("deliverable_size_limit", 1000),
        Some(_) => ("issue_size_limit", 3000),
    };
    let limit = limit(root, key, default)?;
    let mut counts = diff(root, target.unwrap_or("HEAD"))?;
    if !target.is_some_and(|t| t.contains("..")) {
        for path in untracked(root)? {
            let bytes =
                std::fs::read(root.join(&path)).map_err(|e| format!("cannot read {path}: {e}"))?;
            *counts.entry(path).or_default() += line_count(&bytes);
        }
    }
    let mut files: Vec<(u64, String)> = counts
        .into_iter()
        .filter(|(path, n)| *n > 0 && !left_out(path))
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

/// The positive integer `key` of the `[project]` table of the configuration, `default` when the file or the key is
/// absent (SIZ-8, SIZ-10).
fn limit(root: &Path, key: &str, default: u64) -> Result<u64, String> {
    let path = crate::config_of(root);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(default);
    };
    let table: toml::Table = text
        .parse()
        .map_err(|e| format!("invalid {}: {e}", path.display()))?;
    match table.get("project").and_then(|p| p.get(key)) {
        None => Ok(default),
        Some(value) => value
            .as_integer()
            .and_then(|n| u64::try_from(n).ok())
            .filter(|n| *n > 0)
            .ok_or_else(|| format!("{key} of {} is not a positive integer", path.display())),
    }
}

/// The added plus removed lines of each file `git diff <target>` changes, renames detected only when exact, so a
/// renamed and edited file counts its whole old and new content (SIZ-5, SIZ-6); a binary file counts 0 (SIZ-7).
fn diff(root: &Path, target: &str) -> Result<BTreeMap<String, u64>, String> {
    let out = git(
        root,
        &[
            "diff",
            "--numstat",
            "-z",
            "-M100%",
            "--no-ext-diff",
            target,
            "--",
        ],
    )?;
    let mut counts = BTreeMap::new();
    let mut fields = out.split('\0');
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
    Ok(counts)
}

/// The untracked files git does not ignore, nested repositories left out as `repo_files` does.
fn untracked(root: &Path) -> Result<Vec<String>, String> {
    let out = git(root, &["ls-files", "-o", "--exclude-standard", "-z"])?;
    Ok(out
        .split('\0')
        .filter(|p| !p.is_empty() && !p.ends_with('/'))
        .filter(|p| root.join(p).symlink_metadata().is_ok_and(|m| m.is_file()))
        .map(String::from)
        .collect())
}

/// The lines of a text file, a last one without its newline included; 0 for a binary one (a NUL byte).
fn line_count(bytes: &[u8]) -> u64 {
    if bytes.contains(&0) {
        return 0;
    }
    String::from_utf8_lossy(bytes).lines().count() as u64
}

fn left_out(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    path.starts_with("issues/")
        || LOCK_FILES
            .iter()
            .any(|p| super::check_specs::name_matches(p, name))
}

/// The stdout of `git <args>` run in `root`, or git's message.
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
