---
priority: low
why: 'a comfort: no known consumer file holds either case; the scanners would follow CommonMark on two more points'
---
# Read HTML comments and multi-line code spans as CommonMark

# Goal
`crate::markdown` reads code spans line by line and also inside HTML comments. In CommonMark the content of a comment
holds no code span (its first `-->` closes it), and a code span can run over the line endings of its paragraph.

# Findings
* Found by the `high` review of `issues/done/move_the_markdown_scanning_to_a_module_of_its_own.md` (2026-11-04).
* `hidden`: a block comment `<!--`, then a line ``use `-->` here``, then `## B`: the `-->` is blanked as code, the
  comment stays open and hides `## B` (`roadmap`, `check-specs`).
* `prose_words`: ``Use `the result of the`` then ``function` here`` on the next line: both halves hold an unmatched
  backtick and are read as prose by the language check (LNG-4, BRE-10).
* Since `issues/done/skip_the_imports_of_every_markdown_code_block.md`, `memory.rs::imports` reads Markdown with
  `pulldown-cmark` and the tools with `crate::markdown`: an indented block or a quoted fence is code for the imports
  and prose for the tools (`high` review, 2026-11-04).

# Approach
1. In `hidden`, search `-->` in the raw text of a line once a comment is open, and after the opener on its line.
2. Read the code spans of `prose_words` over each paragraph, not each line; check whether `hidden` and `imports` need
   the same.
3. Or tokenize with `pulldown-cmark`, as `imports` does, where the tools' rules match CommonMark.

# Resources
* `crates/cardinal/src/markdown.rs`, `crates/cardinal/src/tools/language.rs`, `specs/roadmap.md` (RMP-1),
  `specs/rule-breaches.md` (BRE-10)

# Hints
* RMP-1 says a comment ends at its `-->`: step 1 makes that literal; amend the specs to name the rules.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-04

## Understanding
Make `crate::markdown` follow CommonMark on two points: an open HTML comment ends at its first `-->` in the raw text
(no code span inside it), and a code span runs over the line endings of its paragraph (`prose_words`, `hidden`).
Out of scope: finding 3 (indented blocks and quoted fences read as prose by the tools), no move to `pulldown-cmark`.

## Plan
1. [x] Comment closer in the raw text: `hidden` searches `-->` in the raw line once a comment is open, and after the
   opener on its line; verified by a `roadmap` test (a comment holding ``use `-->` here`` then `## B`); makes true
   RMP-1 (amended). ~60 lines. (autonomous)
   - done: `hidden` reads the `-->` of an open comment in the raw text (`markdown::comment_after`).
   - red: RMP-1: `no row B`.
   - Code review: see the commit.
2. [x] Code spans over a paragraph: a `markdown` function that masks the code spans of each paragraph (lines grouped by
   `breaks_paragraph`, outside fences), used by `prose_words` and by `hidden` for its openers; verified by a
   `rule-breaches` test (a span split over two lines) and a `roadmap` test (``see `the <!--`` / ``marker` here``);
   makes true BRE-10 (amended), LNG-4, RMP-1. ~150 lines. (autonomous)
   - done: `markdown::code_spans_by_line` reads the spans of each paragraph; `prose_words` and `hidden` use it; `unfenced` dropped.
   - red: BRE-10, RMP-1: the split span was read as English prose (`FAIL - a long block ... is a breach`); the link after ``see `the <!--`` was hidden (`issues` cell empty).
   - Code review: no findings
3. [x] Fixes of the `high` review: `hidden` reads the code spans of each line again (the paragraph spans paired a
   backtick inside a comment, or after a fence inside one, with one after it), `<!-->` and `<!--->` close their
   comment, `prose_words` computes the fences once; verified by a `roadmap` test (`<!-->`, `<!--->`); makes true
   RMP-1. (autonomous)
   - done: `comment_after` back on `code_spans` of the line, closer searched after the `<!`; RMP-1 amended, its
     multi-line span scenario dropped.
   - red: RMP-1: `<!-->` hid every line after it (`issues` cell of `A` empty, no row `B`), reproduced by the review.
   - Code review: no findings

## Decisions
- Mode: autonomous
- Base commit: 95533a484bfc11a5d970c98d99d0205609ec6aea
- Finding 3 (indented blocks, quoted fences): out of scope, an issue of its own.
- `check_specs` (CLR-5) keeps reading code spans line by line.
- `high` review: `hidden` keeps the code spans of each line (a `<!--` in a span over two lines still opens a
  comment); only `prose_words` reads them over a paragraph. Left: an indented block after a blank line and the `-->`
  line of an HTML block are read as paragraph text by `code_spans_by_line`.
