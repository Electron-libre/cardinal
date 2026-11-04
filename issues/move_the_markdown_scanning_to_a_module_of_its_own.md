---
priority: low
why: 'a comfort: the scanners agree on every known consumer file; this removes a dependency of the core on tools and a second code-span grammar'
---
# Move the Markdown scanning to a module of its own

# Goal
The fence grammar (`Fence`, `fence_marker`, `next_fence`, `fenced`, `unfenced`, `hidden`) lives in `tools/mod.rs`,
and `memory.rs`, used by `check`, `sync` and `update` (the budget), now imports it from `crate::tools`. The code spans
are read two ways: `memory.rs::imports` matches backtick runs of equal length, `language.rs::prose_words` and
`hidden` split on single backticks, so the same line yields different prose. A `crate::markdown` module would hold
both grammars once, the core no longer depending on the tools.

# Findings
* Found by the `high` review of `issues/read_the_fences_of_memory_imports_with_the_shared_grammar.md` (2026-11-04).
* On ``` ``a`b`` @x ```, `imports` reads one span then `@x`; `split('`')` pairs the backticks otherwise.

# Approach
1. Move the fence functions to `crates/cardinal/src/markdown.rs`, alone in a deliverable (a move counts twice).
2. Add one code-span function there (CommonMark: runs of equal length) and read `imports`, `prose_words` and `hidden`
   through it.

# Resources
* `crates/cardinal/src/tools/mod.rs`, `crates/cardinal/src/memory.rs`, `crates/cardinal/src/tools/language.rs`

# Hints
* Step 2 changes what QUA-12 and the language check read: check whether their specs name the code-span rule.
