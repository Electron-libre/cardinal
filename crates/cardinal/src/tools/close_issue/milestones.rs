//! The milestones of the `roadmap` documents a close completes: the issues their `Issues:` list links and the `done`
//! set on their `State:` line once all are closed (CLS-8).

use super::config_or_exit;
use super::relink::{link_targets, resolve, RepoPath};
use crate::project_table::ProjectTable;
use crate::tools::roadmap;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// The issues the `Issues:` list `list` of a milestone of the roadmap document `path` (components from the root)
/// links, as components from the root: its link targets resolved against the directory of the document that fall
/// under the `issues` directory; a URL, an anchor or another document is no issue (CLS-8, RMP-2).
pub(in crate::tools) fn listed_issues(
    path: &[String],
    list: &str,
    issues: &[String],
) -> Vec<RepoPath> {
    let dir = &path[..path.len().saturating_sub(1)];
    // dedented, so an item indented by four spaces or more is no code block
    let list: String = list
        .lines()
        .map(|l| l.trim_start().to_string() + "\n")
        .collect();
    link_targets(&list)
        .into_iter()
        .filter_map(|span| {
            let target = &list[span];
            if target.starts_with(['/', '#']) || target.contains(':') {
                return None;
            }
            resolve(
                dir,
                &target[..target.find(['#', '?']).unwrap_or(target.len())],
            )
        })
        .filter(|p| p.starts_with(issues))
        .collect()
}

/// Whether the issue `path` is closed: a file directly under `done/` of the `issues` directory that `exists` (CLS-8,
/// RMP-2).
pub(in crate::tools) fn is_closed(
    issues: &[String],
    path: &[String],
    exists: impl Fn(&[String]) -> bool,
) -> bool {
    done_stem(issues, path).is_some() && exists(path)
}

/// The stem of `path` when it is a file directly under `done/` of the `issues` directory.
pub(super) fn done_stem(issues: &[String], path: &[String]) -> Option<String> {
    match path.strip_prefix(issues)? {
        [done, name] if done == "done" => name.strip_suffix(".md").map(str::to_string),
        _ => None,
    }
}

/// The roadmap `text` of the document `path` (components from the root) with `done` on the `State:` line of each
/// milestone (`roadmap`) that is `in issues`, whose `Issues:` list links one of the `closed` issues of
/// `issues/done/` and whose every link to the `issues` directory designates a file of `issues/done/` that `exists`
/// (CLS-8); and the names of
/// those milestones. The commit gate replays it (`review_gate`, REV-20).
pub(in crate::tools) fn milestones_done(
    path: &[String],
    text: &str,
    issues: &[String],
    closed: &HashSet<String>,
    exists: impl Fn(&[String]) -> bool,
) -> (String, Vec<String>) {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut set = Vec::new();
    for milestone in roadmap::milestones(&lines) {
        let listed = listed_issues(path, &milestone.issues(&lines), issues);
        let state = milestone.state_line(&lines, "in issues").filter(|_| {
            listed
                .iter()
                .any(|p| done_stem(issues, p).is_some_and(|s| closed.contains(&s)))
                && listed.iter().all(|p| is_closed(issues, p, &exists))
        });
        if let Some(at) = state {
            set.push((at, milestone.name));
        }
    }
    let mut out = String::with_capacity(text.len());
    for (i, l) in lines.iter().enumerate() {
        match (set.iter().any(|(at, _)| *at == i), l.split_once(':')) {
            (true, Some((key, value))) => {
                out.push_str(key);
                out.push_str(": done");
                out.push_str(&value[value.trim_end().len()..]);
            }
            _ => out.push_str(l),
        }
    }
    (out, set.into_iter().map(|(_, name)| name).collect())
}

/// The `roadmap` documents of the configuration of the project at `project` (components from the repository root
/// `root`), each as written in the configuration and as components from `root`, leaving out one outside the
/// repository; an error when the configuration cannot be read or its `roadmap` is not a list of strings.
pub(in crate::tools) fn roadmap_documents(
    root: &Path,
    project: &[String],
) -> Result<Vec<(String, RepoPath)>, String> {
    let project_root = project.iter().fold(root.to_path_buf(), |p, c| p.join(c));
    Ok(ProjectTable::read(&project_root)?
        .list("roadmap")?
        .into_iter()
        .filter_map(|document| {
            let path = resolve(project, &document)?;
            Some((document, path))
        })
        .collect())
}

/// Set `done` the milestones of the `roadmap` documents of the project of `issues` (relative to `root`) that the close
/// of `stem` completes, and print them (CLS-8); returns the documents written.
pub(super) fn set_milestones_done(root: &Path, issues: &[String], stem: &str) -> Vec<PathBuf> {
    let project = &issues[..issues.len().saturating_sub(1)];
    let closed = HashSet::from([stem.to_string()]);
    let file_of = |p: &[String]| p.iter().fold(root.to_path_buf(), |f, c| f.join(c));
    let mut written = Vec::new();
    for (document, path) in config_or_exit(roadmap_documents(root, project)) {
        let file = file_of(&path);
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let exists = |p: &[String]| file_of(p).is_file();
        let (updated, names) = milestones_done(&path, &text, issues, &closed, exists);
        if names.is_empty() {
            continue;
        }
        match std::fs::write(&file, updated) {
            Ok(()) => {
                for name in &names {
                    println!("milestone {name} done in {document}");
                }
                written.push(file);
            }
            Err(e) => eprintln!(
                "could not set the milestones done in {}: {e}",
                file.display()
            ),
        }
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: CLS-8 (internal: an `Issues:` list indented as a code block would be)
    #[test]
    fn a_deeply_indented_list_still_lists_its_issues() {
        let issues = vec!["issues".to_string()];
        let listed = listed_issues(&["R.md".to_string()], "      - [a](issues/a.md)\n", &issues);
        assert_eq!(listed, [vec!["issues".to_string(), "a.md".to_string()]]);
    }
}
