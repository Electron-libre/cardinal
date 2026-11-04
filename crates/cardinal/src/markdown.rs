//! The Markdown scanning of the tools: fenced code blocks (QUA-12), HTML comments and code spans.

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

/// The lines of `lines` outside every fenced code block, its fence lines included.
pub(crate) fn unfenced<'a>(
    lines: impl IntoIterator<Item = &'a str>,
) -> impl Iterator<Item = &'a str> {
    fenced(lines).filter_map(|(line, fenced)| (!fenced).then_some(line))
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
    let hidden = lines
        .iter()
        .map(|line| {
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
                let (open, opener) = comment_after(line, comment);
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
/// that opened it on this line, if any: a comment holds no code span and ends at its first `-->`, a `<!--` inside a
/// code span opens none.
fn comment_after(line: &str, mut open: bool) -> (bool, Option<usize>) {
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
            let spans = code_spans(rest);
            let start = rest
                .match_indices("<!--")
                .map(|(i, _)| i)
                .find(|i| !spans.iter().any(|span| span.contains(i)));
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
pub(crate) fn code_spans(line: &str) -> Vec<std::ops::Range<usize>> {
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
    replace_code_spans(line, " ")
}

/// `line` with each of its code spans (`code_spans`) replaced by `with`.
fn replace_code_spans(line: &str, with: &str) -> String {
    let mut out = String::new();
    let mut last = 0;
    for span in code_spans(line) {
        out.push_str(&line[last..span.start]);
        out.push_str(with);
        last = span.end;
    }
    out.push_str(&line[last..]);
    out
}
