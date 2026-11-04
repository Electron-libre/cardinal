---
priority: low
depends_on: [skip_the_html_comments_of_the_requirements_section]
why: 'a comfort: no spec of this repository indents a fence by four spaces or opens a comment after the text of an item'
---
# Unify the fence grammars of the Markdown scanners

# Goal
The tools read fences with two grammars: `tools/mod.rs::fence` (any indentation, a backtick fence whose info string
holds a backtick is inline code) and `roadmap::next_fence` (three spaces of indentation at most). Both are fed to
`tools/mod.rs::hidden`, whose comment state reads a line indented by four spaces as an indented code block. In
`check-specs`, a fence indented by four spaces therefore opens a block that hides the requirements after it, where
CommonMark reads an indented code block. One grammar, close to CommonMark, should mark the hidden lines of every tool.

# Findings
* Found by the `high` review of `issues/done/skip_the_html_comments_of_the_requirements_section.md` (2026-11-04).
* `check-specs` passes `super::fence` to `hidden`; `roadmap` passes `next_fence`; `language` and
  `delivered_quality` still use `super::fenced`.
* A `<!--` after the text of a list item (`- **RER-1** ... <!-- see`) opens a comment that hides the following items
  up to a `-->`; CommonMark ends that inline comment at the next block. The roadmap grammar states this
  behaviour on purpose (RMP-1: a line that opens one after its text stays text).

# Approach
1. Keep one fence step (`next_fence`, with the backtick info string rule of `fence`) and drop `fence` and `fenced`,
   so `hidden` loses its type parameter.
2. Decide whether a comment opened after text ends at the next list item or heading, which would amend RMP-1.

# Specs
* `specs/roadmap.md` (RMP-1) if approach 2 changes the comment rule; the checks of `check-specs` are tested by
  `tests/check_specs.rs`.

# Resources
* `crates/cardinal/src/tools/mod.rs` (`fence`, `fenced`, `hidden`)
* `crates/cardinal/src/tools/roadmap.rs` (`fence_marker`, `next_fence`)
* `crates/cardinal/src/tools/check_specs.rs`, `language.rs`, `delivered_quality.rs`

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-04

## Understanding
Every Markdown scanner of the tools reads fences with one step (`roadmap::next_fence`: three spaces of indentation at
most, a backtick fence whose info string holds a backtick is inline code), and a comment opened after the text of a
line ends at the end of its paragraph. Out of scope: list-aware CommonMark parsing. `fenced` stays, as a thin
iterator over that one step: `language` and `delivered_quality` must not hide comments (QUA-12, BRE-10 say nothing of them).

## Plan
1. [x] One fence grammar: `next_fence` takes the backtick info string rule, `fence` is dropped, `fenced` and
   `check-specs` use `next_fence`, `hidden` loses its type parameter; verified by `tests/delivered_quality.rs` (a
   four-space fence left open no longer hides `# Tracking`) and a `tests/check_specs.rs` case (a four-space fence
   left open hides no requirement and is not an unclosed block); makes true QUA-12. (autonomous)
2. [x] A comment opened after the text of a line ends at the end of its paragraph (blank, list item, heading, fence,
   quote), the paragraph breakers of `check-specs` moved to `tools/mod.rs`; verified by `tests/roadmap.rs` and a
   `tests/check_specs.rs` case (`- **ABC-1** ... <!-- see` followed by `- **ABC-2**`); makes true RMP-1. (autonomous)
3. [x] High review fixes: a comment block and a nested item end an inline comment, a heading ends it on its line, an
   inline comment open at the end is not unclosed, `with_continuation` and `link_targets` reuse the shared steps;
   verified by the `hidden` cases of `roadmap.rs`; makes true RMP-1. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 668d716d49efc5a81c66da437443420e72ed8a55
- A fence opens indented by three spaces at most (1 a).
- A comment opened after text ends at the end of its paragraph, RMP-1 amended (2 a).

## Deliverable 1
- done: `Fence`, `fence_marker` and `next_fence` moved to `tools/mod.rs` (three spaces of indentation at most, the
  backtick info string rule added); `fence` dropped, `fenced` and `hidden` (no more type parameter) read through
  `next_fence`, so `check-specs`, `language`, `delivered_quality`, `roadmap` and `close-issue` share one grammar.

| check | result |
|---|---|
| tests | `cardinal run project-check` passes |
| lint | clippy clean |
| specs | QUA-12 amended (three spaces at most), with its scenario |
| docs | no doc names the fence grammar |
| code review | `low`: no findings |

- red: QUA-12: `delivered_quality`: "a heading inside a fence neither starts nor ends a section, whatever the fence" failed (a four-space fence left open hid `# Tracking`); `check_specs`: "a fence indented by four spaces opens no block" failed.
- Code review: no findings
- not verified: none
- decisions needed: none

## Deliverable 2
- done: `hidden` ends a comment opened after the text of a line at the end of its paragraph (`breaks_paragraph`:
  blank, list item, heading, fence, quote); `is_list_item` and `is_heading` moved from `check_specs.rs` to
  `tools/mod.rs`; the unit test of the old behaviour now states the new one.

| check | result |
|---|---|
| tests | `cardinal run project-check` passes |
| lint | clippy clean |
| specs | RMP-1 amended, with its scenario |
| docs | no doc states the comment rule |
| code review | `low`: 1 finding, fixed |

- red: RMP-1: `roadmap`: "a_comment_after_text_ends_with_its_paragraph" failed (the `issues` cell of `Next` read empty); `check_specs`: "a comment opened after the text of a requirement ends with its paragraph" failed (RER-2 hidden).
- Code review: 1 finding: `breaks_paragraph` ignored indentation, so a line indented by four spaces ended the comment; fixed (three spaces at most), proved by a `hidden` case in `roadmap.rs`.
- not verified: none
- decisions needed: none

## Deliverable 3
- done: fixes of the `high` review of `668d716..b376800` (8 findings): `breaks_paragraph` ends an inline comment on
  a comment opener and on a list item at any indentation (nested items); a heading ends a comment opened after its
  text on its own line; `hidden` reports no unclosed comment for an inline one; `with_continuation` (CLR-8) calls
  `breaks_paragraph`; `link_targets` reads its fences through `fenced`; `hidden` computes the fence marker once.

| check | result |
|---|---|
| tests | `cardinal run project-check` passes |
| lint | clippy clean |
| specs | no change (RMP-1 as amended) |
| docs | none |
| code review | `high`: 8 findings, 7 fixed, 1 became an issue; `low`: no findings |

- red: none (the `hidden` cases were written with the fixes)
- High code review: 8 findings: 7 fixed; the fence parser of `memory.rs` (REP-2) is out of the tools: became
  `issues/done/read_the_fences_of_memory_imports_with_the_shared_grammar.md`. `next_fence` still calls `fence_marker` once
  more per line: negligible.
- Code review: no findings (low)
- not verified: none
- decisions needed: none
