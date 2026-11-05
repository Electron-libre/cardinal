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
   CommonMark (`issues/read_list_items_and_html_interruptions_as_marked_does.md`), and may stay apart.

# Specs
* `specs/spec-clarity.md` (CLR-5, CLR-8), `specs/close-issue.md` (CLS-6).

# Resources
* `crates/cardinal/src/markdown.rs`, `crates/cardinal/src/tools/check_specs.rs`, `crates/cardinal/src/tools/close_issue.rs`,
  `crates/cardinal/src/tools/language.rs`

# Hints
* `delivered_quality` keeps the fence rule of QUA-12 (`markdown::fenced`): an older issue file must be read as before.
