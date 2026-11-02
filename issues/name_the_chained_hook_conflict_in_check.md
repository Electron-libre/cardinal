---
priority: low
why: 'a conflict on pre-commit.user is rare, but check then sends the user to a sync that always fails'
---
# Name the chained hook conflict in check

# Goal
When the project has a `pre-commit` of the user and a `pre-commit.user` (a dangling link included), `sync` refuses to
install the gate (SYN-19), yet `check` only reports `missing: native review gate`, which `sync` cannot fix. Expected:
`check` names the conflict and what to merge by hand.

# Findings
* `crates/cardinal/src/gate.rs`, `problem`: `State::Missing | State::Foreign` both print `missing: {GATE}`; `plan`
  bails on `Foreign` with a `pre-commit.user` (found by the `high` review of
  `guard_a_dangling_chained_hook_at_the_gate_install`).

# Approach
1. Write the failing test of SYN-23 with both hooks present, then share the test of `plan` with `problem`.

# Specs
* `specs/sync.md` SYN-23 (amend: the line printed on the conflict), SYN-19.

# Resources
* `crates/cardinal/src/gate.rs`, `crates/cardinal/tests/gate_cli.rs`.
