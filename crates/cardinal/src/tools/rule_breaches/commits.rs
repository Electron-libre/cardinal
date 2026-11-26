//! The commit messages as `rule-breaches` checks them (BRE-9): the AI attribution and the commit convention of each
//! non-merge commit of the local branches, against the session it is tied to.

use super::super::commit_check::{BUILT_IN_SCOPES, TYPES};
use super::super::transcripts::{self, commit_config, git};
use super::Record;
use crate::features;
use crate::project_table::ProjectTable;

use chrono::DateTime;
use std::path::{Path, PathBuf};

/// Whether a commit message carries an AI or tool attribution.
fn attributed(message: &str) -> bool {
    message.lines().map(str::to_lowercase).any(|l| {
        l.contains("co-authored-by:")
            || l.contains("noreply@anthropic.com")
            || (l.contains("generated with") && l.contains("claude"))
    })
}

/// The built-in scope `cardinal` before the rename, still read in the past commits.
const OLD_SCOPE: &str = "harness";

/// Whether a subject follows `<type>(<scope>): <description>` (scope optional) with one of `scopes`.
fn conventional(subject: &str, scopes: &[String]) -> bool {
    let Some((head, description)) = subject.split_once(": ") else {
        return false;
    };
    let (kind, scope) = match head.split_once('(') {
        Some((kind, rest)) => match rest.strip_suffix(')') {
            Some(scope) => (kind, Some(scope)),
            None => return false,
        },
        None => (head, None),
    };
    subject.chars().count() <= 72
        && TYPES.contains(&kind)
        && scope.is_none_or(|s| {
            s == OLD_SCOPE || BUILT_IN_SCOPES.contains(&s) || scopes.iter().any(|k| k == s)
        })
        && !description.is_empty()
        && !description.starts_with(char::is_uppercase)
        && !description.ends_with('.')
}

/// The `commit_scopes` of the configuration of a commit; none when it gives no list of strings (BRE-9).
fn commit_scopes(config: Option<&toml::Table>) -> Vec<String> {
    config
        .and_then(|c| ProjectTable::of(PathBuf::from(crate::CONFIG), c).ok())
        .and_then(|p| p.list("commit_scopes").ok())
        .unwrap_or_default()
}

/// Checks the non-merge commits of the local branches of `repo` against the messages rules, each against the session
/// of `records` (all of that repository) `session_of` ties it to (BRE-9): a commit the agent rebased keeps the date
/// the user wrote it, and stashes and notes are on no branch.
pub(super) fn check_commits(repo: &Path, records: &mut [&mut Record]) {
    let Some(log) = git(
        repo,
        &[
            "-c",
            "core.quotePath=false",
            "log",
            "--branches",
            "--no-merges",
            "--name-only",
            "--format=%x1e%h%x1f%H%x1f%aI%x1f%B%x1f",
        ],
    ) else {
        return;
    };
    for entry in log.split('\x1e') {
        let mut fields = entry.splitn(5, '\x1f');
        let (Some(hash), Some(full), Some(date), Some(message), Some(paths)) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            continue;
        };
        let Ok(at) = DateTime::parse_from_rfc3339(date) else {
            continue;
        };
        // the issues whose file it touches tie it to the session that claimed them (BRE-9)
        let stems: Vec<&str> = paths
            .lines()
            .filter_map(transcripts::issue_file_stem)
            .collect();
        let Some(k) = transcripts::session_of(at, &stems, records.iter().map(|r| &r.span)) else {
            continue;
        };
        let record = &mut *records[k];
        let verdict = &mut record.verdict;
        let config = commit_config(repo, full);
        // a project that turned the ban off at that commit has no attribution rule to breach (FEA-11)
        let off = config
            .as_ref()
            .is_some_and(|c| features::turns_off(c, features::NO_AI_ATTRIBUTION));
        if !off {
            verdict.applied.insert("commit-attribution");
            if attributed(message) {
                verdict.add("commit-attribution", false, "", hash.to_string());
            }
        }
        // nor a format rule when it turned the convention off (FEA-22)
        if config
            .as_ref()
            .is_some_and(|c| features::turns_off(c, features::CONVENTIONAL_COMMITS))
        {
            continue;
        }
        verdict.applied.insert("commit-format");
        if !conventional(
            message.lines().next().unwrap_or_default(),
            &commit_scopes(config.as_ref()),
        ) {
            verdict.add("commit-format", false, "", hash.to_string());
        }
    }
}
