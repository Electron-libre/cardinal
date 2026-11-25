//! The rewriting of the references to a closed issue in the text of a file (CLS-1, CLS-2, CLS-4): its plain
//! `issues/<stem>.md` mentions and the Markdown links that resolve to it, as `CommonMark` reads them.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::Path;

/// Insert `done/` in `issues/<stem>.md` for the given stems, only where `issues/` starts a path
/// (not `my_issues/` nor `docs/issues/`).
fn relink_text(text: &str, stems: &HashSet<String>) -> String {
    const MARK: &str = "issues/";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(MARK) {
        out.push_str(&rest[..i + MARK.len()]);
        let boundary = out[..out.len() - MARK.len()]
            .chars()
            .next_back()
            .is_none_or(|c| c.is_whitespace() || "`\"'([<{=,:".contains(c));
        rest = &rest[i + MARK.len()..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if boundary && rest[name.len()..].starts_with(".md") && stems.contains(&name) {
            out.push_str("done/");
        }
    }
    out.push_str(rest);
    out
}

/// A path relative to the repository root, as its components.
pub(in crate::tools) type RepoPath = Vec<String>;

pub(super) fn components(path: &Path) -> RepoPath {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect()
}

/// `target` (a `/`-separated relative path) resolved against the directory `dir`; `None` when it climbs above the root.
pub(in crate::tools) fn resolve(dir: &[String], target: &str) -> Option<RepoPath> {
    let mut parts = dir.to_vec();
    for part in target.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            part => parts.push(part.to_string()),
        }
    }
    Some(parts)
}

/// The relative path from the directory `from` to `to`; `.` when they are the same.
fn relative(from: &[String], to: &[String]) -> String {
    let common = from.iter().zip(to).take_while(|(a, b)| a == b).count();
    let mut parts = vec![".."; from.len() - common];
    parts.extend(to[common..].iter().map(String::as_str));
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

/// The new target of a link written from `old_dir` and read from `new_dir`, when the file it designates moved
/// (`moves`) or the file holding the link did; `None` when it stays as written (URLs, absolute paths and anchors
/// alone included).
fn relink_target(
    target: &str,
    old_dir: &[String],
    new_dir: &[String],
    moves: &HashMap<RepoPath, RepoPath>,
) -> Option<String> {
    if target.starts_with(['/', '#']) || target.contains(':') {
        return None;
    }
    let (path, anchor) = target.split_at(target.find(['#', '?']).unwrap_or(target.len()));
    if path.is_empty() {
        return None;
    }
    let Some(resolved) = resolve(old_dir, path) else {
        // above the root: only the move of the file holding the link changes it, one `../` per level gone down
        let deeper = new_dir.strip_prefix(old_dir).filter(|d| !d.is_empty())?;
        return Some("../".repeat(deeper.len()) + target);
    };
    let dest = moves.get(&resolved).unwrap_or(&resolved);
    let slash = if path.ends_with('/') { "/" } else { "" };
    (dest != &resolved || old_dir != new_dir).then(|| relative(new_dir, dest) + slash + anchor)
}

/// The range of the link destination starting at `start` of `text`, after the blanks before it and the quote
/// markers of a line it moved to: inside its `<...>` when it has them, else up to an ASCII blank, a control
/// character or the `)` that closes the link (balanced parentheses inside), `\` escapes read, as `CommonMark` reads a
/// destination; empty when there is none.
fn destination(text: &str, start: usize) -> Range<usize> {
    let mut start = start;
    let mut new_line = false;
    while let Some(c) = text[start..].chars().next() {
        if !(c.is_ascii_whitespace() || (new_line && c == '>')) {
            break;
        }
        new_line |= c == '\n';
        start += 1;
    }
    let rest = &text[start..];
    let bracketed = rest.starts_with('<');
    let (mut depth, mut escaped) = (0usize, false);
    for (i, c) in rest.char_indices().skip(usize::from(bracketed)) {
        match c {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            '>' if bracketed => return start + 1..start + i,
            '\n' | '<' if bracketed => break,
            _ if bracketed => {}
            '(' => depth += 1,
            ')' if depth == 0 => return start..start + i,
            ')' => depth -= 1,
            c if c == ' ' || c.is_control() => return start..start + i,
            _ => {}
        }
    }
    if bracketed {
        start..start
    } else {
        start..text.len()
    }
}

/// The range of the destination of the reference definition `[label]: target` that `def` spans in `text`, after
/// the markers of the quotes its target line may hold; `None` for a footnote (`[^1]: text`).
fn definition_target(text: &str, def: Range<usize>) -> Option<Range<usize>> {
    let open = def.start + text[def.clone()].find('[')?;
    let mut escaped = false;
    let close = text[open + 1..def.end].char_indices().find_map(|(i, c)| {
        let end = !escaped && c == ']';
        escaped = !escaped && c == '\\';
        end.then_some(open + 1 + i)
    })?;
    if text[open + 1..].starts_with('^') || !text[close + 1..].starts_with(':') {
        return None;
    }
    Some(destination(text, close + 2)).filter(|t| !t.is_empty() && t.end <= def.end)
}

/// Byte ranges of the link targets of a Markdown text as `CommonMark` reads it (`pulldown-cmark`): the destinations
/// of its inline links and images `[text](target)` and of its reference definitions `[label]: target`, so none of a
/// code span, a code block, an HTML comment or block, an autolink or escaped brackets, nor a footnote (CLS-2, CLS-6).
pub(super) fn link_targets(text: &str) -> Vec<Range<usize>> {
    use pulldown_cmark::{Event, LinkType, Parser, Tag, TagEnd};
    let parser = Parser::new(text);
    let mut spans: Vec<Range<usize>> = parser
        .reference_definitions()
        .iter()
        .filter_map(|(_, def)| definition_target(text, def.span.clone()))
        .collect();
    // the links and images being read, inline or not, and the end of the text of each read so far
    let mut open: Vec<(bool, usize)> = vec![];
    for (event, range) in parser.into_offset_iter() {
        let closed = match &event {
            Event::End(TagEnd::Link | TagEnd::Image) => open.pop(),
            _ => None,
        };
        for (_, text_end) in &mut open {
            *text_end = (*text_end).max(range.end);
        }
        match event {
            Event::Start(Tag::Link { link_type, .. } | Tag::Image { link_type, .. }) => {
                open.push((link_type == LinkType::Inline, range.start + 1));
            }
            _ => {
                if let Some((true, text_end)) = closed {
                    let target = text[text_end..range.end]
                        .find("](")
                        .map(|i| destination(text, text_end + i + 2));
                    spans.extend(target.filter(|t| !t.is_empty()));
                }
            }
        }
    }
    spans.sort_by_key(|s| s.start);
    spans
}

/// Rewrite the Markdown link targets of `text`, written from `old_dir` and read from `new_dir` (CLS-2, CLS-4).
pub(in crate::tools) fn relink_links(
    text: &str,
    old_dir: &[String],
    new_dir: &[String],
    moves: &HashMap<RepoPath, RepoPath>,
) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for span in link_targets(text) {
        if span.start < last {
            continue;
        }
        if let Some(target) = relink_target(&text[span.clone()], old_dir, new_dir, moves) {
            out.push_str(&text[last..span.start]);
            out.push_str(&target);
            last = span.end;
        }
    }
    out.push_str(&text[last..]);
    out
}

/// The text of the file `path` (components from the root) once the references to the closed issues `stems` of the
/// `issues/` directory `issues` are rewritten (CLS-1, CLS-2); `moved` when it is the issue just moved to
/// `issues/done/`, whose own links are rebased (CLS-4). The commit gate replays it (`review_gate`, REV-20).
pub(in crate::tools) fn rewritten(
    path: &[String],
    text: &str,
    issues: &[String],
    stems: &HashSet<String>,
    moved: bool,
) -> String {
    if !moved && !stems.iter().any(|stem| text.contains(stem.as_str())) {
        return text.to_string();
    }
    let in_issues = |parts: &[&str]| -> RepoPath {
        let mut path = issues.to_vec();
        path.extend(parts.iter().map(|p| (*p).to_string()));
        path
    };
    let relocations: HashMap<RepoPath, RepoPath> = stems
        .iter()
        .map(|stem| {
            let name = format!("{stem}.md");
            (in_issues(&[&name]), in_issues(&["done", &name]))
        })
        .collect();
    let dir = &path[..path.len().saturating_sub(1)];
    if path
        .last()
        .is_some_and(|name| Path::new(name).extension().is_some_and(|e| e == "md"))
    {
        let old_dir = if moved { issues } else { dir };
        relink_text(&relink_links(text, old_dir, dir, &relocations), stems)
    } else {
        relink_text(text, stems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(text: &str) -> Vec<&str> {
        link_targets(text).into_iter().map(|r| &text[r]).collect()
    }

    // spec: CLS-2 (internal: the shapes of a destination the parser hands over by its offsets)
    #[test]
    fn reads_the_destinations_as_commonmark_does() {
        assert_eq!(
            targets("[a `](x)`](a.md \"t\") ![i](i.png) [![j](j.png)](b(1).md) [e](<c d.md>)\n"),
            ["a.md", "i.png", "j.png", "b(1).md", "c d.md"]
        );
        assert_eq!(targets("> [z]:\n> q.md\n\n[^1]: n.md\n"), ["q.md"]);
        assert_eq!(targets("> [a](\n> f.md)\n"), ["f.md"]);
        assert_eq!(
            targets("[x]: <a\\>b.md>\n\n[y](b\u{a0}c.md)\n"),
            ["a\\>b.md", "b\u{a0}c.md"]
        );
    }
}
