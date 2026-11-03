---
priority: low
depends_on: [tell_the_needed_oversized_results_from_the_waste]
why: 'the session''s own oversized results cost 3.7 KB per session, many of them whole reads a range would bound'
---
# Hint before a whole read of a large file

# Goal
A third of the session's own tool results over 25,000 bytes are a `Read`, among them the `tool-results/`
file where Claude Code saved a result it had cut. A line in the fixed context would cost every session more than
the waste it saves; a hook costs nothing until it fires. The outcome: before a whole `Read` of a large file the agent
is told to read a range, and `oversized-result` falls.

# Findings
* Measured 2026-11-03 (`issues/done/tell_the_needed_oversized_results_from_the_waste.md`, deliverable 3): 36
  `oversized-result` breaches in 30 of 290 sessions, 3,708 bytes per session; over every session of the machine,
  69 such results: `Read` 24 (10 whole code files, 8 whole `tool-results/` files), `cat` 22, `cd …;` commands 8,
  `WebFetch` 7, others 8.

# Approach
1. A `PreToolUse` hook on `Read` with no `offset` nor `limit` of a file over 25,000 bytes: a hint (or an `ask`)
   naming the size and the range options, never a refusal.
2. Measure whether a hint on a `cat` of a large file through `Bash` is worth its parsing (`shell.rs`).
3. Declare the hook in `entries/.claude/settings.json` and give it a feature to turn it off.

# Specs
* A new spec of the hook.

# Milestone
Lean agent loop

# Resources
* `entries/.claude/settings.json`, `features.toml`, `crates/cardinal/src/tools/language_hint.rs` (a hook that hints).
* `specs/rule-breaches.md` BRE-19.

# Hints
* A file read whole may be needed (an edit of most of it): the hint must not block.
* Verdict: `Expected-effect: bytes_per_session:oversized-result down`.
