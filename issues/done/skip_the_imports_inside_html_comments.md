---
priority: low
why: 'a comfort: no memory file of a consumer is known to hold an import in an HTML comment'
---
# Skip the imports inside HTML comments

# Goal
`memory.rs::imports` (REP-2, SYN-38) still scans the HTML of a memory file for `@<path>` imports. Claude Code skips
an HTML token unless it starts with a comment, and then scans only what is left once its `<!--...-->` comments are
removed, so a commented-out import counts a file a session never loads.

# Findings
* Found by the `high` review of `issues/done/skip_the_imports_of_every_markdown_code_block.md` (2026-11-04), left out
  of it by its decision 2 a.
* `<!-- old: @docs/big.md -->` counts `docs/big.md` in `claude_md_bytes` and `instruction_files`.
* Claude Code 2.1.318 (read from its binary): `if(S.type==="html"){... if(H.startsWith("<!--")&&H.includes("-->"))
  {strip the comments; scan the rest} continue}`.

# Approach
1. In `imports`, read an `Html` or `InlineHtml` event of `pulldown-cmark` as Claude Code reads an `html` token of
   `marked`: nothing unless it starts with a whole comment, else the text left once the comments are removed.
2. Check how the HTML blocks of `pulldown-cmark` (one event per line) map to the `html` tokens of `marked` (one per
   block) for a comment over several lines.

# Specs
* `specs/cardinal-report.md` (REP-2): an import inside an HTML comment or tag counts nothing.

# Resources
* `crates/cardinal/src/memory.rs` (`imports`)

# Hints
* What counts is what Claude Code loads, not only CommonMark.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-05

## Understanding
Read HTML as Claude Code 2.1.318 does (`h()` of its import scanner): an inline HTML event counts nothing (a `marked`
tag token is the tag or comment alone), and an HTML block, its `Html` lines gathered between `Start(HtmlBlock)` and
`End(HtmlBlock)`, counts only when, trimmed, it starts with `<!--` and holds `-->`, then in what is left once the
`<!--...-->` comments are removed. Out of scope: the content stripping of loaded files (`jYe`), which `claude_md_bytes`
does not model.

## Plan
1. [x] HTML read as Claude Code reads it: `imports` gathers each HTML block, skips it unless it starts with a whole
   comment, else scans its text without comments, and skips inline HTML; verified by the unit tests of `memory.rs`
   (one-line and multi-line comment blocks, a block quote, a `<div>` block, inline comment and tag); makes true REP-2. (autonomous)
2. [x] Fixes of the `high` review: the binary test `fixed_context.rs` proves the HTML cases of REP-2; the cases where
   `marked` departs from CommonMark go to `issues/done/read_list_items_and_html_interruptions_as_marked_does.md`;
   verified by `fixed_context`, failing on the code of the base commit; makes true REP-2. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 93e8d59f2b7dd9e0be8698cf1661ce5e585a1491
- Inline HTML still only splits the text: `marked` reads the text between two tags as a text token, so
  `<span>@b.md</span>` imports `b.md`; REP-2 says so.

## Summary of deliverable 1
done: an HTML block counts its imports only when it starts with a whole comment, once its comments are removed; an
inline comment counts nothing.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | REP-2 and its scenario, doc comment of `imports` |
| code review | see below |

red: REP-2: `left: ["docs/big.md", "a.md", "b.md"] right: []` (`an_import_stands_outside_html`)

Code review: 2 findings
- `<!-->` counted closed by the check, not by the removal loop: no change, Claude Code does the same (`includes("-->")`
  then `/<!--[\s\S]*?-->/g`, whose close is searched after the opening).
- an unclosed comment after a closed one is scanned: no change, the regex of Claude Code leaves it in place too.

not verified: none
decisions needed: none

## High review of the issue
Code review: 4 findings
- tight-list text tokens scanned raw by Claude Code (`- note <!-- @a.md -->`): outside the requirements, issue
  `read_list_items_and_html_interruptions_as_marked_does.md`.
- HTML blocks of types 3 to 5 that `marked` does not let interrupt a paragraph: same issue.
- a lone tag line in a list item, an HTML block for `marked`: same issue.
- no binary test of the HTML scenario: fixed, deliverable 2.

## Summary of deliverable 2
done: `docs/x.md` of the `fixed_context` fixture holds a commented and a `<div>` import of `big.md`, not counted.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | none to change |
| code review | see below |

red: REP-2: `FAIL - the bytes and skills come from the commit the revision came in force with` (on the code of the base commit)

Code review: no findings

not verified: cases 1 and 3 of the new issue are derived from the rules of `marked`, not run
decisions needed: none

All deliverables done.
