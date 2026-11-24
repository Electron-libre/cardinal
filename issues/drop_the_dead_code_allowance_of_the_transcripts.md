---
priority: low
why: 'cheap; it removes the only blind spot of the compiler for dead code in the source'
---
# Drop the dead code allowance of the transcripts

# Goal
`crates/cardinal/src/tools/transcripts.rs:5` carries `#![allow(dead_code)]` for the whole module, a leftover of the
time it was shared by separate scripts (ADR 0006). The tools now compile into one binary, so the compiler would
flag a function of the module that loses its last caller. The outcome: the allowance gone, no warning left.

# Findings
* Code audit of 2026-11-24: every item of the module has a caller today; it is the only `allow(dead_code)` outside
  the test helpers; `git log -S` traces it to the commit that printed per session metrics from the transcripts.

# Approach
1. Delete the line, run `just check`; mark `#[cfg(test)]` an item only the tests use, if any.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/transcripts.rs`, `docs/adr/0010-the-tools-ship-in-the-harness-binary.md`.
