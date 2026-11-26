//! The citations of the tests (`// spec: ID`, or the comment marker of the test file's language, TRC-12): the test
//! files of a root, whatever its layout (TRC-9 to TRC-11), the ids each cites and whether it gives the reason of an
//! internal proof (TRC-6 to TRC-8), the managed and internal test files (TRC-1, TRC-5, TRC-14). `check-specs`,
//! `spec-coverage` and `delivered-quality` read them.

use super::repo_files;
use crate::project_table::ProjectTable;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Every file under `dir` (outside a git repository, where no worktree can nest), but those of the directories
/// `skipped` says to leave; a symbolic link is never followed, as in `repo_files` (ADR 0009).
fn all_files(dir: &Path, skipped: &dyn Fn(&Path) -> bool, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() && !skipped(&path) {
            all_files(&path, skipped, found);
        } else if kind.is_file() {
            found.push(path);
        }
    }
}

/// The ids of a `// spec:` list and the reason it gives for an internal proof, `A-1, B-2 (internal: <reason>)`
/// (TRC-6, TRC-8): the reason never adds an id. The reason is the trimmed text after `(internal:`, its closing `)`
/// included.
pub(super) fn citation(list: &str) -> (impl Iterator<Item = &str>, Option<&str>) {
    let (ids, reason) = match list.split_once("(internal:") {
        Some((ids, reason)) => (ids, Some(reason.trim())),
        None => (list, None),
    };
    let ids = ids
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty());
    (ids, reason)
}

/// The line-comment markers a citation may open with (TRC-12).
const MARKERS: &[&str] = &["//", "#", "--", ";"];

/// The extension of `file`, lowercased.
fn extension(file: &Path) -> String {
    file.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// The markers of `file`, by its extension (TRC-12): the four for an extension the table does not know.
pub(super) fn markers(file: &Path) -> &'static [&'static str] {
    const ANY: &[&str] = MARKERS;
    match extension(file).as_str() {
        "rs" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "go" | "java" | "kt" | "kts"
        | "scala" | "swift" | "c" | "cc" | "cpp" | "h" | "hpp" | "cs" | "dart" | "php" => &ANY[..1],
        "py" | "sh" | "bash" | "zsh" | "bats" | "rb" | "pl" | "r" | "ex" | "exs" | "nix" => {
            &ANY[1..2]
        }
        "lua" | "sql" | "hs" | "elm" => &ANY[2..3],
        "clj" | "cljs" | "lisp" | "el" | "scm" | "rkt" => &ANY[3..],
        _ => ANY,
    }
}

/// The id list of `line` when it is a citation: its trimmed text opens with one of `markers` (`;` once or more),
/// then ` spec:` (TRC-12).
pub(super) fn citation_list<'a>(markers: &[&str], line: &'a str) -> Option<&'a str> {
    let line = line.trim();
    markers.iter().find_map(|m| {
        let rest = line.strip_prefix(m)?;
        let rest = if *m == ";" {
            rest.trim_start_matches(';')
        } else {
            rest
        };
        rest.strip_prefix(" spec:")
    })
}

/// Whether the file name `name` matches the glob `pattern`, `*` standing for any run of characters and every other
/// character for itself (TRC-11).
pub(super) fn name_matches(pattern: &str, name: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == name,
        Some((head, rest)) => name.strip_prefix(head).is_some_and(|name| {
            (0..=name.len())
                .filter(|&i| name.is_char_boundary(i))
                .any(|i| name_matches(rest, &name[i..]))
        }),
    }
}

/// The names of the list `key` of `.claude/cardinal.toml`, `default` when it is absent or empty, and the problems
/// of the key: a value that is not a list of strings (TRC-16), an entry holding a path separator (TRC-15). A
/// `cardinal.toml` that does not parse gives the default, its problem being reported with the `skipped_dirs`.
pub(super) fn config_names(root: &Path, key: &str, default: &str) -> (Vec<String>, Vec<String>) {
    let mut problems = Vec::new();
    let names = ProjectTable::read(root).map_or_default(|p| {
        p.list(key).unwrap_or_else(|e| {
            problems.push(e);
            Vec::new()
        })
    });
    let path = crate::config_of(root);
    for name in names.iter().filter(|n| n.contains(['/', '\\'])) {
        problems.push(format!(
            "{}: `{key}` entry `{name}` must be a name, not a path",
            path.display()
        ));
    }
    if names.is_empty() {
        return (vec![default.to_string()], problems);
    }
    (names, problems)
}

/// The file-name patterns of the test files, the `spec_files` of `.claude/cardinal.toml` or `*.rs` (TRC-11), with
/// the problems of the key.
pub(super) fn spec_patterns(root: &Path) -> (Vec<String>, Vec<String>) {
    config_names(root, "spec_files", "*.rs")
}

/// The directories of the internal tests, the `internal_dirs` of `.claude/cardinal.toml` or `src` (TRC-14), with the
/// problems of the key.
pub(super) fn internal_dirs(root: &Path) -> (Vec<String>, Vec<String>) {
    config_names(root, "internal_dirs", "src")
}

/// The test files of the root, whatever the layout of the project (TRC-9, TRC-11): those whose name matches a pattern
/// of `spec_patterns`, but those under `.git`, `node_modules`, a `target` beside a `Cargo.toml` (the build output) or
/// a directory named like one of the `skipped_dirs` of `.claude/cardinal.toml` (TRC-10); a failure of git, or a
/// `cardinal.toml` that cannot be read or parsed or whose `skipped_dirs` is not a list of strings, is a problem in
/// `out`.
pub(super) fn test_files(root: &Path, out: &mut Vec<String>) -> Vec<PathBuf> {
    let mut names = vec![".git".to_string(), "node_modules".to_string()];
    match ProjectTable::read(root).and_then(|p| p.list("skipped_dirs")) {
        Ok(dirs) => names.extend(dirs),
        Err(e) => out.push(e),
    }
    let patterns = spec_patterns(root).0;
    let skipped = |dir: &Path| {
        dir.file_name().is_some_and(|name| {
            names.iter().any(|n| name == n.as_str())
                || (name == "target" && dir.with_file_name("Cargo.toml").is_file())
        })
    };
    // in a git repository the files git lists, so never those of a nested worktree (ADR 0008) nor the ignored ones
    // every file, the patterns being matched below: a pathspec would read their other characters as globs
    let files: Vec<PathBuf> = match repo_files::repo_files(root, &[]) {
        Ok(Some(files)) => files
            .into_iter()
            .filter(|f| {
                let dirs = f.strip_prefix(root).unwrap_or(f).ancestors().skip(1);
                !dirs
                    .filter(|dir| !dir.as_os_str().is_empty())
                    .any(|dir| skipped(&root.join(dir)))
            })
            .collect(),
        Ok(None) => {
            let mut files = Vec::new();
            all_files(root, &skipped, &mut files);
            files
        }
        // never fall back to the disk, which holds the nested worktrees
        Err(e) => {
            out.push(e);
            Vec::new()
        }
    };
    files
        .into_iter()
        .filter(|f| {
            f.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| patterns.iter().any(|p| name_matches(p, n)))
        })
        .collect()
}

/// A file citing a requirement, and whether that citation gives the reason of an internal proof.
pub(super) struct Cite {
    pub(super) file: String,
    pub(super) reasoned: bool,
}

/// Ids cited by the `// spec: A-1, B-2` comments of `files`, with the files that cite them and whether the citation gives a
/// reason. A citation sits right above a test: only attributes and other comments may come between it and the `fn`;
/// any other place is a problem, and so is a reason that is blank, misplaced or unclosed (TRC-7).
pub(super) fn citations(
    root: &Path,
    files: &[PathBuf],
    out: &mut Vec<String>,
) -> BTreeMap<String, Vec<Cite>> {
    let mut cited: BTreeMap<String, Vec<Cite>> = BTreeMap::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let name = file
            .strip_prefix(root)
            .unwrap_or(file)
            .display()
            .to_string();
        let lines: Vec<&str> = text.lines().collect();
        let (markers, rust) = (markers(file), extension(file) == "rs");
        for (i, line) in lines.iter().enumerate() {
            let Some(list) = citation_list(markers, line) else {
                continue;
            };
            let next = lines[i + 1..].iter().map(|l| l.trim()).find(|l| {
                if rust {
                    !l.starts_with("#[") && !l.starts_with("//")
                } else {
                    !l.starts_with('@') && !MARKERS.iter().any(|m| l.starts_with(m))
                }
            });
            if rust && !next.is_some_and(|l| l.contains("fn ")) {
                out.push(format!(
                    "{name}:{}: `// spec:` must sit right above a test `fn`",
                    i + 1
                ));
            } else if !rust && next.is_none_or(str::is_empty) {
                out.push(format!(
                    "{name}:{}: a `spec:` citation must sit right above its test",
                    i + 1
                ));
            }
            let (ids, reason) = citation(list);
            let ids: Vec<&str> = ids.collect();
            let problem = reason.and_then(|r| match r.strip_suffix(')') {
                // a `)` of the reason closes one of its own `(`: an unmatched one ends the reason early
                Some(r) if !ids.is_empty() && r.matches('(').count() == r.matches(')').count() => {
                    r.trim().is_empty().then_some("gives no reason")
                }
                _ => Some("must follow the ids and end the line"),
            });
            if let Some(problem) = problem {
                out.push(format!(
                    "{name}:{}: `(internal: <reason>)` {problem}",
                    i + 1
                ));
            }
            let reasoned = reason.is_some() && problem.is_none();
            for id in ids {
                cited.entry(id.to_string()).or_default().push(Cite {
                    file: name.clone(),
                    reasoned,
                });
            }
        }
    }
    cited
}

/// Whether a test file is a managed one, under `.claude/`.
pub(super) fn is_managed(file: &str) -> bool {
    file.replace('\\', "/").starts_with(".claude/")
}

/// Whether a test file calls the code from inside: it sits under a directory named like one of `internal`, outside
/// the managed tests (TRC-1, TRC-5, TRC-14).
pub(super) fn is_internal(file: &str, internal: &[String]) -> bool {
    !is_managed(file)
        && Path::new(file)
            .components()
            .any(|c| internal.iter().any(|d| c.as_os_str() == d.as_str()))
}
