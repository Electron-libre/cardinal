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
2. [ ] A comment opened after the text of a line ends at the end of its paragraph (blank, list item, heading, fence,
   quote), the paragraph breakers of `check-specs` moved to `tools/mod.rs`; verified by `tests/roadmap.rs` and a
   `tests/check_specs.rs` case (`- **ABC-1** ... <!-- see` followed by `- **ABC-2**`); makes true RMP-1.

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
