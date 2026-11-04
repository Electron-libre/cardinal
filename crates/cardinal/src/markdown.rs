//! The Markdown scanning of the tools: fenced code blocks (QUA-12), HTML comments and code spans.

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

/// Whether each line of `lines` is part of a fenced block (its fences included) and whether it is part of an HTML
/// comment (one ends at its first `-->`, code spans aside: see `comment_after`; a line that opens one with `<!--` after its text stays text, and that comment ends at the end of its
/// paragraph, as in `CommonMark`: see `breaks_paragraph`, or with its line on a heading); then the fence and whether a
/// comment are still open after the last line. A fence inside a comment opens no block, a comment opener in a fence line opens no comment (RMP-1).
pub(crate) fn hidden(lines: &[&str]) -> (Vec<(bool, bool)>, Option<Fence>, bool) {
    let mut fence = None;
    let mut comment = false;
    // whether the open comment was opened after the text of its line
    let mut inline = false;
    let spans = code_spans_by_line(lines);
    let hidden = lines
        .iter()
        .zip(&spans)
        .map(|(line, spans)| {
            let body = line.trim_start_matches(' ');
            // an indented code block (four spaces) holds no comment
            let text = line.len() - body.len() <= 3;
            let marker = fence_marker(line);
            if comment && inline && breaks_paragraph(line, marker.is_some()) {
                comment = false;
            }
            let in_comment = comment || (fence.is_none() && text && body.starts_with("<!--"));
            let before = fence;
            if comment || (fence.is_none() && text && marker.is_none()) {
                let (open, opener) = comment_after(line, spans, comment);
                // a comment opened on this line: after text (a code span is text) or at its start
                if let Some(at) = opener {
                    inline = !line[..at].trim().is_empty();
                }
                // a heading is one line: a comment opened after its text ends with it
                comment = open && !(inline && text && is_heading(body.trim()));
            }
            if !in_comment {
                fence = next_fence(fence, line);
            }
            (
                !in_comment && (before.is_some() || fence.is_some()),
                in_comment,
            )
        })
        .collect();
    // an inline comment open at the end ends with its paragraph
    (hidden, fence, comment && !inline)
}

/// Whether a comment is open after `line`, given whether one is open before it, and the byte offset of the `<!--`
/// that opened it on this line, if any: a comment holds no code span and ends at its first `-->`, a `<!--` inside
/// one of the code spans `spans` of the line opens none.
fn comment_after(line: &str, spans: &[Range<usize>], mut open: bool) -> (bool, Option<usize>) {
    let mut opener = None;
    let mut at = 0;
    loop {
        let rest = &line[at..];
        if open {
            let Some(close) = rest.find("-->") else {
                return (true, opener);
            };
            open = false;
            opener = None;
            at += close + 3;
        } else {
            let start = rest
                .match_indices("<!--")
                .map(|(i, _)| i)
                .find(|i| !spans.iter().any(|span| span.contains(&(at + i))));
            let Some(start) = start else {
                return (false, None);
            };
            open = true;
            opener = Some(at + start);
            at += start + 4;
        }
    }
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

/// The byte ranges of each line of `lines` inside a code span, the spans read over the line endings of each paragraph
/// (`breaks_paragraph`; a heading is a paragraph of one line, a fenced block holds none), as in `CommonMark`.
pub(crate) fn code_spans_by_line(lines: &[&str]) -> Vec<Vec<Range<usize>>> {
    let fences: Vec<bool> = fenced(lines.iter().copied()).map(|(_, f)| f).collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut start = 0;
    while start < lines.len() {
        let mut end = start + 1;
        if !fences[start] && !is_heading_line(lines[start]) {
            while end < lines.len() && !fences[end] && !breaks_paragraph(lines[end], false) {
                end += 1;
            }
        }
        let paragraph = lines[start..end].join("\n");
        let spans = if fences[start] {
            vec![]
        } else {
            code_spans(&paragraph)
        };
        let mut offset = 0;
        for line in &lines[start..end] {
            let within = |span: &Range<usize>| {
                let (from, to) = (span.start.max(offset), span.end.min(offset + line.len()));
                (from < to).then(|| from - offset..to - offset)
            };
            out.push(spans.iter().filter_map(within).collect());
            offset += line.len() + 1;
        }
        start = end;
    }
    out
}

/// Whether `line` is an ATX heading indented by three spaces at most.
fn is_heading_line(line: &str) -> bool {
    line.len() - line.trim_start_matches(' ').len() <= 3 && is_heading(line.trim())
}

/// `line` with each of its code spans (`code_spans`) replaced by a space: its text outside them, in order.
pub(crate) fn without_code_spans(line: &str) -> String {
    without_spans(line, &code_spans(line))
}

/// `line` with each of the ranges `spans`, in order, replaced by a space.
pub(crate) fn without_spans(line: &str, spans: &[Range<usize>]) -> String {
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
