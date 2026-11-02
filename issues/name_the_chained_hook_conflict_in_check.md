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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-02

## Understanding
`check` prints `conflict: native review gate (...)` instead of `missing: native review gate` when a foreign `pre-commit`
and a `pre-commit.user` (a dangling link included) coexist, naming the manual merge; `plan` and `problem` share one
test of the conflict so they never disagree. Out of scope: changing what `sync` does (SYN-19 stays).

## Plan
1. [x] Conflict line in `check`: a `State`-level conflict test shared by `plan` and `problem`, a test in
   `gate_cli.rs` with a regular and a dangling `pre-commit.user`; verified by that test; makes true SYN-23 (amended). (autonomous)

## Deliverable 1
- done: `check` prints the conflict line of SYN-23; `gate::conflicts` is the one test `plan` and `problem` share.
- red: SYN-23: `left: ["missing: native review gate"] right: ["conflict: native review gate (...)"]`
- checks: `just check` ok; deliverable size 53 lines, within.
- Code review: no findings
- not verified: none.


## Decisions
- Mode: autonomous
- Base commit: 599d3e26eae163abd185f1964695f1193406d0b4
- Conflict line: the text of SYN-23 (1 a), so the next `sync` installs the gate and chains `pre-commit.user`, moving nothing.
