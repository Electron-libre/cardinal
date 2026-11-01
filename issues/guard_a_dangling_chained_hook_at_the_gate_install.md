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
