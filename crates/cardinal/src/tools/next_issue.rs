//! Print the next issue to work on: the open issue of lowest `rank` in its front matter, with its title, its `why`
//! and the rest of the order (ADR 0019). Reports (stderr) the issues without a rank and the shared ranks.
//! An issue is taken, and skipped, when a git worktree has a branch named after its stem checked out, or when such
//! a branch holds work that is not on the branch of the main worktree (patch-equivalent commits and a squash of any
//! number of commits count as on it: merging the branch would change no file). A branch with neither (left over after a merge, empty) is reported `Stale:` and takes nothing. The mark
//! comes from git alone, so every worktree sees it.
//! Usage: `just next-issue` (from the repo root, or pass the `issues` directory as argument).

use std::path::Path;
use std::process::Command;

fn title(dir: &Path, stem: &str) -> String {
    std::fs::read_to_string(dir.join(format!("{stem}.md")))
        .ok()
        .and_then(|t| {
            let body = split_front_matter(&t).map_or(0, |(_, _, body)| body);
            t[body..]
                .lines()
                .find_map(|l| l.strip_prefix("# ").map(str::to_string))
        })
        .unwrap_or_else(|| "(no file)".to_string())
}

fn git_lines(dir: &Path, args: &[&str]) -> Vec<String> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `branch` holds work that `trunk` lacks; true when git cannot tell, so unmerged work is never ignored.
/// No work when every commit has a patch-equivalent on `trunk` (`git cherry`), or when merging the branch into `trunk`
/// would change no file (a squash of any number of commits): the merge tree is then the tree of `trunk`. Keep in sync
/// with `finish` of `issue_worktree.rs`.
fn has_own_commits(dir: &Path, trunk: &str, branch: &str) -> bool {
    let branch = format!("refs/heads/{branch}");
    let equivalent = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["cherry", trunk, &branch])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| {
            !String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|l| l.starts_with('+'))
        });
    if equivalent {
        return false;
    }
    let merged = git_lines(dir, &["merge-tree", "--write-tree", trunk, &branch])
        .first()
        .cloned();
    let trunk_tree = git_lines(dir, &["rev-parse", &format!("{trunk}^{{tree}}")])
        .first()
        .cloned();
    merged.is_none() || merged != trunk_tree
}

/// A local branch: the path of the worktree that has it checked out, if any, and whether it takes its issue.
struct Branch {
    name: String,
    worktree: Option<String>,
    taken: bool,
}

/// The branches of the repository that holds `dir`. The branch of the main worktree is left out: it is the trunk,
/// not an issue. Empty outside a git repository. Only the branches named after one of `stems` are examined for commits.
fn branches(dir: &Path, stems: &[&str]) -> Vec<Branch> {
    let mut out: Vec<(String, Option<String>)> = git_lines(
        dir,
        &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
    )
    .into_iter()
    .map(|b| (b, None))
    .collect();
    let (mut path, mut first) = (String::new(), true);
    let mut trunk = String::new();
    for line in git_lines(dir, &["worktree", "list", "--porcelain"]) {
        if let Some(p) = line.strip_prefix("worktree ") {
            path = p.to_string();
        } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
            if first {
                out.retain(|(name, _)| name != b);
                trunk = b.to_string();
            } else if let Some(entry) = out.iter_mut().find(|(name, _)| name == b) {
                entry.1 = Some(path.clone());
            }
            first = false;
        } else if let Some(h) = line
            .strip_prefix("HEAD ")
            .filter(|_| first && trunk.is_empty())
        {
            trunk = h.to_string();
        } else if line.is_empty() {
            first = false;
        }
    }
    out.into_iter()
        .map(|(name, worktree)| {
            let taken = worktree.is_some()
                || trunk.is_empty()
                || !stems.contains(&name.as_str())
                || has_own_commits(dir, &trunk, &name);
            Branch {
                name,
                worktree,
                taken,
            }
        })
        .collect()
}

/// The stems of the issue files of `dir`, sorted.
fn issue_stems(dir: &Path) -> Vec<String> {
    let mut stems: Vec<String> = std::fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "md"))
                .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .collect()
        })
        .unwrap_or_default();
    stems.sort();
    stems
}

/// The lines of the front matter of `text` (between a first line `---` and the next `---`, line endings kept) and
/// the offsets of its closing line and of the body; `None` when the file opens no closed front matter.
fn split_front_matter(text: &str) -> Option<(Vec<&str>, usize, usize)> {
    let mut lines = text.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end() != "---" {
        return None;
    }
    let (mut at, mut matter) = (first.len(), Vec::new());
    for l in lines {
        if l.trim_end() == "---" {
            return Some((matter, at, at + l.len()));
        }
        at += l.len();
        matter.push(l);
    }
    None
}

/// `value` without the single (a doubled `'` inside stands for one) or double quotes around it.
fn unquote(value: &str) -> String {
    let inner = |q: char| {
        (value.len() >= 2 && value.starts_with(q) && value.ends_with(q))
            .then(|| &value[1..value.len() - 1])
    };
    if let Some(v) = inner('\'') {
        v.replace("''", "'")
    } else {
        inner('"').unwrap_or(value).to_string()
    }
}

/// The value of `key` in the front matter of `text`, unquoted.
fn front_matter(text: &str, key: &str) -> Option<String> {
    let (matter, _, _) = split_front_matter(text)?;
    matter
        .iter()
        .find_map(|l| l.strip_prefix(key)?.strip_prefix(':'))
        .map(|v| unquote(v.trim()))
}

/// The order of the `rank` of each issue file (NXT-1): ranked first, by rank then stem, then the unranked ones,
/// reported (NXT-2); a shared rank is reported without drift (NXT-3).
fn from_ranks(dir: &Path) -> (Vec<(String, String)>, bool) {
    let mut ranked: Vec<(u64, String, String)> = Vec::new();
    let mut unranked = Vec::new();
    for stem in issue_stems(dir) {
        let text = std::fs::read_to_string(dir.join(format!("{stem}.md"))).unwrap_or_default();
        let why = front_matter(&text, "why").unwrap_or_default();
        match front_matter(&text, "rank").and_then(|r| r.parse().ok()) {
            Some(rank) => ranked.push((rank, stem, why)),
            None => unranked.push((stem, why)),
        }
    }
    ranked.sort();
    for group in ranked.chunk_by(|a, b| a.0 == b.0).filter(|g| g.len() > 1) {
        let stems: Vec<&str> = group.iter().map(|(_, s, _)| s.as_str()).collect();
        eprintln!("same rank {}: {}", group[0].0, stems.join(", "));
    }
    for (stem, _) in &unranked {
        eprintln!("issue without a rank: {stem}");
    }
    let drift = !unranked.is_empty();
    let entries = ranked
        .into_iter()
        .map(|(_, s, w)| (s, w))
        .chain(unranked)
        .collect();
    (entries, drift)
}

/// The open issues of a directory in order, with their `why`, and the branches that may take them.
struct Queue {
    entries: Vec<(String, String)>,
    drift: bool,
    branches: Vec<Branch>,
}

impl Queue {
    /// The order of the ranks (NXT-1); the drift is reported on stderr.
    fn read(dir: &Path) -> Self {
        let (entries, drift) = from_ranks(dir);
        let stems: Vec<&str> = entries.iter().map(|(s, _)| s.as_str()).collect();
        let branches = branches(dir, &stems);
        Queue {
            entries,
            drift,
            branches,
        }
    }

    /// The branch that takes the issue `stem`, if any (PAR-4).
    fn taken_by(&self, stem: &str) -> Option<&Branch> {
        self.branches.iter().find(|b| b.name == stem && b.taken)
    }

    /// The issues no branch takes, with their position in the order: `(index, stem, why)`.
    fn free(&self) -> Vec<(usize, &str, &str)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, (s, _))| self.taken_by(s).is_none())
            .map(|(i, (s, w))| (i, s.as_str(), w.as_str()))
            .collect()
    }

    /// What `next-issue` offers first: `Next: issues/<stem>.md` and the title of the first free issue, else the line
    /// saying there is none.
    fn offer(&self, dir: &Path) -> String {
        match self.free().first() {
            None if self.entries.is_empty() => "No open issue.".to_string(),
            None => "Every open issue is taken.".to_string(),
            Some((_, stem, _)) => format!("Next: issues/{stem}.md\n  {}", title(dir, stem)),
        }
    }
}

/// The issue `next-issue` offers first in `dir` (CLS-7: `close-issue` prints it after a close).
pub(super) fn next(dir: &Path) -> String {
    Queue::read(dir).offer(dir)
}

pub(super) fn main(args: &[String]) -> u8 {
    let dir = args.first().map_or("issues", String::as_str);
    let dir = Path::new(dir);
    if !dir.is_dir() {
        eprintln!("no {} directory", dir.display());
        std::process::exit(2);
    }
    let queue = Queue::read(dir);
    let free = queue.free();
    println!("{}", queue.offer(dir));
    if let Some((_, _, why)) = free.first() {
        println!("  why here: {why}");
        for (i, s, _) in free.iter().skip(1) {
            println!("{:>3}. {s}", i + 1);
        }
    }
    let taken = |stem: &str| queue.taken_by(stem).map(|b| b.worktree.as_deref());
    let stale = |stem: &str| queue.branches.iter().any(|b| b.name == stem && !b.taken);
    for (stem, _) in &queue.entries {
        if let Some(path) = taken(stem) {
            println!(
                "Taken: {stem}{}",
                path.map_or(String::new(), |p| format!(" (worktree {p})"))
            );
        }
    }
    for (stem, _) in &queue.entries {
        if stale(stem) {
            println!("Stale: {stem} (branch with no worktree and nothing off the main branch; run `just finish-issue {stem}`)");
        }
    }
    std::process::exit(i32::from(queue.drift));
}
