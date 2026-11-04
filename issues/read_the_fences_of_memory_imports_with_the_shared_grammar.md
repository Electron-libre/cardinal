---
priority: low
depends_on: [unify_the_fence_grammars_of_the_markdown_scanners]
why: 'a comfort: no memory file of a consumer is known to open a line with a backtick fence followed by inline code'
---
# Read the fences of memory imports with the shared grammar

# Goal
`memory.rs::imports`, which finds the `@<path>` imports counted in `claude_md_bytes` (REP-2), keeps its own fence
parser, while the tools read fences with `tools/mod.rs::fence_marker` and `next_fence`. It lacks the rule that a
backtick fence followed by a backtick is inline code, and counts a tab as indentation (`trim_start`). A memory file
line ```` ```just check``` fails. ```` opens a fence there and hides every later import, so `claude_md_bytes`
undercounts. The imports should be read with the same fence grammar as the other Markdown scanners.

# Findings
* Found by the `high` review of `issues/done/unify_the_fence_grammars_of_the_markdown_scanners.md` (2026-11-04).
* `memory.rs` is outside `tools/`: the shared fence step is private to `tools/mod.rs`.

# Approach
1. Make the fence step (`fenced` or `next_fence`) reachable from `memory.rs` (`pub(crate)`, or a module of its own)
   and read the imports through it.
2. Check whether REP-2 needs the fence rule stated (it says "outside a fenced code block" only).

# Specs
* `specs/cardinal-report.md` (REP-2), if approach 2 states the rule.

# Resources
* `crates/cardinal/src/memory.rs` (`imports`)
* `crates/cardinal/src/tools/mod.rs` (`fence_marker`, `next_fence`, `fenced`)

# Hints
* What counts is what Claude Code loads: the grammar should match the way Claude Code itself skips the imports in
  code blocks, not only CommonMark.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-04

## Understanding
`memory.rs::imports` reads its fences with `tools/mod.rs::fenced` (made `pub(crate)`) instead of its own parser: a
backtick fence followed by a backtick is inline code, a tab does not indent a fence. Claude Code tokenizes memory files
with `marked`, whose fence rule (` {0,3}`, no backtick in a backtick info string) is the one `fence_marker` already has.
REP-2 keeps its wording ("a fenced code block" is CommonMark's); one scenario states the case.

## Plan
1. [x] Shared fence grammar for imports: `fenced` reachable from `memory.rs`, `imports` reads through it, a REP-2
   scenario for the inline-code line; verified by a `budget_cli` test (a ```` ```just check``` ```` line, then an import
   counted); makes true REP-2. (autonomous)

## Decisions
- Mode: autonomous
- REP-2 keeps its wording; a scenario states the inline-code case (answer 1 a).
- Base commit: 318b9b25f2e32ee849534defdee060075ad8d0ff

## Summary of deliverable 1
- done: `imports` reads its fences through `tools::fenced` (`pub(crate)`); a REP-2 scenario states the inline-code case.
- red: REP-2: the fixed context is 65 bytes, CLAUDE.md 55 (expected 1165 and 1155: the tab-indented fence and the
  inline-code line hid both imports).
- checks: project-check green, docs unchanged (no doc names this grammar), deliverable-size 56 lines.
- Code review: no findings
- not verified: that Claude Code's own scanner treats a tab-indented fence as no fence (read from `marked`'s rule).
