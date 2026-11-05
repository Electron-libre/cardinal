//! The Markdown scanning of the tools: the fenced code blocks of QUA-12, read by hand, and one walk of `pulldown-cmark`
//! that says of each line whether it lies in a code block or an HTML comment, where its code spans are and which
//! paragraph it belongs to (RMP-1, CLR-8).

use std::ops::Range;

/// A fence of a fenced code block: its character (`` ` `` or `~`) and its length, three at least.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Fence {
    ch: char,
    len: usize,
}

/// The fence `line` starts with, indented by three spaces at most (four make an indented code block), and whether
/// nothing but spaces follows it; a backtick fence followed by a backtick is inline code, not a fence (QUA-12).
fn fence_marker(line: &str) -> Option<(Fence, bool)> {
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

/// What the `CommonMark` reading of a text says of one of its lines (RMP-1): whether it lies in a code block, whether it
/// lies in an HTML comment, the byte ranges of the line inside inline code spans, and the paragraph its first inline
/// text belongs to: the lines of one paragraph, or of the text of one list item, a heading or a table cell, share it.
#[derive(Clone, Default)]
pub(crate) struct Line {
    pub(crate) code: bool,
    pub(crate) comment: bool,
    pub(crate) spans: Vec<Range<usize>>,
    pub(crate) paragraph: Option<usize>,
}

/// The lines of a text as `CommonMark` reads them, then whether a fenced block and whether a comment block run to the
/// end of the text with no closing fence or `-->`.
pub(crate) struct Walk {
    pub(crate) lines: Vec<Line>,
    pub(crate) open_fence: bool,
    pub(crate) open_comment: bool,
}

/// Read `lines` as `CommonMark` does, in one walk of `pulldown-cmark` (RMP-1): a line lies where its first character
/// does, the markers of its quotes and list items aside. A code block is fenced (its fences included) or indented, in
/// a quote or a list item or not; a comment is one of an HTML block (`html_comments`) or an inline `<!--...-->`. A code
/// span, backticks included, may run over several lines: each of them gets its part, the line ending aside.
pub(crate) fn walk(lines: &[&str]) -> Walk {
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
    let mut read = vec![Line::default(); lines.len()];
    // the number of the run of inline text being read: any block that starts or ends ends it
    let mut paragraph = 0;
    let (mut open_fence, mut open_comment) = (false, false);
    // the range of the fenced block being read and the end of its content so far
    let mut fence: Option<(Range<usize>, usize)> = None;
    // the comments of the HTML block being read
    let mut comments: Vec<Range<usize>> = Vec::new();
    for (event, range) in Parser::new(&text).into_offset_iter() {
        if !is_inline(&event) {
            paragraph += 1;
        } else if !range.is_empty() {
            for line in &mut read[line_of(range.start)..=line_of(range.end - 1)] {
                line.paragraph.get_or_insert(paragraph);
            }
        }
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
            Event::Code(_) => {
                for i in line_of(range.start)..=line_of(range.end - 1) {
                    // the part of the span on line `i`, its line ending aside
                    let content = lines[i].trim_end_matches(['\n', '\r']).len();
                    let span =
                        range.start.saturating_sub(starts[i])..(range.end - starts[i]).min(content);
                    read[i].spans.extend((!span.is_empty()).then_some(span));
                }
                Some((false, false))
            }
            Event::Start(_) | Event::End(_) | Event::SoftBreak | Event::HardBreak => None,
            _ => Some((false, false)),
        };
        if let Some(kind) = kind.filter(|_| !range.is_empty()) {
            for line in &mut hidden[line_of(range.start)..=line_of(range.end - 1)] {
                line.get_or_insert(kind);
            }
        }
    }
    for (line, kind) in read.iter_mut().zip(hidden) {
        (line.code, line.comment) = kind.unwrap_or_default();
    }
    Walk {
        lines: read,
        open_fence,
        open_comment,
    }
}

/// Whether `event` lies inside a run of inline text rather than starting or ending a block.
fn is_inline(event: &pulldown_cmark::Event) -> bool {
    use pulldown_cmark::{Event, Tag, TagEnd};
    match event {
        Event::Start(tag) => matches!(
            tag,
            Tag::Emphasis
                | Tag::Strong
                | Tag::Strikethrough
                | Tag::Superscript
                | Tag::Subscript
                | Tag::Link { .. }
                | Tag::Image { .. }
        ),
        Event::End(tag) => matches!(
            tag,
            TagEnd::Emphasis
                | TagEnd::Strong
                | TagEnd::Strikethrough
                | TagEnd::Superscript
                | TagEnd::Subscript
                | TagEnd::Link
                | TagEnd::Image
        ),
        Event::Text(_)
        | Event::Code(_)
        | Event::InlineMath(_)
        | Event::InlineHtml(_)
        | Event::SoftBreak
        | Event::HardBreak
        | Event::FootnoteReference(_)
        | Event::TaskListMarker(_) => true,
        Event::Html(_) | Event::DisplayMath(_) | Event::Rule => false,
    }
}

impl Walk {
    /// Each of `lines`, the lines it was read from, with its code spans blanked (`blank`); `None` for a line of a code
    /// block.
    pub(crate) fn prose(&self, lines: &[&str]) -> Vec<Option<String>> {
        lines
            .iter()
            .zip(&self.lines)
            .map(|(line, read)| (!read.code).then(|| blank(line, &read.spans)))
            .collect()
    }
}

/// Whether each line of `lines` lies in a code block and whether it lies in an HTML comment, as `walk` reads them,
/// then whether a fenced block and whether a comment block run to the end of the text unclosed.
pub(crate) fn hidden(lines: &[&str]) -> (Vec<(bool, bool)>, bool, bool) {
    let walk = walk(lines);
    let hidden = walk.lines.iter().map(|l| (l.code, l.comment)).collect();
    (hidden, walk.open_fence, walk.open_comment)
}

/// `line` with each byte of the ranges `spans` replaced by a space: its text outside them, at the same offsets.
fn blank(line: &str, spans: &[Range<usize>]) -> String {
    let mut out = line.as_bytes().to_vec();
    for span in spans {
        out[span.clone()].fill(b' ');
    }
    String::from_utf8(out).expect("whole characters replaced by spaces")
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
