//! The git history of a repository as `delivered-quality` reads it (QUA-2, QUA-4): its non-merge commits, those of an
//! issue, and the reverts and the blamed writers of the lines a later fix changes.

use super::super::transcripts::{default_branch, git};
use super::{section, Issues, Section, UNRELATED_FILES, UNRELATED_PREFIXES};

use chrono::{DateTime, FixedOffset};
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

/// A non-merge commit of the default branch.
pub(super) struct Commit {
    pub(super) hash: String,
    pub(super) date: DateTime<FixedOffset>,
    pub(super) subject: String,
    pub(super) body: String,
    /// The paths it changed, renames read as a deletion and an addition.
    pub(super) paths: Vec<String>,
    /// The paths it added.
    pub(super) added: Vec<String>,
}

impl Commit {
    /// The Conventional Commits type of its subject (feature `conventional-commits`), empty when it has none.
    pub(super) fn kind(&self) -> &str {
        let head = self.subject.split(':').next().unwrap_or_default();
        let kind = head
            .split('(')
            .next()
            .unwrap_or_default()
            .trim_end_matches('!');
        if self.subject.contains(':')
            && !kind.is_empty()
            && kind.chars().all(|c| c.is_ascii_lowercase())
        {
            kind
        } else {
            ""
        }
    }

    pub(super) fn touches(&self, path: &str) -> bool {
        self.paths.iter().any(|p| p == path)
    }
}

/// The non-merge commits of the default branch of `repo`, oldest first.
pub(super) fn commits(repo: &Path) -> Vec<Commit> {
    let Some(branch) = default_branch(repo) else {
        return Vec::new();
    };
    let Some(log) = git(
        repo,
        &[
            "-c",
            "core.quotePath=false",
            "log",
            "--no-merges",
            "--no-renames",
            "--reverse",
            "--name-status",
            "--format=%x1e%H%x1f%aI%x1f%B%x1f",
            branch,
        ],
    ) else {
        return Vec::new();
    };
    log.split('\x1e')
        .filter_map(|record| {
            let mut fields = record.splitn(4, '\x1f');
            let hash = fields.next()?.trim().to_string();
            let date = DateTime::parse_from_rfc3339(fields.next()?).ok()?;
            let message = fields.next()?;
            let status = fields.next().unwrap_or_default();
            let mut paths = Vec::new();
            let mut added = Vec::new();
            for line in status.lines().filter(|l| !l.trim().is_empty()) {
                let mut parts = line.split('\t');
                let code = parts.next().unwrap_or_default();
                if let Some(path) = parts.next() {
                    if code == "A" {
                        added.push(path.to_string());
                    }
                    paths.push(path.to_string());
                }
            }
            Some(Commit {
                hash,
                date,
                subject: message.lines().next().unwrap_or_default().to_string(),
                body: message.to_string(),
                paths,
                added,
            })
        })
        .collect()
}

/// The indexes in `log` of the commits of the issue `stem` (QUA-2), oldest first; empty when it was never closed on
/// the default branch.
pub(super) fn issue_commits(log: &[Commit], issues: &mut Issues, stem: &str) -> Vec<usize> {
    let Some((start, Some(close))) = issues.span(log, stem) else {
        return Vec::new();
    };
    let done = format!("issues/done/{stem}.md");
    let closes_another = |c: &Commit| {
        c.added
            .iter()
            .any(|p| p.starts_with("issues/done/") && *p != done)
    };
    issues.files[stem]
        .iter()
        .copied()
        .filter(|&i| i >= start && i <= close && (i == close || !closes_another(&log[i])))
        .collect()
}

/// Whether the version of the file of the issue `stem` that `commit` wrote holds the `# Tracking` heading outside a
/// fence (QUA-12): the oldest such commit starts the work of the issue (QUA-2), the version under `issues/done/` once
/// it moved there.
pub(super) fn starts_work(repo: &Path, commit: &Commit, stem: &str) -> bool {
    let done = format!("issues/done/{stem}.md");
    let path = if commit.touches(&done) {
        done
    } else {
        format!("issues/{stem}.md")
    };
    git(
        repo,
        &["cat-file", "blob", &format!("{}:{path}", commit.hash)],
    )
    .is_some_and(|text| matches!(section(&text, "# Tracking", |_| true), Section::Read(_)))
}

/// Whether `commit` reverts the work of the issue (QUA-4): a `revert` type or a `Revert "` subject changing one of
/// its `files`, or a `This reverts commit <hash>` citing one of its `own` commits.
pub(super) fn reverts(commit: &Commit, files: &BTreeSet<&str>, own: &[&Commit]) -> bool {
    let typed = commit.kind() == "revert" || commit.subject.starts_with("Revert \"");
    let cited = commit
        .body
        .split("This reverts commit ")
        .skip(1)
        .any(|rest| {
            let hash: String = rest.chars().take_while(char::is_ascii_hexdigit).collect();
            hash.len() >= 7 && own.iter().any(|c| c.hash.starts_with(&hash))
        });
    cited || typed && commit.paths.iter().any(|p| files.contains(p.as_str()))
}

/// Whether `path` says nothing of an issue's work (QUA-4): an issue file, a doc or the lock of the sync.
pub(super) fn unrelated(path: &str) -> bool {
    UNRELATED_PREFIXES.iter().any(|d| path.starts_with(d)) || UNRELATED_FILES.contains(&path)
}

/// The commits that last wrote, at the first parent of `fix`, the lines it removes or changes and, above a hunk that
/// only adds lines, the line right above it (QUA-4), in the paths `unrelated` leaves; renames followed, lines moved or
/// copied from another file of the same commit included. Empty for a root commit.
pub(super) fn blamed(repo: &Path, fix: &str) -> HashSet<String> {
    let parent = format!("{fix}^");
    // `-z` gives the paths unquoted, whatever the prefixes and the quoting of the diff headers
    let Some(status) = git(repo, &["diff", "-M", "--name-status", "-z", &parent, fix]) else {
        return HashSet::new();
    };
    let mut fields = status.split('\0').filter(|f| !f.is_empty());
    let mut writers = HashSet::new();
    while let Some(code) = fields.next() {
        let paired = code.starts_with('R') || code.starts_with('C');
        let (Some(old), new) = (fields.next(), if paired { fields.next() } else { None }) else {
            break;
        };
        // an added file or the source of a copy holds no line the fix removes
        if code.starts_with('A') || code.starts_with('C') || unrelated(old) {
            continue;
        }
        let mut args = vec![
            "diff",
            "-M",
            "-U0",
            "--no-color",
            "--no-ext-diff",
            &parent,
            fix,
            "--",
            old,
        ];
        args.extend(new);
        let Some(diff) = git(repo, &args) else {
            continue;
        };
        // a hunk header is the only line of a diff that starts with `@@`: every line of a hunk starts with ` `, `+` or `-`
        let ranges: Vec<String> = diff
            .lines()
            .filter_map(|l| l.strip_prefix("@@ -")?.split(' ').next())
            .filter_map(|old| {
                let (start, count) = old.split_once(',').unwrap_or((old, "1"));
                // a hunk that only adds lines sits below its `start`, 0 at the top of the file
                let (start, count) = (
                    start.parse::<usize>().ok()?,
                    count.parse::<usize>().ok()?.max(1),
                );
                (start > 0).then(|| format!("{start},{}", start + count - 1))
            })
            .collect();
        if ranges.is_empty() {
            continue;
        }
        let mut args = vec!["blame", "--porcelain", "-M", "-C"];
        for range in &ranges {
            args.extend(["-L", range]);
        }
        args.extend([parent.as_str(), "--", old]);
        // the user's `blame.ignoreRevsFile` makes blame fail in a repository without that file, or skip a commit
        let mut blame = crate::git::command(repo);
        blame
            .args(&args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1");
        let Ok(out) = crate::git::exec(blame, None) else {
            continue;
        };
        // the header line of each blamed group opens with the hash of its commit (40 hex digits, 64 with SHA-256)
        writers.extend(String::from_utf8_lossy(&out).lines().filter_map(|l| {
            let hash = l.split(' ').next()?;
            (hash.len() >= 40 && hash.chars().all(|c| c.is_ascii_hexdigit()))
                .then(|| hash.to_string())
        }));
    }
    writers
}
