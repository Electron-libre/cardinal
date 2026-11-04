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
* Found by the `high` review of `issues/done/read_the_fences_of_memory_imports_with_the_shared_grammar.md` (2026-11-04).
* On ``` ``a`b`` @x ```, `imports` reads one span then `@x`; `split('`')` pairs the backticks otherwise.

# Approach
1. Move the fence functions to `crates/cardinal/src/markdown.rs`, alone in a deliverable (a move counts twice).
2. Add one code-span function there (CommonMark: runs of equal length) and read `imports`, `prose_words` and `hidden`
   through it.

# Resources
* `crates/cardinal/src/tools/mod.rs`, `crates/cardinal/src/memory.rs`, `crates/cardinal/src/tools/language.rs`

# Hints
* Step 2 changes what QUA-12 and the language check read: check whether their specs name the code-span rule.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-04

## Understanding
One `crate::markdown` module holds the fence grammar (with `hidden` and its paragraph helpers) and one code-span
reader (CommonMark runs of equal length); the core (`memory.rs`) no longer imports `crate::tools`. Five readers of code
spans exist (`imports`, `close_issue::code_spans`, `check_specs::without_code_spans`, `prose_words`, `hidden`): all go
through the shared one, so `prose_words` and `hidden` stop pairing single backticks.

## Plan
1. [x] Move: `Fence`, `fence_marker`, `next_fence`, `fenced`, `unfenced`, `hidden`, `breaks_paragraph`, `is_list_item`, `is_heading` from `tools/mod.rs` and `code_spans` from `close_issue.rs` to `crates/cardinal/src/markdown.rs`, callers updated, no change of behaviour; verified by `just check`. (autonomous)
2. [x] Read `imports`, `without_code_spans`, `prose_words` and `hidden` through `markdown::code_spans`; verified by tests through the binary (a two-backtick span holding a backtick: not prose for the language check, a `<!--` in it opens no roadmap comment); makes true the amended BRE-10, CLR-5 and RMP-1. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: ed375d6dd71e383ef148c8d09e7c392ed4896234
- Specs amended (still `accepted`, ids kept): BRE-10 and CLR-5 name the CommonMark code-span rule, RMP-1 says a `<!--` in a code span opens no comment; one scenario each for BRE-10 and RMP-1.

## Deliverable 1
- done: the fence grammar, `hidden` and its helpers, and `code_spans` moved to `crates/cardinal/src/markdown.rs`; callers read `crate::markdown`; the spec amendments of deliverable 2 committed with it.
- checks: `cardinal run project-check` passes; `deliverable-size` 367 lines, within.
- red: none (a move, no new behaviour)
- Code review: no findings
- not verified: none; decisions needed: none

## Deliverable 2
- done: `markdown::without_code_spans` (over `code_spans`) replaces the backtick split of `hidden` and `prose_words` and the reader of `check_specs`; `imports` skips the `@` inside `code_spans`.
- checks: `cardinal run project-check` passes.
- red: LNG-4, BRE-10, RMP-1: the hook sent the reply back (`code: Some(2)`, "Your last reply is not in French") for an English code span holding a backtick; `roadmap` read the state of `Next` as `(none)` after a line ``` ``<!--`` opens a comment ```.
- Code review: no findings
- not verified: none; decisions needed: none
