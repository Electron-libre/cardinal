//! The git and configuration helpers of the size tools (`deliverable-size`, `file-size`): git run with a copy of the
//! index that holds the untracked files, the empty tree, the files the counts leave out and the limits of
//! `.claude/cardinal.toml`.

use std::path::{Path, PathBuf};

/// The lock files, left out with `issues/` (SIZ-4), which counts when the issue files are off (FEA-37).
const LOCK_FILES: &[&str] = &[
    "*.lock",
    "package-lock.json",
    "npm-shrinkwrap.json",
    "pnpm-lock.yaml",
    "go.sum",
];

/// The id of the empty tree, the base of a branch with no commit yet.
pub(super) fn empty_tree(root: &Path) -> Result<String, String> {
    let id = git(
        root,
        &["hash-object", "-t", "tree", "--stdin"],
        None,
        Some(b""),
    )?;
    Ok(String::from_utf8_lossy(&id).trim().to_string())
}

/// The integer `key` of the `[project]` table of the configuration, at least `min`, `default` when the file or the
/// key is absent (SIZ-8, SIZ-10).
pub(super) fn limit(root: &Path, key: &str, default: u64, min: u64) -> Result<u64, String> {
    let project = crate::project_table::ProjectTable::read(root)?;
    let wrong = || {
        format!(
            "{key} of {} is not an integer of at least {min}",
            project.path().display()
        )
    };
    match project.int(key).map_err(|_| wrong())? {
        None => Ok(default),
        Some(n) => u64::try_from(n)
            .ok()
            .filter(|n| *n >= min)
            .ok_or_else(wrong),
    }
}

/// `git <args>` run on the working tree with the untracked files included as new ones: they are marked
/// intent-to-add in a copy of the index, removed after, so the index of the repository is untouched (SIZ-11).
pub(super) fn working_tree_git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let index = git(
        root,
        &["rev-parse", "--path-format=absolute", "--git-path", "index"],
        None,
        None,
    )?;
    let index = PathBuf::from(String::from_utf8_lossy(&index).trim());
    let copy = TempIndex(index.with_extension(format!("cardinal-size-{}", std::process::id())));
    if index.exists() {
        std::fs::copy(&index, &copy.0)
            .map_err(|e| format!("cannot copy {}: {e}", index.display()))?;
    }
    let untracked: Vec<u8> = git(
        root,
        &["ls-files", "-o", "--exclude-standard", "-z"],
        None,
        None,
    )?
    .split(|b| *b == 0)
    // a nested repository is listed as its directory: never counted, as `repo_files` drops it
    .filter(|p| !p.is_empty() && !p.ends_with(b"/"))
    .flat_map(|p| p.iter().copied().chain([0]))
    .collect();
    if !untracked.is_empty() {
        git(
            root,
            &[
                "--literal-pathspecs",
                "add",
                "--intent-to-add",
                "--pathspec-from-file=-",
                "--pathspec-file-nul",
            ],
            Some(&copy.0),
            Some(&untracked),
        )?;
    }
    git(root, args, Some(&copy.0), None)
}

/// A copy of the index, deleted when dropped.
struct TempIndex(PathBuf);

impl Drop for TempIndex {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Whether `path` is left out of the count: a lock file, or a file under `issues/` when `issue_files` is on.
pub(super) fn left_out(path: &str, issue_files: bool) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    issue_files && path.starts_with("issues/")
        || LOCK_FILES
            .iter()
            .any(|p| super::check_specs::name_matches(p, name))
}

/// The stdout of `git <args>` run in `root`, with `index` as its index and `stdin` as its input, or git's message.
pub(super) fn git(
    root: &Path,
    args: &[&str],
    index: Option<&Path>,
    stdin: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let mut command = crate::git::command(root);
    command.args(args);
    if let Some(index) = index {
        command.env("GIT_INDEX_FILE", index);
    }
    crate::git::exec(command, stdin).map_err(String::from)
}
