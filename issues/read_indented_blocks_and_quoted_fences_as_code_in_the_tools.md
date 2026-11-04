---
priority: low
why: 'a comfort: no known consumer file holds these cases; the tools and the imports would read code blocks alike'
---
# Read indented blocks and quoted fences as code in the tools

# Goal
`memory.rs::imports` reads Markdown with `pulldown-cmark`, the tools with `crate::markdown` by hand: an indented code
block or a fence inside a block quote is code for the imports and prose for the tools. The tools should read the
code blocks and code spans as CommonMark does.

# Findings
* `high` review of `issues/done/read_html_comments_and_multiline_code_spans_as_commonmark.md` (2026-11-04): an indented
  block or a quoted fence is code for `imports` and prose for `hidden`, `fenced` and `prose_words`.
* Its second `high` review (2026-11-05): `markdown::code_spans_by_line` reads an indented block after a blank line and
  the `-->` line that ends an HTML block as paragraph text, so a code span pairs across them (`prose_words`, BRE-10).
* `hidden` reads the code spans line by line: a `<!--` inside a code span over two lines still opens a comment
  (RMP-1).

# Approach
1. Tokenize `prose_words` with `pulldown-cmark` (`Event::Text` outside code blocks, `Event::Code` left out), as
   `imports` does; BRE-10 and LNG-4 then name the CommonMark rules instead of the fence rule of QUA-12.
2. Do the same for `hidden` (code blocks, `Event::Html` for the comments): RMP-1's rule that a comment opened after
   text ends at the end of its paragraph differs from CommonMark (an unclosed inline `<!--` is text) and would be
   amended.

# Specs
* `specs/rule-breaches.md` (BRE-10), `specs/language-hint.md` (LNG-4), `specs/roadmap.md` (RMP-1).

# Resources
* `crates/cardinal/src/markdown.rs`, `crates/cardinal/src/tools/language.rs`, `crates/cardinal/src/memory.rs`,
  `specs/delivered-quality.md` (QUA-12)

# Hints
* `check-specs` (`hidden`, CLR-5) and `roadmap` read the same module: a change of `hidden` moves both.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-05

## Understanding
`prose_words` (BRE-10, LNG-4) and `markdown::hidden` (RMP-1, CLR-5, and the milestone grammar of REL-9, CLS-8,
NXT-10) read Markdown with `pulldown-cmark`, as `memory::imports` does: indented and quoted code blocks become code, an
unclosed inline `<!--` becomes text. Out of scope: `fenced`/`code_spans` of `close-issue` and `delivered-quality`
(QUA-12), the per-line code spans of `check_markers`, and the paragraph rule of CLR-8.

## Plan
1. [x] Prose words as CommonMark: `prose_words` keeps the `Event::Text` outside code blocks (code spans, HTML left out); `code_spans_by_line` and `without_spans` removed if unused; verified by tests of the two new BRE-10 scenarios and the existing ones; makes true BRE-10, LNG-4. (autonomous)
2. [ ] Hidden lines as CommonMark: `hidden` marks a line code or comment when its first content lies in a code block or an HTML comment (block or inline), reports a fenced block or a comment block reaching the end unclosed; the hand-written comment scan (`comment_after`, the inline rule) removed; verified by the new RMP-1 scenarios through `roadmap`, the amended internal tests of `roadmap.rs` and a CLR-5 test of an indented block; makes true RMP-1, CLR-5, REL-9, CLS-8, NXT-10.

## Decisions
- Mode: autonomous
- Base commit: 27d0c25193de9a876fab967b0d4748a8101c6b52
- The words of an HTML comment are not prose (BRE-10): only `Event::Text` counts.
- `fenced`, `code_spans` (close-issue, QUA-12) and the paragraph rule of CLR-8 stay out of scope.

## Deliverable 1

done: `prose_words` reads the prose with `pulldown-cmark` (code blocks, code spans and HTML left out); `code_spans_by_line` removed; the spec drafts of the whole issue committed with it (`rule-breaches`, `language-hint` accepted again, the others `draft` until deliverable 2).

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | pass (two older wording warnings, CLS-9 and BRE-5, untouched) |
| docs | n/a: no document describes how the prose is read |
| code review | no findings |

red: BRE-10: the indented block of English stop words was a breach (`s.jsonl:10`) and the backtick of the `-->` line paired with the next one, hiding the breach of `s.jsonl:12`.

Code review: no findings (`low`).

not verified: none. decisions needed: none.
