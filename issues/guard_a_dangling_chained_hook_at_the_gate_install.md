---
priority: low
why: 'a dangling link named pre-commit.user is rare, but the install then breaks the promise never to overwrite a hook of the user'
---
# Guard a dangling chained hook at the gate install

# Goal
`gate::plan` refuses to install the gate over a `pre-commit` of the user when `pre-commit.user` already exists
(SYN-19), but it tests it with `exists()`, which follows links: a dangling `pre-commit.user` link passes. The install
then renames the user's `pre-commit` over it, or fails at `symlink()` after the new hook was staged. Expected: a
`pre-commit.user` of any kind, a dangling link included, stops the install changing nothing.

# Findings
* `crates/cardinal/src/gate.rs`, `plan`: `hooks.join(CHAINED).exists()`; the removal of SYN-60 reads the same path with
  `symlink_metadata` (found by the `high` review of `turn_off_the_review_gate`).

# Approach
1. Write the failing test of SYN-19 with a dangling `pre-commit.user`, then read it with `symlink_metadata`.

# Specs
* `specs/sync.md` SYN-19 (a scenario).

# Resources
* `crates/cardinal/src/gate.rs`, `crates/cardinal/tests/gate_cli.rs`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-02

## Understanding
`gate::plan` must treat a `pre-commit.user` of any kind, a dangling link included, as existing (`symlink_metadata`),
so the install over a user's `pre-commit` fails changing nothing. Out of scope: the other `exists()` of the gate.

## Plan
1. [x] Refuse a dangling `pre-commit.user`: a `gate_cli` test (a dangling link, `sync` fails, hooks unchanged), then
   `plan` reads it with `symlink_metadata`; verified by that test; makes true SYN-19 (its new scenario). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 14775aee4312628a57660dc70b14f5676edf9075

## Deliverable 1
- done: `gate::plan` refuses a `pre-commit.user` of any kind (`symlink_metadata`), a dangling link included.
- checks: tests, lint, specs pass (`just check`); docs n/a (`docs/gates.md` already promises it).
- red: SYN-19: `sync` succeeded (`code: Some(0)`) and moved the user's `pre-commit` over the dangling link.
- Code review: no findings
