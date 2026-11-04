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
* Found by the `high` review of `issues/skip_the_html_comments_of_the_requirements_section.md` (2026-11-04).
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
