---
priority: low
why: 'a comfort: no known spec, issue or roadmap holds these cases; the tools would read Markdown with one parser'
---
# Read the last Markdown of the tools as CommonMark

# Goal
`markdown::hidden` and `language::prose_words` now read Markdown with `pulldown-cmark`; other readers of the tools still
scan it by hand, line by line, and read code spans, code blocks and paragraphs otherwise. They should read them as
CommonMark does, from one shared walk of the parser.

# Findings
* `high` review of `issues/done/read_indented_blocks_and_quoted_fences_as_code_in_the_tools.md` (2026-11-05), read from the
  code, not run:
* 1. `check_specs::check_markers` removes the code spans line by line (`markdown::without_code_spans`), while CLR-5
  says a code span as in CommonMark: in an accepted spec, ``Write `the marker`` then
  ``[NEEDS CLARIFICATION: x]` like this.`` on the next line leaves the marker flagged.
* 2. `close_issue` finds code with `markdown::fenced` and `markdown::code_spans` by line: a link in a fenced block
  inside a quote or a list item (`> ```` / `> [a](issues/a.md)`), or in an indented block, is rewritten (CLS-6).
* 3. The continuation lines of CLR-8 end at `markdown::breaks_paragraph` (with `fence_marker`, `is_list_item`,
  `is_heading`): a type-6 HTML block (`<div>`) or a setext underline (`---`) does not end the statement, so its text
  joins the wording checks of CLR-3 and CLR-4.
* 4. `markdown::hidden`, `memory::imports` and `language::prose_words` each walk `pulldown-cmark` and track the code
  blocks their own way.

# Approach
1. Share in `markdown.rs` one walk of `pulldown-cmark` that gives, per line, its code blocks, comments, code spans
   and paragraphs, and port `check_markers` (1) and `close_issue` (2) to it.
2. Read the statement of a requirement from the paragraph or item range of the parser instead of `breaks_paragraph`
   (3); drop the hand-written helpers left unused.
3. Optionally let `prose_words` and `hidden` use the shared walk (4); `memory::imports` follows `marked`, not
   CommonMark (`issues/done/read_list_items_and_html_interruptions_as_marked_does.md`), and may stay apart.

# Specs
* `specs/spec-clarity.md` (CLR-5, CLR-8), `specs/close-issue.md` (CLS-6).

# Resources
* `crates/cardinal/src/markdown.rs`, `crates/cardinal/src/tools/check_specs.rs`, `crates/cardinal/src/tools/close_issue.rs`,
  `crates/cardinal/src/tools/language.rs`

# Hints
* `delivered_quality` keeps the fence rule of QUA-12 (`markdown::fenced`): an older issue file must be read as before.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-05

## Understanding
One `pulldown-cmark` walk in `markdown.rs` gives, per line, code block, comment, code span ranges, and the paragraph
line ranges; `check_markers`, `check_wording` (CLR-5, CLR-8) and `close_issue::link_targets` (CLS-6) read it, and the
hand-written helpers left unused go. Out of scope: `memory::imports` (follows `marked`), `delivered_quality` (QUA-12),
`prose_words` (reads text, not lines: it stays on its own walk).

## Plan
1. [ ] Shared walk and code: `markdown::walk` (code, comment, code spans per line; `hidden` becomes a view of it),
   `check_markers` and `check_wording` drop the code spans it reads, `link_targets` skips its code blocks and spans;
   verified by tests of a code span over two lines (check-specs) and of a link in a quoted fence and in an indented
   block (close-issue); makes true CLR-5, CLS-6. ~250 lines.
2. [x] Statement as a paragraph: the walk gives the paragraph line ranges, `with_continuation` reads them;
   `breaks_paragraph`, `is_list_item`, `is_heading`, `code_spans`, `without_code_spans` dropped; verified by a test of
   a `<div>` and a `---` after a requirement; makes true CLR-8. ~150 lines. (autonomous)

3. [x] Fixes of the `high` review of the issue: the statement offset of a requirement line ending in spaces, one
   `Read` of the walk in `check_specs` instead of three parallel vectors; verified by a test of a line ending in a hard
   break; makes true CLR-8. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 99ef7c199683ea47f31d68398f2e26ab3f466510
- Q1: a, CLR-8 and CLS-6 keep their ids, their wording is amended.

## Deliverable 1
done: `markdown::walk` gives code, comment and code spans per line; `check_markers`, `check_wording` and
`link_targets` read it; `code_spans` and `without_code_spans` dropped.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (`spec-clarity` draft until deliverable 2) |
| docs | module doc of `markdown.rs` |
| code review | see below |

red: CLR-5, CLS-6: `FAIL - a clarification marker in a code span over two lines of an accepted spec passes`, `FAIL - a link in a code span over two lines, a quoted fence or an indented block is untouched`

## Deliverable 2
done: `markdown::walk` numbers the paragraph of each line; `with_continuation` reads the paragraph of the item line;
`breaks_paragraph`, `is_list_item`, `is_heading` dropped; `spec-clarity` back to `accepted`; `specs/README.md` says it.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | `files/specs/README.md` (synced), module doc of `markdown.rs` |
| code review | no findings |

red: CLR-8: `FAIL - the continuation lines end before "\n<div>\nit may must\n</div>"`, `"\n---\nit may must"`, `"\n  ---\n  it may must"`

Code review: no findings

not verified: none. decisions needed: none.

## Deliverable 3
done: the statement offset of a requirement no longer counts the trailing spaces of its line (a panic or a misread
`withdrawn:`); `check_specs` passes one `Read` of the walk; the doc of `walk` says where a code span over several lines
starts on its later lines.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | doc comments of `walk` and `with_continuation` |
| code review | no findings |

red: CLR-8: `FAIL - a requirement line ending in a hard break passes` (panic: byte index not a char boundary)

High review of the issue: 7 findings
1. offset of the statement past trailing spaces: fixed.
2. backticks in an HTML comment no longer hide a marker: kept, as CLR-5 reads code spans as CommonMark does, and
   CommonMark reads none in HTML; no spec of this repository or the local consumers cites a marker that way.
3. a line inside a multi-line inline comment ends the statement: not fixed, as before this issue (CLR-8 ends at an
   HTML comment); rare.
4. a code span over several lines blanks the quote markers of its later lines: documented; no caller reads them.
5. `link_targets` still scans `](` by hand: out of the requirements of this issue,
   `issues/take_the_link_targets_of_close_issue_from_the_parser.md` created.
6. three parallel vectors in `check_specs`: fixed (`Read`).
7. a full walk per file in `close-issue --relink`: not fixed; `pulldown-cmark` is linear and the files are small.

Code review: no findings

not verified: none. decisions needed: none.

All deliverables done.
