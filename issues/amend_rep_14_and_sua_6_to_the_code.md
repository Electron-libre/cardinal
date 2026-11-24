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
