//! Validate `specs/*.md` against `specs/README.md` and trace them to the tests (`// spec: ID`, or the comment marker of
//! the test file's language, TRC-12).
//! Usage: `cardinal run check-specs` (from the repo root; an optional argument is another root; `--strict` also flags the
//! citations of managed tests (under `.claude/`) whose spec the project lacks: for the repository that ships them).
//! Prints one line per problem, exit 1 if any; a `[NEEDS CLARIFICATION: ...]` marker is a problem in an `accepted`
//! spec and a warning on stderr in a `draft` one; the vague wording of a `draft` requirement (`VAGUE`, not exactly one
//! `must`) is a warning, and so are the requirements proven only by tests under a `src` directory whose citations
//! give no reason, `// spec: ID (internal: <reason>)` (TRC-1, TRC-6). The happy paths that open `## Scenarios`,
//! `- **ID-H<n>** (happy; <ids>)`, are traced like requirements (TRC-17 to TRC-24); a spec without one is a warning
//! (CLR-9, CLR-10), and so is one whose `## Requirements` holds no `### Behaviour` (CLR-11, CLR-12, ADR 0029).

use super::repo_files;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const SECTIONS: [&str; 4] = [
    "## Intent",
    "## Requirements",
    "## Scenarios",
    "## Non-goals",
];

pub(super) struct Requirement {
    pub(super) id: String,
    /// Needs no test: withdrawn, or of the former `quality` kind.
    pub(super) exempt: bool,
    /// A happy path of `## Scenarios` (TRC-17), traced like a requirement.
    pub(super) happy: bool,
}

pub(super) struct Spec {
    file: String,
    id: String,
    pub(super) status: String,
    pub(super) requirements: Vec<Requirement>,
    /// Whether `## Requirements` holds a `### Behaviour` heading (CLR-11); `None` with no `## Requirements`, already a
    /// problem of its own.
    behaviour: Option<bool>,
}

fn is_id_prefix(s: &str) -> bool {
    s.len() == 3 && s.chars().all(|c| c.is_ascii_uppercase())
}

/// Whether `s` is a requirement id (`ABC-3`); `delivered-quality` reads the citations with it.
pub(super) fn is_req_id(s: &str) -> bool {
    s.split_once('-').is_some_and(|(p, n)| {
        is_id_prefix(p) && !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())
    })
}

const MARKER: &str = "[NEEDS CLARIFICATION:";

/// The text of a marker up to its closing bracket, brackets inside it included.
fn question(after: &str) -> &str {
    let mut depth = 1;
    for (i, c) in after.char_indices() {
        depth += match c {
            '[' => 1,
            ']' => -1,
            _ => 0,
        };
        if depth == 0 {
            return after[..i].trim();
        }
    }
    after.trim()
}

/// An open question left in the spec: a problem once `accepted`, a warning while `draft`. A marker cited in a code
/// span or a code block (fenced or indented) is an example, not a question; one in an HTML comment is still a question.
/// `prose` holds each line with its code spans blanked (`markdown::blank`), `None` for a line of a code block.
fn check_markers(
    file: &str,
    prose: &[Option<String>],
    accepted: bool,
    out: &mut Vec<String>,
    warn: &mut Vec<String>,
) {
    for (i, text) in prose.iter().enumerate() {
        let Some(text) = text else {
            continue;
        };
        let mut rest = text.as_str();
        while let Some(start) = rest.find(MARKER) {
            let after = &rest[start + MARKER.len()..];
            let question = question(after);
            if accepted {
                out.push(format!(
                    "{file}:{}: `{MARKER} ...]` left in an accepted spec: settle it",
                    i + 1
                ));
            } else {
                warn.push(format!(
                    "warning: {file}:{}: needs clarification: {question}",
                    i + 1
                ));
            }
            rest = after;
        }
    }
}

/// Words that leave the obligation of a requirement open (`etc` stands for `etc.`).
const VAGUE: [&str; 5] = ["should", "may", "etc", "appropriate", "fast"];

/// `text` without its double-quoted parts, straight or curly: a quoted message is not the requirement's wording. A
/// quote with no closing one (an inch mark) is kept, so the text after it is still checked.
fn without_quotes(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some((start, open)) = rest
        .char_indices()
        .find(|(_, c)| matches!(c, '"' | '\u{201c}'))
    {
        let after = &rest[start + open.len_utf8()..];
        let close = if open == '"' { '"' } else { '\u{201d}' };
        out.push_str(&rest[..start]);
        if let Some(end) = after.find(close) {
            rest = &after[end + close.len_utf8()..];
        } else {
            out.push(open);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// Warns of a `draft` requirement whose wording `statement`, its code spans blanked, is vague: a word of `VAGUE`, or not exactly one `must`. Warnings
/// only: the word list has false positives.
fn check_wording(file: &str, id: &str, statement: &str, warn: &mut Vec<String>) {
    // `shouldn't` holds `should` and `mustn't` a `must`
    let text = without_quotes(statement)
        .to_lowercase()
        .replace("n't", " not")
        .replace("n\u{2019}t", " not");
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let mut warned = BTreeSet::new();
    for word in words.iter().filter(|w| VAGUE.contains(w)) {
        if warned.insert(*word) {
            let shown = if *word == "etc" { "etc." } else { word };
            warn.push(format!(
                "warning: {file}: requirement {id}: vague word `{shown}`"
            ));
        }
    }
    let musts = words.iter().filter(|w| **w == "must").count();
    if musts != 1 {
        warn.push(format!(
            "warning: {file}: requirement {id}: holds {musts} `must`, expected 1"
        ));
    }
}

/// Record the problem `m` of the spec `file` in `out`.
fn problem(out: &mut Vec<String>, file: &str, m: impl std::fmt::Display) {
    out.push(format!("{file}: {m}"));
}

/// Whether `docs/adr` of `root` holds the ADR numbered `n`.
fn adr_exists(root: &Path, n: &str) -> bool {
    std::fs::read_dir(root.join("docs/adr"))
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{n}-"))
        })
}

fn parse(
    root: &Path,
    file: &str,
    text: &str,
    out: &mut Vec<String>,
    warn: &mut Vec<String>,
) -> Spec {
    let mut spec = Spec {
        file: file.to_string(),
        id: String::new(),
        status: String::new(),
        requirements: Vec::new(),
        behaviour: None,
    };
    let lines: Vec<&str> = text.lines().collect();
    let close = lines
        .iter()
        .skip(1)
        .position(|l| l.trim() == "---")
        .map(|p| p + 1);
    if lines.first().map(|l| l.trim()) != Some("---") || close.is_none() {
        problem(out, file, "no front matter (`---` ... `---`)");
        return spec;
    }
    let close = close.unwrap_or(0);
    for line in &lines[1..close] {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.split('#').next().unwrap_or("").trim();
        match key.trim() {
            "id" => spec.id = value.to_string(),
            "status" => spec.status = value.to_string(),
            "adrs" => {
                for n in value
                    .trim_matches(|c| c == '[' || c == ']')
                    .split(',')
                    .map(str::trim)
                    .filter(|n| !n.is_empty())
                {
                    if !adr_exists(root, n) {
                        problem(out, file, format_args!("adr {n} not found in docs/adr"));
                    }
                }
            }
            _ => {}
        }
    }
    if !is_id_prefix(&spec.id) {
        problem(
            out,
            file,
            format_args!(
                "front matter `id` must be three capital letters, got `{}`",
                spec.id
            ),
        );
    }
    // `implemented` was a status of its own, enforced like `accepted`: still read as such so older specs keep passing
    if spec.status == "implemented" {
        warn.push(format!(
            "warning: {file}: status `implemented` is deprecated, use `accepted`"
        ));
        spec.status = "accepted".to_string();
    }
    if !["draft", "accepted"].contains(&spec.status.as_str()) {
        problem(
            out,
            file,
            format_args!(
                "front matter `status` must be draft or accepted, got `{}`",
                spec.status
            ),
        );
    }
    let walk = crate::markdown::walk(&lines);
    let prose = walk.prose(&lines);
    for section in SECTIONS {
        if !lines
            .iter()
            .zip(&walk.lines)
            .any(|(l, read)| !read.hidden() && l.trim() == section)
        {
            problem(out, file, format_args!("missing section `{section}`"));
        }
    }
    // an open fence or comment hides every line after it: the requirements there would pass unread
    if walk.open_fence {
        problem(out, file, format_args!("unclosed fenced block"));
    }
    if walk.open_comment {
        problem(out, file, format_args!("unclosed HTML comment"));
    }
    let read = Read {
        lines: &lines,
        walk: &walk.lines,
        prose: &prose,
    };
    parse_requirements(file, &read, &mut spec, out, warn);
    if spec.status == "draft" && spec.behaviour == Some(false) {
        warn.push(format!(
            "warning: {file}: no behaviour (add `### Behaviour` under `## Requirements`)"
        ));
    }
    parse_happy_paths(file, &read, &mut spec, out, warn);
    check_markers(file, &prose, spec.status == "accepted", out, warn);
    spec
}

/// The lines of a spec as `markdown::walk` read them, each with its text with its code spans blanked (`check_markers`).
struct Read<'a> {
    lines: &'a [&'a str],
    walk: &'a [crate::markdown::Line],
    prose: &'a [Option<String>],
}

/// Whether `line` is the `### Behaviour` heading (or `### Behavior`), at the start of the line: an indented one is in a
/// list item or continues a paragraph (CLR-11).
fn is_behaviour_heading(line: &str) -> bool {
    line.strip_prefix("### ")
        .map(|t| t.trim().trim_end_matches('#').trim())
        .is_some_and(|t| t == "Behaviour" || t == "Behavior")
}

/// Read the requirement lines of the `## Requirements` section of `read`, the lines hidden in a code block or a comment
/// aside, into `spec`.
fn parse_requirements(
    file: &str,
    read: &Read,
    spec: &mut Spec,
    out: &mut Vec<String>,
    warn: &mut Vec<String>,
) {
    let mut seen = BTreeSet::new();
    let mut in_requirements = false;
    for (i, (&line, walked)) in read.lines.iter().zip(read.walk).enumerate() {
        if walked.hidden() {
            continue;
        }
        if line.starts_with("## ") {
            in_requirements = line.trim() == "## Requirements";
            if in_requirements {
                spec.behaviour.get_or_insert(false);
            }
        }
        if in_requirements && is_behaviour_heading(line) {
            spec.behaviour = Some(true);
        }
        if !in_requirements || !line.starts_with("- ") {
            continue;
        }
        let parsed = line
            .strip_prefix("- **")
            .and_then(|r| r.split_once("**"))
            .filter(|(id, _)| is_req_id(id));
        let Some((id, after)) = parsed else {
            problem(
                out,
                file,
                format_args!(
                    "requirement line must start with `- **{}-<n>**`: `{line}`",
                    spec.id
                ),
            );
            continue;
        };
        if !id.starts_with(&format!("{}-", spec.id)) {
            problem(
                out,
                file,
                format_args!(
                    "requirement {id} does not start with the spec id `{}`",
                    spec.id
                ),
            );
        }
        if !seen.insert(id.to_string()) {
            problem(out, file, format_args!("duplicate requirement {id}"));
        }
        // trimmed at the start only: `rest` stays a suffix of `line`, so `at` is its offset
        let kind = after
            .trim_start()
            .strip_prefix('(')
            .and_then(|a| a.split_once(')'));
        // the statement starts after the `(kind)`; with none, it was not found
        let (kind, at) = kind.map_or((String::new(), line.len()), |(k, rest)| {
            (k.to_string(), line.len() - rest.len())
        });
        let first = line[at..].trim();
        let withdrawn = first.starts_with("withdrawn:");
        // `quality` was a kind proved by a benchmark: still read, untraced, so older specs keep passing
        if kind == "quality" && !withdrawn {
            warn.push(format!("warning: {file}: requirement {id}: kind `quality` is deprecated, reword it for a test or withdraw it"));
        } else if kind != "deterministic" && kind != "quality" {
            problem(
                out,
                file,
                format_args!("requirement {id}: kind must be `(deterministic)`, got `({kind})`"),
            );
        }
        // with no `(kind)`, the statement was not found: the format problem is enough
        if spec.status == "draft" && !withdrawn && !kind.is_empty() {
            // the prose of the line has its bytes at the same offsets, its code spans blanked
            let first = read.prose[i].as_deref().map_or("", |p| p[at..].trim());
            check_wording(file, id, &with_continuation(first, read, i), warn);
        }
        spec.requirements.push(Requirement {
            id: id.to_string(),
            exempt: withdrawn || kind == "quality",
            happy: false,
        });
    }
}

/// The id and the requirement ids of a happy path item, `**<ID>-H<n>** (happy; <ids>) ...` with `<ID>` the `prefix`
/// (TRC-17), or `None` when `item`, the text after `- `, is not of that form.
fn happy_path<'a>(item: &'a str, prefix: &str) -> Option<(&'a str, Vec<&'a str>)> {
    if !is_id_prefix(prefix) {
        return None;
    }
    let (id, after) = item.strip_prefix("**")?.split_once("**")?;
    let n = id.strip_prefix(prefix)?.strip_prefix("-H")?;
    if n.is_empty() || !n.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let (inside, _) = after.trim_start().strip_prefix("(happy")?.split_once(')')?;
    let ids = match inside.strip_prefix(';') {
        Some(list) => list
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect(),
        None if inside.trim().is_empty() => Vec::new(),
        None => return None,
    };
    Some((id, ids))
}

/// Read the happy paths of the `## Scenarios` section of `read` into `spec`, after its requirements: their form
/// (TRC-17, TRC-19), the ids they name (TRC-18), their place before the other scenarios (TRC-20) and their ids (TRC-21);
/// a `draft` with none is warned of (CLR-9).
fn parse_happy_paths(
    file: &str,
    read: &Read,
    spec: &mut Spec,
    out: &mut Vec<String>,
    warn: &mut Vec<String>,
) {
    // the live requirements: a happy path walks through no withdrawn one
    let requirements: BTreeSet<String> = spec
        .requirements
        .iter()
        .filter(|r| !r.exempt)
        .map(|r| r.id.clone())
        .collect();
    let mut in_scenarios = false;
    let mut edge_case = false;
    for (i, (&line, walked)) in read.lines.iter().zip(read.walk).enumerate() {
        if walked.hidden() {
            continue;
        }
        if line.starts_with("## ") {
            in_scenarios = line.trim() == "## Scenarios";
        }
        let Some(item) = line.strip_prefix("- ").filter(|_| in_scenarios) else {
            continue;
        };
        let Some((id, ids)) = happy_path(item, &spec.id) else {
            // only the opening TRC-17 asks for: a scenario a consumer titled `Happy path (ABC-2):` keeps passing
            let after_id = item
                .strip_prefix("**")
                .and_then(|r| r.split_once("**"))
                .map_or(item, |(_, after)| after.trim_start());
            if after_id.starts_with("(happy") {
                out.push(format!(
                    "{file}:{}: a happy path must start with `- **{}-H<n>** (happy; <ids>)`",
                    i + 1,
                    spec.id
                ));
            } else {
                edge_case = true;
            }
            continue;
        };
        if edge_case {
            problem(
                out,
                file,
                format_args!("happy path {id} must come before the other scenarios"),
            );
        }
        if ids.is_empty() {
            problem(
                out,
                file,
                format_args!("happy path {id} names no requirement"),
            );
        }
        for unknown in ids.iter().filter(|r| !requirements.contains(**r)) {
            problem(
                out,
                file,
                format_args!(
                    "happy path {id} names {unknown}, which is not a live requirement of this spec"
                ),
            );
        }
        if spec.requirements.iter().any(|r| r.id == id) {
            problem(out, file, format_args!("duplicate happy path {id}"));
            continue;
        }
        spec.requirements.push(Requirement {
            id: id.to_string(),
            exempt: false,
            happy: true,
        });
    }
    if spec.status == "draft" && !spec.requirements.iter().any(|r| r.happy) {
        warn.push(format!(
            "warning: {file}: no happy path (open `## Scenarios` with `- **{}-H1** (happy; <ids>)`)",
            spec.id
        ));
    }
}

/// The statement `first` of the requirement of line `item` of `read` followed by its continuation lines: the lines
/// after it in the paragraph of the item line, as `CommonMark` reads it (CLR-8), up to a line hidden in a code block or
/// a comment, read with their code spans blanked.
fn with_continuation(first: &str, read: &Read, item: usize) -> String {
    let mut statement = first.to_string();
    let paragraph = read.walk[item].paragraph;
    for (walked, text) in read.walk.iter().zip(read.prose).skip(item + 1) {
        if walked.hidden() || paragraph.is_none() || walked.paragraph != paragraph {
            break;
        }
        statement.push(' ');
        statement.push_str(text.as_deref().unwrap_or_default().trim());
    }
    statement
}

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
    let path = crate::config_of(root);
    let table: toml::Table = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or_default();
    let value = table
        .get("project")
        .and_then(|p| p.get(key))
        .or_else(|| table.get(key));
    let mut problems = Vec::new();
    let names: Vec<String> = match value.map(|v| {
        v.as_array().map(|a| {
            a.iter()
                .map(toml::Value::as_str)
                .collect::<Option<Vec<_>>>()
        })
    }) {
        None => Vec::new(),
        Some(Some(Some(names))) => names.into_iter().map(str::to_string).collect(),
        Some(_) => {
            problems.push(format!(
                "{}: `{key}` must be a list of strings",
                path.display()
            ));
            Vec::new()
        }
    };
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
fn internal_dirs(root: &Path) -> (Vec<String>, Vec<String>) {
    config_names(root, "internal_dirs", "src")
}

/// The test files of the root, whatever the layout of the project (TRC-9, TRC-11): those whose name matches a pattern
/// of `spec_patterns`, but those under `.git`, `node_modules`, a `target` beside a `Cargo.toml` (the build output) or
/// a directory named like one of the `skipped_dirs` of `.claude/cardinal.toml` (TRC-10); a failure of git, or a
/// `cardinal.toml` that does not parse, is a problem in `out`.
pub(super) fn test_files(root: &Path, out: &mut Vec<String>) -> Vec<PathBuf> {
    let mut names = vec![".git".to_string(), "node_modules".to_string()];
    match super::config_list(root, "skipped_dirs") {
        Ok(dirs) => names.extend(dirs),
        // a project with no `cardinal.toml` skips nothing more
        Err(e) if crate::config_of(root).exists() => out.push(e),
        Err(_) => {}
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
    reasoned: bool,
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

/// The specs of `specs/`, by file name; their problems go to `out`, their warnings to `warn`.
pub(super) fn read_specs(root: &Path, out: &mut Vec<String>, warn: &mut Vec<String>) -> Vec<Spec> {
    let mut names: Vec<String> = std::fs::read_dir(root.join("specs"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| {
            Path::new(n).extension().is_some_and(|e| e == "md")
                && n != "README.md"
                && n != "TEMPLATE.md"
        })
        .collect();
    names.sort();
    let mut specs = Vec::new();
    for name in names {
        let file = format!("specs/{name}");
        let text = std::fs::read_to_string(root.join(&file)).unwrap_or_default();
        specs.push(parse(root, &file, &text, out, warn));
    }
    specs
}

/// The problems of the specs of `root` and of their citations, each a line; the warnings go to stderr.
pub(super) fn problems(root: &Path, strict: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut warnings = Vec::new();
    let specs = read_specs(root, &mut out, &mut warnings);
    for w in &warnings {
        eprintln!("{w}");
    }
    let mut ids: BTreeMap<&str, &str> = BTreeMap::new();
    for spec in specs.iter().filter(|s| !s.id.is_empty()) {
        if let Some(other) = ids.insert(&spec.id, &spec.file) {
            problem(
                &mut out,
                &spec.file,
                format_args!("spec id `{}` is already used by {other}", spec.id),
            );
        }
    }
    let files = test_files(root, &mut out);
    let cited = citations(root, &files, &mut out);
    let known: BTreeSet<&str> = specs
        .iter()
        .flat_map(|s| s.requirements.iter().map(|r| r.id.as_str()))
        .collect();
    for (id, cites) in cited.iter().filter(|(id, _)| !known.contains(id.as_str())) {
        // a managed test (under `.claude/`) may cite a spec that stays in the Cardinal repository
        let foreign = |file: &str| {
            !strict && is_managed(file) && !ids.contains_key(id.split('-').next().unwrap_or(""))
        };
        for file in cites.iter().map(|c| &c.file).filter(|f| !foreign(f)) {
            out.push(format!("{file}: cites `{id}`, which no spec defines"));
        }
    }
    for spec in specs.iter().filter(|s| s.status == "accepted") {
        for req in spec.requirements.iter().filter(|r| !r.exempt) {
            if !cited.contains_key(&req.id) {
                let what = if req.happy {
                    "happy path"
                } else {
                    "requirement"
                };
                problem(
                    &mut out,
                    &spec.file,
                    format_args!("{what} {}: no test cites `spec: {}`", req.id, req.id),
                );
            }
        }
    }
    let (internal, internal_problems) = internal_dirs(root);
    out.extend(spec_patterns(root).1);
    out.extend(internal_problems);
    boundary_warnings(&specs, &cited, &internal);
    // one line for every accepted spec: a nudge the specs written before happy paths existed all get (CLR-10)
    warn_accepted(&specs, "no happy path", |s| {
        s.requirements.iter().any(|r| r.happy)
    });
    // and before the behaviour was split from the contract (CLR-12)
    warn_accepted(&specs, "no behaviour", |s| s.behaviour != Some(false));
    out
}

/// Print one warning naming the `accepted` specs that lack `what`, `has` telling a spec that has it, if any.
fn warn_accepted(specs: &[Spec], what: &str, has: impl Fn(&Spec) -> bool) {
    let lacking: Vec<&str> = specs
        .iter()
        .filter(|s| s.status == "accepted" && !has(s))
        .map(|s| s.file.as_str())
        .collect();
    if !lacking.is_empty() {
        let plural = if lacking.len() == 1 {
            " holds"
        } else {
            "s hold"
        };
        eprintln!(
            "warning: {} accepted spec{plural} {what}: {}",
            lacking.len(),
            lacking.join(", ")
        );
    }
}

/// Whether a test file is a managed one, under `.claude/`.
fn is_managed(file: &str) -> bool {
    file.replace('\\', "/").starts_with(".claude/")
}

/// Whether a test file calls the code from inside: it sits under a directory named like one of `internal`, outside
/// the managed tests (TRC-1, TRC-5, TRC-14).
fn is_internal(file: &str, internal: &[String]) -> bool {
    !is_managed(file)
        && Path::new(file)
            .components()
            .any(|c| internal.iter().any(|d| c.as_os_str() == d.as_str()))
}

/// Warn, per spec, of the cited requirements whose every citation is internal, one at least giving no reason
/// (TRC-1 to TRC-4, TRC-6), the internal ones being under a directory named like one of `internal_dirs`.
fn boundary_warnings(
    specs: &[Spec],
    cited: &BTreeMap<String, Vec<Cite>>,
    internal_dirs: &[String],
) {
    let dirs = internal_dirs
        .iter()
        .map(|d| format!("`{d}`"))
        .collect::<Vec<_>>()
        .join(", ");
    for spec in specs {
        let internal: Vec<&str> = spec
            .requirements
            .iter()
            .filter(|r| !r.exempt)
            .filter(|r| {
                cited.get(&r.id).is_some_and(|cites| {
                    cites.iter().all(|c| is_internal(&c.file, internal_dirs))
                        && cites.iter().any(|c| !c.reasoned)
                })
            })
            .map(|r| r.id.as_str())
            .collect();
        if !internal.is_empty() {
            let plural = if internal.len() == 1 { "" } else { "s" };
            eprintln!(
                "warning: {}: {} requirement{plural} proven only by tests under {dirs} with no reason: {} \
                 (give it as `spec: ID (internal: <reason>)` after the comment marker)",
                spec.file,
                internal.len(),
                internal.join(", ")
            );
        }
    }
}

pub(super) fn main(args: &[String]) -> u8 {
    let args: Vec<String> = args.to_vec();
    if let Some(flag) = args.iter().find(|a| a.starts_with('-') && *a != "--strict") {
        eprintln!("unknown option `{flag}` (usage: cardinal run check-specs [--strict] [root])");
        std::process::exit(2);
    }
    if args.iter().filter(|a| *a != "--strict").count() > 1 {
        eprintln!("usage: cardinal run check-specs [--strict] [root]");
        std::process::exit(2);
    }
    let strict = args.iter().any(|a| a == "--strict");
    let root = PathBuf::from(
        args.iter()
            .find(|a| *a != "--strict")
            .map_or(".", String::as_str),
    );
    let found = problems(&root, strict);
    for p in &found {
        println!("{p}");
    }
    std::process::exit(i32::from(!found.is_empty()));
}
