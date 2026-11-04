//! The memory files a Claude Code session of a project loads at launch (`specs/cardinal-report.md`, REP-2), measured
//! in one way for a commit (`fixed-context`) and for the work tree (the budget, SYN-38).

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fs;
use std::path::{Component, Path};

/// The hops of imports Claude Code follows from a memory file.
const HOPS: usize = 4;

/// Where a directory keeps its memory files and rules.
pub(crate) struct Layout {
    claude: &'static [&'static str],
    rules: &'static str,
    /// The files read in place of the `CLAUDE.md` files when none exists, counted apart.
    agents: &'static [&'static str],
    /// The personal memory files, kept out of git: loaded at launch, listed by the audit, never measured (`measure`).
    local: &'static [&'static str],
}

/// A directory of a project.
pub(crate) const PROJECT: Layout = Layout {
    claude: &["CLAUDE.md", ".claude/CLAUDE.md"],
    rules: ".claude/rules",
    agents: &["AGENTS.md", ".claude/AGENTS.md"],
    local: &["CLAUDE.local.md"],
};

/// The configuration directory of the user (AUD-2).
pub(crate) const USER: Layout = Layout {
    claude: &["CLAUDE.md"],
    rules: "rules",
    agents: &[],
    local: &[],
};

/// A tree the memory files are read from; paths are relative and `/`-separated.
pub(crate) trait Tree {
    /// The path of the file `path` names, a symbolic link followed, and its content; `None` when absent.
    fn read(&self, path: &str) -> Option<(String, Vec<u8>)>;
    /// The paths of the files under the directory `dir`, at any depth.
    fn files(&self, dir: &str) -> Vec<String>;
}

/// The work tree at a path.
pub(crate) struct WorkTree<'a>(pub &'a Path);

impl Tree for WorkTree<'_> {
    fn read(&self, path: &str) -> Option<(String, Vec<u8>)> {
        let file = fs::canonicalize(self.0.join(path)).ok()?;
        let content = fs::read(&file).ok()?;
        let root = fs::canonicalize(self.0).ok()?;
        let file = file.strip_prefix(&root).unwrap_or(&file);
        Some((file.to_string_lossy().into_owned(), content))
    }

    fn files(&self, dir: &str) -> Vec<String> {
        let mut files = Vec::new();
        let mut dirs = vec![dir.to_string()];
        while let Some(dir) = dirs.pop() {
            for entry in fs::read_dir(self.0.join(&dir))
                .into_iter()
                .flatten()
                .flatten()
            {
                let path = format!("{dir}/{}", entry.file_name().to_string_lossy());
                // a linked directory is not walked: it could loop
                match entry.file_type() {
                    Ok(t) if t.is_dir() => dirs.push(path),
                    Ok(_) => files.push(path),
                    Err(_) => {}
                }
            }
        }
        files
    }
}

/// The bytes of the memory files loaded at launch: `AGENTS.md` and `.claude/AGENTS.md` apart from the others.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Memory {
    pub claude: u64,
    pub agents: u64,
}

/// Measures the memory files of `tree` (REP-2).
pub(crate) fn measure(tree: &impl Tree) -> Memory {
    let mut memory = Memory::default();
    // the personal memory files are never measured
    let layout = Layout {
        local: &[],
        ..PROJECT
    };
    for file in loaded(tree, "", &layout) {
        if file.agents {
            memory.agents += file.bytes;
        } else {
            memory.claude += file.bytes;
        }
    }
    memory
}

/// A memory file loaded at launch.
struct Loaded {
    /// Its path in the tree, a symbolic link followed (absolute when it points outside the tree).
    path: String,
    /// Whether it was loaded through the name `AGENTS.md` or `.claude/AGENTS.md`, whatever file that name points to.
    agents: bool,
    bytes: u64,
}

/// The memory files of the directory `dir` of `tree` (`""` for its root) loaded at launch (REP-2): each file once,
/// however many paths name it, the imports followed breadth first so that a file counts at its fewest hops.
fn loaded(tree: &impl Tree, dir: &str, layout: &Layout) -> Vec<Loaded> {
    let mut read: HashMap<String, Option<(String, Vec<u8>)>> = HashMap::new();
    let mut read = |path: &str| {
        read.entry(path.to_string())
            .or_insert_with(|| tree.read(path))
            .clone()
    };
    let agents = agents(dir, layout);
    let mut roots: Vec<String> = layout
        .claude
        .iter()
        .map(|p| under(dir, p))
        .filter(|p| read(p).is_some())
        .collect();
    if roots.is_empty() {
        roots.extend(agents.clone());
    }
    roots.extend(layout.local.iter().map(|p| under(dir, p)));
    roots.extend(
        rules(tree, dir, layout)
            .into_iter()
            .filter(|p| read(p).is_some_and(|(_, text)| !scoped(&String::from_utf8_lossy(&text)))),
    );
    let mut loaded = Vec::new();
    let mut seen = BTreeSet::new();
    let mut queue: VecDeque<(String, usize)> = roots.into_iter().map(|p| (p, 0)).collect();
    while let Some((path, hops)) = queue.pop_front() {
        let Some((file, text)) = read(&path) else {
            continue;
        };
        if !seen.insert(file.clone()) {
            continue;
        }
        if hops < HOPS {
            // against the directory of the file a symbolic link points to; outside the tree, `resolve` drops them
            let dir = Path::new(&file).parent().unwrap_or(Path::new(""));
            queue.extend(
                imports(&String::from_utf8_lossy(&text))
                    .iter()
                    .filter_map(|i| resolve(dir, i))
                    .map(|p| (p, hops + 1)),
            );
        }
        loaded.push(Loaded {
            path: file,
            agents: agents.contains(&path),
            bytes: text.len() as u64,
        });
    }
    loaded
}

/// The instruction files of the directory `dir` of `tree` the user wrote (AUD-1, AUD-2): the memory files loaded at
/// launch, `CLAUDE.local.md` and its imports included, plus `AGENTS.md`, `.claude/AGENTS.md` where the layout reads
/// them and the scoped rules, each once, sorted by path.
pub(crate) fn instruction_files(tree: &impl Tree, dir: &str, layout: &Layout) -> BTreeSet<String> {
    let mut files: BTreeSet<String> = loaded(tree, dir, layout)
        .into_iter()
        .map(|f| f.path)
        .collect();
    let extra = agents(dir, layout)
        .into_iter()
        .chain(rules(tree, dir, layout));
    files.extend(extra.filter_map(|p| tree.read(&p)).map(|(file, _)| file));
    files
}

/// The path `path` of the directory `dir` of a tree (`""` for its root).
fn under(dir: &str, path: &str) -> String {
    if dir.is_empty() {
        path.to_string()
    } else {
        format!("{dir}/{path}")
    }
}

/// The paths of the agents files of the layout in the directory `dir`.
fn agents(dir: &str, layout: &Layout) -> Vec<String> {
    layout.agents.iter().map(|p| under(dir, p)).collect()
}

/// The `**/*.md` files of the rules directory of `dir` in `tree`, sorted.
fn rules(tree: &impl Tree, dir: &str, layout: &Layout) -> Vec<String> {
    let mut rules: Vec<String> = tree
        .files(&under(dir, layout.rules))
        .into_iter()
        .filter(|p| Path::new(p).extension().is_some_and(|e| e == "md"))
        .collect();
    rules.sort();
    rules
}

/// Whether a rule's front matter, closed by its `---` line, has a top-level `paths` key: the rule loads on demand
/// only.
fn scoped(rule: &str) -> bool {
    let mut lines = rule.lines();
    if lines.next().map(str::trim) != Some("---") {
        return false;
    }
    let mut paths = false;
    for line in lines {
        if line.trim() == "---" {
            return paths;
        }
        paths |= line.split_once(':').is_some_and(|(key, _)| key == "paths");
    }
    false
}

/// The `@<path>` imports of a memory file: in its text as the `CommonMark` lexer reads it, outside every code span and
/// code block (indented or fenced, in a block quote or a list item too), as Claude Code skips its code tokens, and
/// outside HTML but what an HTML block that starts with a comment holds besides its comments.
fn imports(text: &str) -> Vec<String> {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd};
    let mut found = Vec::new();
    // the text read since the last block, inline tag, HTML or code span boundary, in its source form: Claude Code
    // scans each text token apart, so an `@` right after a code span or an emphasis starts an import
    let mut run = String::new();
    let mut code_block = false;
    // the lines of the HTML block being read: `marked` makes one token of a block, `pulldown-cmark` one event per line
    let mut html_block = String::new();
    for (event, range) in Parser::new(text).into_offset_iter() {
        match event {
            Event::Text(_) if !code_block => {
                // the range of an escaped character starts after its backslash, which keeps `\@` from importing
                let escaped = text[..range.start].ends_with('\\')
                    && text[range.start..].starts_with(|c: char| c.is_ascii_punctuation());
                run.push_str(&text[range.start - usize::from(escaped)..range.end]);
                continue;
            }
            Event::SoftBreak | Event::HardBreak => {
                run.push('\n');
                continue;
            }
            Event::Start(Tag::CodeBlock(_)) => code_block = true,
            Event::End(TagEnd::CodeBlock) => code_block = false,
            Event::Html(line) => html_block.push_str(&line),
            Event::End(TagEnd::HtmlBlock) => {
                html_imports(&std::mem::take(&mut html_block), &mut found);
            }
            _ => {}
        }
        run_imports(&run, &mut found);
        run.clear();
    }
    run_imports(&run, &mut found);
    found
}

/// Adds to `found` the imports of an HTML block as Claude Code reads its `html` token: none unless it starts with a
/// comment closed in it, else those of what is left once its `<!--...-->` comments are removed.
fn html_imports(block: &str, found: &mut Vec<String>) {
    let trimmed = block.trim_start();
    if !trimmed.starts_with("<!--") || !trimmed.contains("-->") {
        return;
    }
    // as Claude Code's `/<!--[\s\S]*?-->/g`: a close searched after the opening, an open comment left as text
    let mut rest = String::new();
    let mut tail = block;
    while let Some(start) = tail.find("<!--") {
        let Some(end) = tail[start + 4..].find("-->") else {
            break;
        };
        rest.push_str(&tail[..start]);
        tail = &tail[start + 4 + end + 3..];
    }
    rest.push_str(tail);
    run_imports(&rest, found);
}

/// Adds to `found` the imports of a run of text: an `@` at its start or after a white space, the path ending at the
/// first white space a backslash does not escape; a quoted path imports nothing.
fn run_imports(run: &str, found: &mut Vec<String>) {
    let mut chars = run.chars().peekable();
    let mut previous = None;
    while let Some(c) = chars.next() {
        let starts = c == '@' && previous.is_none_or(char::is_whitespace);
        previous = Some(c);
        if !starts {
            continue;
        }
        let mut path = String::new();
        while let Some(c) = chars.next_if(|c| !c.is_whitespace()) {
            if c == '\\' && chars.peek() == Some(&' ') {
                path.push(chars.next().unwrap_or(' '));
            } else {
                path.push(c);
            }
        }
        if !path.is_empty() && !path.starts_with(['"', '\'']) {
            found.push(path);
        }
    }
}

/// The path of an import (or of the target of a symbolic link) relative to the root of the tree; `None` for an
/// absolute path, a path from `~`, or one that leaves the tree.
pub(crate) fn resolve(dir: &Path, import: &str) -> Option<String> {
    if import.starts_with(['/', '~']) {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    for component in dir.join(import).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::imports;

    // spec: REP-2 (internal: the reader of one memory file; fixed_context.rs proves the counts through the binary)
    #[test]
    fn an_import_stands_outside_the_code_spans() {
        assert_eq!(imports("``a`b`` @x"), ["x"]);
        assert_eq!(imports("@a`b @c`"), ["a"]);
        assert_eq!(imports("x@y `@z` @my\\ file.md '@q'"), ["my file.md"]);
    }

    // spec: REP-2 (internal: the reader of one memory file; fixed_context.rs proves the counts through the binary)
    #[test]
    fn an_import_stands_outside_every_code_block() {
        assert_eq!(
            imports("Example:\n\n    @docs/big.md\n"),
            Vec::<String>::new()
        );
        assert_eq!(
            imports("> ```\n> @docs/big.md\n> ```\n"),
            Vec::<String>::new()
        );
        assert_eq!(
            imports("- item\n\n  ```\n  @docs/big.md\n  ```\n"),
            Vec::<String>::new()
        );
        assert_eq!(imports("Run `just\n@docs/x.md` now"), Vec::<String>::new());
        assert_eq!(imports("Run `x`@a.md and @b/*c*.md"), ["a.md", "b/"]);
        // an escape keeps its backslash; an HTML tag is a token of its own
        assert_eq!(imports("\\@a.md @b\\_c.md"), ["b\\_c.md"]);
        assert_eq!(imports("x <b> @a.md</b> <!-- @c.md -->"), ["a.md"]);
        // a paragraph continues on the next line, a quote and a list item hold text
        assert_eq!(
            imports("text\n    @a.md\n> @b.md\n- @c.md\n"),
            ["a.md", "b.md", "c.md"]
        );
    }

    // spec: REP-2 (internal: the reader of one memory file; fixed_context.rs proves the counts through the binary)
    #[test]
    fn an_import_stands_outside_html() {
        // a tag or comment inside a paragraph is a token of its own, the text between two tags another
        assert_eq!(
            imports("<!-- old: @docs/big.md -->\n\ntext <!-- @a.md --> <span>@b.md</span>\n"),
            ["b.md"]
        );
        assert_eq!(
            imports("<div>\n@docs/big.md\n</div>\n"),
            Vec::<String>::new()
        );
        // a block that starts with a whole comment is scanned once its comments are removed, over all its lines
        assert_eq!(
            imports("<!-- note\n--> @docs/rules.md\n"),
            ["docs/rules.md"]
        );
        assert_eq!(imports("> <!-- a -->\n> <!-- b -->@x.md\n"), ["x.md"]);
        assert_eq!(imports("<!-- open\n@docs/big.md\n"), Vec::<String>::new());
    }
}
