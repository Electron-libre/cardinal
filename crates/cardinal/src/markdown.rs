//! The Markdown scanning of the tools: fenced code blocks (QUA-12), code spans, and the lines hidden in code blocks or
//! HTML comments as `CommonMark` reads them (RMP-1).

use std::ops::Range;

/// A fence of a fenced code block: its character (`` ` `` or `~`) and its length, three at least.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Fence {
    ch: char,
    len: usize,
}

/// The fence `line` starts with, indented by three spaces at most (four make an indented code block), and whether
/// nothing but spaces follows it; a backtick fence followed by a backtick is inline code, not a fence (QUA-12).
pub(crate) fn fence_marker(line: &str) -> Option<(Fence, bool)> {
    let body = line.trim_start_matches(' ');
    let ch = body.chars().next().filter(|c| ['`', '~'].contains(c))?;
    let len = body.len() - body.trim_start_matches(ch).len();
    let rest = &body[len..];
    (line.len() - body.len() <= 3 && len >= 3 && !(ch == '`' && rest.contains('`')))
        .then(|| (Fence { ch, len }, rest.trim().is_empty()))
}

/// The fence open after `line`, given the one open before it (`None` outside a fenced block): a block closes on a
/// bare fence of its character at least as long as the one that opened it.
fn next_fence(open: Option<Fence>, line: &str) -> Option<Fence> {
    match (open, fence_marker(line)) {
        (None, Some((f, _))) => Some(f),
        (Some(o), Some((f, true))) if f.ch == o.ch && f.len >= o.len => None,
        (open, _) => open,
    }
}

/// Whether each line of `lines` belongs to a fenced code block, its fence lines included.
pub(crate) fn fenced<'a>(
    lines: impl IntoIterator<Item = &'a str>,
) -> impl Iterator<Item = (&'a str, bool)> {
    let mut open = None;
    lines.into_iter().map(move |line| {
        let before = open;
        open = next_fence(open, line);
        (line, before.is_some() || open.is_some())
    })
}

/// Whether each line of `lines` lies in a code block and whether it lies in an HTML comment, as `CommonMark` reads them
/// (RMP-1): a line lies where its first character does, the markers of its quotes and list items aside. A code block
/// is fenced (its fences included) or indented, in a quote or a list item or not; a comment is one of an HTML block
/// (`html_comments`) or an inline `<!--...-->`. Then whether a fenced block and whether a comment block run to the end of the
/// text with no closing fence or `-->`.
pub(crate) fn hidden(lines: &[&str]) -> (Vec<(bool, bool)>, bool, bool) {
    use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
    let mut text = String::new();
    let mut starts = Vec::with_capacity(lines.len());
    for line in lines {
        starts.push(text.len());
        text.push_str(line);
        if !line.ends_with('\n') {
            text.push('\n');
        }
    }
    let line_of = |at: usize| starts.partition_point(|&start| start <= at) - 1;
    let mut hidden = vec![None; lines.len()];
    let (mut open_fence, mut open_comment) = (false, false);
    // the range of the fenced block being read and the end of its content so far
    let mut fence: Option<(Range<usize>, usize)> = None;
    // the comments of the HTML block being read
    let mut comments: Vec<Range<usize>> = Vec::new();
    for (event, range) in Parser::new(&text).into_offset_iter() {
        let kind = match &event {
            Event::Start(Tag::CodeBlock(kind)) => {
                if matches!(kind, CodeBlockKind::Fenced(_)) {
                    // its content starts after the line of its opening fence
                    let opened = starts
                        .get(line_of(range.start) + 1)
                        .copied()
                        .unwrap_or(text.len());
                    fence = Some((range.clone(), opened));
                }
                Some((true, false))
            }
            Event::Text(_) => {
                if let Some((_, content)) = &mut fence {
                    *content = (*content).max(range.end);
                }
                Some((false, false))
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((block, content)) = fence.take() {
                    // a closed block ends with the line of its closing fence, after its content
                    open_fence |= block.end == text.len() && block.end <= content;
                }
                None
            }
            Event::Start(Tag::HtmlBlock) => {
                comments = html_comments(&text, range.clone());
                open_comment |= range.end == text.len()
                    && comments
                        .last()
                        .is_some_and(|c| c.end == range.end && !text[c.clone()].ends_with("-->"));
                None
            }
            // a line of an HTML block lies where its first character does, blank or not
            Event::Html(line) => {
                let first = range.start + line.find(|c: char| !c.is_whitespace()).unwrap_or(0);
                Some((false, comments.iter().any(|c| c.contains(&first))))
            }
            // an ATX heading starts with its `#`, out of any inline comment
            Event::Start(Tag::Heading { .. })
                if text[range.clone()].trim_start().starts_with('#') =>
            {
                Some((false, false))
            }
            Event::InlineHtml(html) => Some((false, html.starts_with("<!--"))),
            Event::Start(_) | Event::End(_) | Event::SoftBreak | Event::HardBreak => None,
            _ => Some((false, false)),
        };
        if let Some(kind) = kind.filter(|_| !range.is_empty()) {
            for line in &mut hidden[line_of(range.start)..=line_of(range.end - 1)] {
                line.get_or_insert(kind);
            }
        }
    }
    let hidden = hidden.into_iter().map(Option::unwrap_or_default).collect();
    (hidden, open_fence, open_comment)
}

/// The byte ranges of the HTML comments of the HTML block `block` of `text`: from a `<!--` to its first `-->` after the
/// `<!` (`<!-->` and `<!--->` are whole comments), or to the end of the block.
fn html_comments(text: &str, block: Range<usize>) -> Vec<Range<usize>> {
    let mut comments = Vec::new();
    let mut at = block.start;
    while let Some(open) = text[at..block.end].find("<!--").map(|i| at + i) {
        let end = text[open + 2..block.end]
            .find("-->")
            .map_or(block.end, |i| open + 2 + i + 3);
        comments.push(open..end);
        at = end;
    }
    comments
}

/// Whether `line`, a fence or not as `fence` says, ends a paragraph: blank, a list item (at any indentation, nested
/// items included), or a heading, a quote or an HTML comment indented by three spaces at most (indented by four, they
/// continue it).
pub(crate) fn breaks_paragraph(line: &str, fence: bool) -> bool {
    let trimmed = line.trim();
    let indent = line.len() - line.trim_start_matches(' ').len();
    fence
        || trimmed.is_empty()
        || is_list_item(trimmed)
        || (indent <= 3
            && (is_heading(trimmed) || trimmed.starts_with('>') || trimmed.starts_with("<!--")))
}

/// Whether the trimmed line `trimmed` opens a list item that can break a paragraph: `-`, `*`, `+`, `1.` or `1)` (an
/// ordered list must start at 1 to break one, as in the `CommonMark` spec), then a space, a tab or nothing.
fn is_list_item(trimmed: &str) -> bool {
    let marker = trimmed.strip_prefix(['-', '*', '+']).or_else(|| {
        trimmed
            .strip_prefix("1.")
            .or_else(|| trimmed.strip_prefix("1)"))
    });
    marker.is_some_and(|r| r.is_empty() || r.starts_with([' ', '\t']))
}

/// Whether the trimmed line `trimmed` is an ATX heading: one to six `#`, then a space, a tab or nothing.
fn is_heading(trimmed: &str) -> bool {
    let rest = trimmed.trim_start_matches('#');
    let level = trimmed.len() - rest.len();
    (1..=6).contains(&level) && (rest.is_empty() || rest.starts_with([' ', '\t']))
}

/// Byte ranges of `line` inside inline code spans (a run of backticks up to the next run of the same length).
pub(crate) fn code_spans(line: &str) -> Vec<Range<usize>> {
    let mut spans = vec![];
    let mut from = 0;
    while let Some(i) = line[from..].find('`') {
        let open = from + i;
        let ticks = line[open..].len() - line[open..].trim_start_matches('`').len();
        let fence = &line[open..open + ticks];
        let mut search = open + ticks;
        let close = loop {
            match line[search..].find(fence) {
                Some(j) => {
                    let at = search + j;
                    let run = line[at..].len() - line[at..].trim_start_matches('`').len();
                    if run == ticks {
                        break Some(at);
                    }
                    search = at + run;
                }
                None => break None,
            }
        };
        match close {
            Some(close) => {
                spans.push(open..close + ticks);
                from = close + ticks;
            }
            None => from = open + ticks,
        }
    }
    spans
}

/// `line` with each of its code spans (`code_spans`) replaced by a space: its text outside them, in order.
pub(crate) fn without_code_spans(line: &str) -> String {
    without_spans(line, &code_spans(line))
}

/// `line` with each of the ranges `spans`, in order, replaced by a space.
fn without_spans(line: &str, spans: &[Range<usize>]) -> String {
    let mut out = String::new();
    let mut last = 0;
    for span in spans {
        out.push_str(&line[last..span.start]);
        out.push(' ');
        last = span.end;
    }
    out.push_str(&line[last..]);
    out
}
