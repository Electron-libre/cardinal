//! Move a finished issue file into `issues/done/` (only run after the user agreed).
//! Uses `git mv` when the file is tracked, a plain rename otherwise. Refuses to overwrite.
//! Then rewrites every `issues/foo.md` mentioned in the other text files of the repo to
//! `issues/done/foo.md`, and every Markdown link that resolves to the issue from the directory of its file
//! (`../issues/foo.md` from `docs/`), so no document keeps pointing at the old path; the relative links of the moved
//! issue are rebased so they still resolve from `issues/done/`. The files are those git lists (tracked and
//! untracked, not ignored), so the issue worktrees nested under `.claude/worktrees/` keep their own files.
//! Runs in any worktree, but refuses to close an issue of another worktree named by path. Frees the claim of the
//! issue (`issue_claim.rs`): deleted on the trunk, `merge pending` on another branch until the merge brings the
//! close to the trunk. Sets `done` the milestones of the roadmap whose last open issue it closed (CLS-8). Ends on the
//! issue `next-issue` then offers. Refuses, before writing any file, a `.claude/cardinal.toml` it cannot read (CLS-10).
//! Usage: `cardinal run close-issue issues/foo.md`; `cardinal run close-issue --relink` fixes the references to
//! every issue already in `issues/done/` (run it once to repair old dangling paths).

mod relink;

use super::{issue_claim, repo_files, roadmap};
use crate::project_table::ProjectTable;
pub(super) use relink::{components, relink_links, resolve, rewritten};
use relink::{link_targets, RepoPath};

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const SKIPPED_DIRS: [&str; 3] = [".git", "target", "node_modules"];

fn is_text_file(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n == "justfile")
        || path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| ["md", "rs", "toml", "yaml", "yml", "txt", "json"].contains(&e))
}

/// The names of the directories whose files `close-issue` never rewrites in the repository at `root`.
fn skipped_dirs(root: &Path) -> Result<Vec<String>, String> {
    let mut skipped: Vec<String> = SKIPPED_DIRS.iter().map(|s| (*s).to_string()).collect();
    skipped.extend(ProjectTable::read(root)?.list("skipped_dirs")?);
    Ok(skipped)
}

/// The value of `read`, or its error on stderr and exit 1: `close-issue` refuses a broken configuration (CLS-10).
fn config_or_exit<T>(read: Result<T, String>) -> T {
    read.unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    })
}

/// Exits 1, before any file is written, when the configuration of the repository at `root` or of the project of the
/// `issues` directory (relative to `root`) cannot be read or gives a key of another type (CLS-10).
fn check_config(root: &Path, issues: &[String]) {
    config_or_exit(skipped_dirs(root));
    config_or_exit(roadmap_documents(
        root,
        &issues[..issues.len().saturating_sub(1)],
    ));
}

/// Whether `skipped` names a directory of `path` (relative to the root).
fn in_skipped_dir(path: &Path, skipped: &[String]) -> bool {
    path.parent()
        .into_iter()
        .flat_map(Path::components)
        .any(|c| skipped.iter().any(|s| c.as_os_str() == s.as_str()))
}

/// Whether `close-issue` rewrites the references of the file `path` (relative to the repository at `root`): the commit
/// gate accepts a rewrite only where the tool makes one (`review_gate`, REV-20); none under a broken configuration,
/// which `close-issue` refuses (CLS-10).
pub(super) fn rewrites(root: &Path, path: &Path) -> bool {
    is_text_file(path) && skipped_dirs(root).is_ok_and(|skipped| !in_skipped_dir(path, &skipped))
}

/// The text files of the work tree at `root`, as git lists them (so never those of a nested worktree, ADR 0008),
/// except under a directory named like one of `skipped`. Exits with an error when git cannot list them, since the
/// references would stay dangling.
fn text_files(root: &Path, skipped: &[String]) -> Vec<std::path::PathBuf> {
    let skipped_dir =
        |path: &Path| in_skipped_dir(path.strip_prefix(root).unwrap_or(path), skipped);
    let files = match repo_files::repo_files(root, &[]) {
        Ok(Some(files)) => files,
        Ok(None) => vec![],
        Err(e) => {
            eprintln!(
                "{e}\nreferences were not updated: fix git, then run `cardinal run close-issue --relink`"
            );
            std::process::exit(1);
        }
    };
    files
        .into_iter()
        .filter(|p| is_text_file(p) && !skipped_dir(p))
        .collect()
}

/// links, as components from the root: its link targets resolved against the directory of the document that fall
/// under the `issues` directory; a URL, an anchor or another document is no issue (CLS-8, RMP-2).
pub(super) fn listed_issues(path: &[String], list: &str, issues: &[String]) -> Vec<RepoPath> {
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
pub(super) fn is_closed(
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
pub(super) fn milestones_done(
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
pub(super) fn roadmap_documents(
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
fn set_milestones_done(root: &Path, issues: &[String], stem: &str) -> Vec<PathBuf> {
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

/// Rewrite the references to the given closed issues of the `issues/` directory `issues` (relative to `root`) in
/// every text file under `root`; `moved` is the stem of the issue just moved, whose own links are rebased. Returns the
/// files written.
fn relink(
    root: &Path,
    issues: &[String],
    stems: &HashSet<String>,
    moved: Option<&str>,
) -> Vec<PathBuf> {
    let skipped = config_or_exit(skipped_dirs(root));
    let moved = moved.map(|stem| {
        let mut path = issues.to_vec();
        path.extend(["done".to_string(), format!("{stem}.md")]);
        path
    });
    let mut written = Vec::new();
    for file in text_files(root, &skipped) {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let path = components(file.strip_prefix(root).unwrap_or(&file));
        let updated = rewritten(&path, &text, issues, stems, moved.as_ref() == Some(&path));
        if updated == text {
            continue;
        }
        match std::fs::write(&file, updated) {
            Ok(()) => {
                println!("updated references in {}", file.display());
                written.push(file);
            }
            Err(e) => eprintln!("could not update {}: {e}", file.display()),
        }
    }
    written
}

/// `git --literal-pathspecs args` run in `root`, its output `None` when it fails: a file name is never a glob.
fn git_literal(root: &Path, args: &[&OsStr]) -> Option<Vec<u8>> {
    let mut cmd = crate::git::command(root);
    cmd.arg("--literal-pathspecs").args(args);
    crate::git::exec(cmd, None).ok()
}

/// The paths of the NUL-separated output of git run in `root`, canonical (relative to the top of the work tree).
fn paths_in(root: &Path, out: &[u8]) -> HashSet<PathBuf> {
    out.split(|b| *b == 0)
        .filter(|name| !name.is_empty())
        .map(|name| canonical(&root.join(repo_files::path_of(name))))
        .collect()
}

fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The index of the repository at `root` before a close: the files that differ from `HEAD` in the index or the work
/// tree (`None` when git cannot tell), and those already staged.
struct Before {
    changed: Option<HashSet<PathBuf>>,
    staged: HashSet<PathBuf>,
}

impl Before {
    fn read(root: &Path) -> Self {
        let diff = |args: &[&str]| {
            let args: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
            git_literal(root, &args).map(|out| paths_in(root, &out))
        };
        Before {
            changed: diff(&["diff", "HEAD", "--name-only", "-z"]),
            staged: diff(&["diff", "--cached", "--name-only", "-z"]).unwrap_or_default(),
        }
    }
}

/// Stage the close of the tracked issue `old` (relative to `root`), from `issue` to `moved`: the issue whole, and
/// each tracked file of `written` that `before` shows unchanged, naming the others and what was staged before
/// (CLS-9). `false` when git refuses to stage it.
fn stage_close(
    root: &Path,
    (old, issue, moved): (&str, &Path, &Path),
    written: &[PathBuf],
    before: &Before,
) -> bool {
    let root = &canonical(root);
    let moved = canonical(moved);
    let written: std::collections::BTreeSet<PathBuf> = written
        .iter()
        .map(|f| canonical(f))
        .filter(|f| *f != moved)
        .collect();
    let mut ls: Vec<&OsStr> = vec![OsStr::new("ls-files"), OsStr::new("-z"), OsStr::new("--")];
    ls.extend(written.iter().map(|f| f.as_os_str()));
    let tracked = if written.is_empty() {
        HashSet::new()
    } else {
        git_literal(root, &ls).map_or_else(HashSet::new, |out| paths_in(root, &out))
    };
    let shown = |file: &Path| {
        file.strip_prefix(root)
            .unwrap_or(file)
            .display()
            .to_string()
    };
    let mut stage = vec![moved.clone()];
    for file in written.into_iter().filter(|f| tracked.contains(f)) {
        if before
            .changed
            .as_ref()
            .is_none_or(|changed| changed.contains(&file))
        {
            println!("not staged: {} (it held changes of yours)", shown(&file));
        } else {
            stage.push(file);
        }
    }
    let mut add: Vec<&OsStr> = vec![OsStr::new("add"), OsStr::new("--")];
    add.extend(stage.iter().map(|f| f.as_os_str()));
    if git_literal(root, &add).is_none() {
        eprintln!(
            "could not stage the close: stage {} and its rewrites before the commit",
            shown(&moved)
        );
        return false;
    }
    let issue = canonical(issue);
    let mut others: Vec<String> = before
        .staged
        .iter()
        .filter(|f| **f != issue)
        .map(|f| shown(f))
        .collect();
    others.sort();
    for file in others {
        println!("staged before the close: {file} (a bare `git commit` takes it too)");
    }
    println!("staged: commit the close with `git commit` (no `git add` of {old}: git mv staged its removal)");
    true
}

/// The directory `dir` relative to `root`, as components. Exits with an error when it is not under `root`, since
/// the links could not be resolved.
fn under(root: &Path, dir: &Path) -> RepoPath {
    let relative = std::fs::canonicalize(root).ok().and_then(|root| {
        let dir = std::fs::canonicalize(dir).ok()?;
        Some(components(dir.strip_prefix(root).ok()?))
    });
    relative.unwrap_or_else(|| {
        eprintln!(
            "{} is not under {}: references were not updated, run `cardinal run close-issue --relink` once fixed",
            dir.display(),
            root.display()
        );
        std::process::exit(1);
    })
}

/// Root of the git repository containing `dir`; the references are only rewritten inside it.
fn repo_root(dir: &Path) -> Option<std::path::PathBuf> {
    crate::git::toplevel(dir)
}

/// The git common directory of the repository of `dir` and the top of its worktree, canonical.
fn worktree_of(dir: &Path) -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    let common = crate::git::run(
        dir,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .ok()?;
    let common = std::fs::canonicalize(common).ok()?;
    Some((common, std::fs::canonicalize(repo_root(dir)?).ok()?))
}

/// Exit with an error when the issues `dir` belong to another worktree of the repository of the current directory
/// (PAR-18).
fn refuse_other_worktree(dir: &Path) {
    let (Some(here), Some((common, there))) = (worktree_of(Path::new(".")), worktree_of(dir))
    else {
        return;
    };
    if here.0 == common && here.1 != there {
        eprintln!(
            "{} is an issue of another worktree ({}): close it there",
            dir.display(),
            there.display()
        );
        std::process::exit(1);
    }
}

/// Free the claim of `stem` made in the worktree of `dir` (CLM-7): delete it on the trunk, else mark it `merge
/// pending` on the current branch; the claim of another existing worktree is left to it.
fn free_claim(dir: &Path, stem: &str) {
    let Some((old, mut claim)) = issue_claim::read(dir, stem) else {
        return;
    };
    // a claim whose worktree was removed is freed by whoever closes the issue
    let theirs = std::fs::canonicalize(&claim.worktree).ok();
    let here = repo_root(dir).and_then(|r| std::fs::canonicalize(r).ok());
    if theirs.is_some() && theirs != here {
        eprintln!(
            "the claim of {stem} belongs to another worktree ({}): it was left as is",
            claim.worktree
        );
        return;
    }
    let branch = issue_claim::current_branch(dir);
    if issue_claim::trunk(dir).is_some_and(|t| t == branch) {
        if issue_claim::delete(dir, stem, &old) {
            println!("claim of {stem} released");
        }
        return;
    }
    claim.state = "merge pending".into();
    claim.branch = branch;
    if issue_claim::write(dir, stem, &claim, Some(&old)) {
        println!(
            "claim of {stem}: merge pending ({}) until the trunk holds the close",
            claim.branch
        );
    } else {
        eprintln!("the claim of {stem} changed during the close: it was left as is");
    }
}

/// `--relink`: rewrite the references to every issue of `issues/done`.
fn relink_closed() -> u8 {
    let stems: HashSet<String> = std::fs::read_dir("issues/done")
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .filter(|s| {
            s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        })
        .collect();
    let root = repo_root(Path::new("."));
    if let Some(root) = root.as_deref().filter(|_| Path::new("issues").is_dir()) {
        check_config(root, &under(root, Path::new("issues")));
    }
    if stems.is_empty() {
        return 0;
    }
    let Some(root) = root else {
        eprintln!("not inside a git repository");
        std::process::exit(1);
    };
    relink(&root, &under(&root, Path::new("issues")), &stems, None);
    0
}

pub(super) fn main(args: &[String]) -> u8 {
    let Some(arg) = args.first().cloned() else {
        eprintln!("usage: cardinal run close-issue issues/<name>.md | --relink");
        std::process::exit(2);
    };
    if arg == "--relink" {
        return relink_closed();
    }
    let issue = Path::new(&arg);
    let (Some(dir), Some(name)) = (issue.parent(), issue.file_name()) else {
        eprintln!("not an issue file: {arg}");
        std::process::exit(2);
    };
    if !issue.is_file() || dir.file_name().is_none_or(|d| d != "issues") {
        eprintln!("{arg} is not an issue file directly under an `issues/` directory");
        std::process::exit(1);
    }
    refuse_other_worktree(dir);
    if let Some(root) = repo_root(dir) {
        check_config(&root, &under(&root, dir));
    }
    let done = dir.join("done");
    let target = done.join(name);
    if target.exists() {
        eprintln!("{} already exists", target.display());
        std::process::exit(1);
    }
    std::fs::create_dir_all(&done).expect("done dir is creatable");
    // Run git in the issue's directory with paths relative to it, so the caller's directory does not matter.
    let git = |args: &[&OsStr]| {
        let mut cmd = crate::git::command(dir);
        cmd.args(args);
        crate::git::exec(cmd, None)
    };
    let tracked = git(&[
        "ls-files".as_ref(),
        "--error-unmatch".as_ref(),
        "--".as_ref(),
        name,
    ])
    .is_ok();
    let issue_path = canonical(issue);
    let before = tracked
        .then(|| repo_root(dir).map(|root| Before::read(&root)))
        .flatten();
    let moved = if tracked {
        let into = Path::new("done").join(name);
        git(&["mv".as_ref(), name, into.as_os_str()])
            .inspect_err(|failure| eprintln!("{failure}"))
            .is_ok()
    } else {
        std::fs::rename(issue, &target).is_ok()
    };
    if !moved {
        eprintln!("could not move {arg}");
        std::process::exit(1);
    }
    let how = if tracked {
        "git mv"
    } else {
        "plain rename, not tracked by git"
    };
    println!("moved {} -> {} ({how})", issue.display(), target.display());
    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let mut staged = true;
    match repo_root(dir) {
        Some(root) => {
            let issues = under(&root, dir);
            let mut written = relink(
                &root,
                &issues,
                &HashSet::from([stem.to_string()]),
                Some(stem),
            );
            written.extend(set_milestones_done(&root, &issues, stem));
            if let Some(before) = &before {
                let old = format!("{}/{}", issues.join("/"), name.display());
                if !stage_close(&root, (&old, &issue_path, &target), &written, before) {
                    staged = false;
                }
            }
        }
        None => eprintln!("not inside a git repository: references to {stem} were not updated"),
    }
    free_claim(dir, stem);
    println!("{}", super::next_issue::next(dir));
    u8::from(!staged)
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
