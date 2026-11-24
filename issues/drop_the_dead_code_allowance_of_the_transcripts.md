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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
Remove the module-wide `#![allow(dead_code)]` of `transcripts.rs` so the compiler flags any item that loses its last
caller. A probe build and clippy (all targets) without it report no warning: no item needs `#[cfg(test)]`.
Out of scope: any other change of the module.

## Plan
1. [x] Drop the allowance: delete `#![allow(dead_code)]` of `crates/cardinal/src/tools/transcripts.rs`; verified by
   `cardinal run project-check` (clippy -D warnings) passing with no warning; no spec requirement (pure refactor). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 62562d9141b4a6c6be8377d93e564093604c87d4

## Summary of deliverable 1
- done: the module-wide `#![allow(dead_code)]` of `transcripts.rs` is gone; no item was dead.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | clippy -D warnings clean |
| specs | unchanged (pure refactor) |
| docs | nothing to update |

- red: none (no test: the compiler is the check)
- Code review: no findings
- not verified: nothing
- decisions needed: none

## Whole issue
- update-docs on `62562d9..HEAD`: nothing to update.
- Code review (high): no findings.
- Done: nothing left.
