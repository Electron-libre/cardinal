---
priority: high
why: 'two requirements contradict the code and another requirement: a test written from them would expect the wrong behaviour'
---
# Amend REP-14 and SUA-6 to the code

# Goal
Two requirements no longer state what the code does: REP-14 refuses a `--by` value that REP-20 and the code accept,
and SUA-6 leaves the audit out of the starts that block the other keys and that `q` waits for. The outcome is both
requirements amended to the behaviour of the code, each proved by a traced test.

# Findings
* Code audit of 2026-11-24.
* `specs/cardinal-report.md:37` (REP-14): "`--by revision` ... or `--by setup`, and refuse any other value";
  `specs/cardinal-report.md:43` (REP-20) requires `--by milestone`, which
  `crates/cardinal/src/tools/cardinal_report.rs:963` accepts.
* `specs/steering-ui-actions.md:32` (SUA-6) lists the starts of an issue, the release, the priorities review and a
  cut, and the keys `s`, `f`, `e`, `c`, `E`, `v`, `p`; the audit (`a`, SUA-24 at `specs/steering-ui-actions.md:44`)
  is handled as the other tasks by `crates/cardinal-ui/src/keys.rs:171,189` and
  `crates/cardinal-ui/src/terminal.rs:227`.

# Approach
1. Amend REP-14 to accept `revision`, `setup` or `milestone` (REP-20).
2. Amend SUA-6 to name the audit among the starts and `a` among the keys; check a test proves `a` is refused while
   a task runs, else add one.

# Specs
* `specs/cardinal-report.md`: REP-14. `specs/steering-ui-actions.md`: SUA-6.

# Milestone
Specs and docs true to the code

# Resources
* `skill:write-spec`, `crates/cardinal/tests/cardinal_report.rs`, `crates/cardinal-ui/src/keys.rs`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
REP-14 and SUA-6 amended to what the code does: `--by milestone` accepted, the audit (`a`) among the starts that
block the other keys and that `q` waits for. No change of behaviour. The guard of the keys while a task runs lives in
the loop of `terminal.rs`, untested: it moves to a function of `keys.rs` that a test drives with every start key.

## Plan
1. [x] Amend REP-14 and SUA-6; extract the busy guard of the loop of `terminal.rs` into a function of its own (no change of behaviour);
   verified by a test that `s`, `f`, `e`, `c`, `E`, `v`, `p` and `a` start nothing and show nothing while a task
   runs, REP-14 by the checks of `setup_checks` (unknown or missing `--by` refused) and the `--by milestone` tests of
   REP-20; makes true REP-14, SUA-6. (autonomous)
2. [x] Docs of the whole issue and fixes of its `high` review: `docs/usage.md` names `a` among the keys that act
   whatever the selection and in the palette, and prints the result of any task quit during; the docs of
   `terminal.rs` name `p`, `c` and `a`; a redundant import dropped; verified by `project-check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 99b20e6db1c9f6cc58479bc9fafa5281a929e636
- REP-14 proved by the existing checks of `setup_checks` and the REP-20 tests, no new test (1 a).

## Summary of deliverable 1

done: REP-14 accepts `--by milestone`, SUA-6 names the audit and `a`; the busy guard of the loop is `launch` in
`terminal.rs` (not `keys.rs`, which it would have pushed over 800 lines), `docs/usage.md` names `a`.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass, both `accepted` again |
| docs | `docs/usage.md` |
| code review | see below |

red: SUA-6: cannot find function `launch` in this scope

Code review: no findings

not verified: none
decisions needed: none

## Summary of deliverable 2

done: the docs of the issue and the fixes of the `high` review (`docs/usage.md`, the doc comments and an import of
`terminal.rs`).

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | `docs/usage.md` |
| code review | see below |

red: none

High review: 4 findings: the keys `a` missing in `docs/usage.md` (fixed) and in the docs of `terminal.rs` (fixed),
a redundant import (fixed), the type `docs` of 21d7d30 for a change that also moves code (not fixed: retyping it
rewrites the history, not asked).

Code review: no findings

not verified: none
decisions needed: none
