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
2. [x] Hidden lines as CommonMark: `hidden` marks a line code or comment when its first content lies in a code block or an HTML comment (block or inline), reports a fenced block or a comment block reaching the end unclosed; the hand-written comment scan (`comment_after`, the inline rule) removed; verified by the new RMP-1 scenarios through `roadmap`, the amended internal tests of `roadmap.rs` and a CLR-5 test of an indented block; makes true RMP-1, CLR-5, REL-9, CLS-8, NXT-10. (autonomous)

3. [x] Fixes of the `high` review of the issue: a comment inside any HTML block hides the lines it starts, a heading opening with an inline comment stays a heading, the stale doc comments renamed; verified by an internal test and a binary test of `roadmap`; makes true RMP-1. (autonomous)

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

## Deliverable 2

done: `markdown::hidden` reads the code blocks and the HTML comments with `pulldown-cmark` (a line lies where its first content does); the hand-written comment scan removed; `check-specs` reports an unclosed fence or comment block from the same parse; the specs of RMP-1, CLR-5, REL-9, CLS-8, NXT-10 accepted again; `files/specs/README.md` says a marker in any code block is an example. `check-specs`, `roadmap` and `next-issue` print the same on this repository before and after.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | pass (one older wording warning, CLS-9, untouched) |
| docs | `files/specs/README.md` and its copy, the doc comments of `roadmap.rs` and `check_specs.rs` |
| code review | no findings |

red: RMP-1, CLR-5: the indented `    - State: done` was the state of `A` (`left: "done"`), and a marker in an indented block or a quoted fence was flagged in an accepted spec.

Code review: no findings (`low`).

not verified: none. decisions needed: none.

## Deliverable 3

`high` review of `27d0c25..ec0d88d`: 8 findings. Fixed: 1 (a comment in a `<details>` block hid nothing), 2 (`## <!-- x --> Later` was hidden), 6 (stale comments). Filed in `issues/read_the_last_markdown_of_the_tools_as_commonmark.md`, outside the validated scope (decision 2): 3 (`check_markers` code spans by line), 4 (`close_issue` fences by line), 5 (CLR-8 paragraph rule), 8 (three walkers of `pulldown-cmark`). Not fixed: 7 (`hidden` copies the lines into one text: the callers hold lines, the copy is cheap).

done: `hidden` reads the comments of every HTML block (`html_comments`) and the `#` of a heading as its first character; RMP-1 and two scenarios amended.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | pass |
| docs | doc comments of `markdown.rs`, `roadmap.rs`, `check_specs.rs` |
| code review | no findings |

red: RMP-1: `names` gave `["A", "B", "C"]` for the `<details>` block, and `roadmap` printed the state of `C` as `shaped`.

Code review: no findings (`low`).

not verified: none. decisions needed: none.
