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
