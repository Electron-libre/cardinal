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
* Found by the `high` review of `issues/unify_the_fence_grammars_of_the_markdown_scanners.md` (2026-11-04).
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
