---
priority: low
why: 'same failure as a panicking task, on threads that hold no change of the project; rare'
---
# End a measure or a notification that panics in the UI

# Goal
A measure started by `m` or a desktop notification that panics on its thread breaks the screen of `cardinal-ui`:
Ratatui's panic hook gives the terminal back and prints over the screen while the loop keeps drawing, and a measure
that panics leaves its `running` flag set, so `m` never measures again. The expected outcome is the panic reported as
a failure (a measure on the status line, a notification as a failed one of STU-30), the screen left drawn and `m`
measuring again.

# Findings
* Found by the `high` review of `issues/done/end_a_task_that_panics_in_the_ui.md`, read from the code, not reproduced.
* `crates/cardinal-ui/src/measures.rs` starts the measure on an unnamed `std::thread::spawn` with no `catch_unwind`.
* `crates/cardinal-ui/src/notify.rs` sends each notification on an unnamed `std::thread::spawn` with no `catch_unwind`.
* `crates/cardinal-ui/src/terminal.rs` already handles a task this way: `spawn_task` (named thread, `catch_unwind`),
  `TASK_THREAD` and `pass_over_tasks` (SUS-7).

# Approach
1. Reuse the mechanism of the tasks: run these threads under `catch_unwind`, on a thread name the panic hook of
   `pass_over_tasks` passes over, and turn the panic into the failure each one already reports.
2. Reset the `running` flag of a measure on its panic.

# Specs
* `specs/steering-ui-screen.md` (SUS-7) or `specs/steering-ui.md` (STU-17, STU-30): extend the requirement of a panic.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/terminal.rs` (`spawn_task`, `pass_over_tasks`), `measures.rs`, `notify.rs`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
A measure or a notification that panics ends as a failed one (the `Measures` panel line, the `notify:` status line),
the screen kept drawn and `m` measuring again (SUS-8). Out of scope: the panic of the reads (a non-goal of SUS).
Means: the caught thread of the tasks moves to a module `panics.rs` (`spawn` a named thread under `catch_unwind`, the
hook passing over the first panic of its threads), used by the tasks, the measures and the notifications.

## Plan
1. [x] Caught threads for the measures and the notifications: `panics.rs` holds the thread spawner and the hook taken
   from `terminal.rs`; `Measurer` and `Notifier` run their job through it, the job injectable for the tests; verified by
   internal tests (a panicking measure leaves `the measure panicked: boom` and `running` false, then starts again; a
   panicking notification gives `notify: the notification panicked: boom` once; neither reaches the hook of the
   terminal); makes true SUS-8. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 2b6dfcd3ca52c3f06391bc6a7d2b47632b0b13b3
- Measure failure shown in the `Measures` panel (STU-19), not on the status line (1 a).

## Deliverable 1 summary
- done: `panics.rs` runs the tasks, the measures and the notifications on named threads under `catch_unwind`, their
  panic given as the failed line, the hook passing over them; `Measurer` and `Notifier` use it.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (SUS-8 traced, spec `accepted`) |
| docs | `docs/architecture.md`, `docs/usage.md` |
| code review | no findings |

- red: SUS-8: `left: (true, None) right: (false, Some("the measure panicked: boom"))` and `no failure shown`
- Code review: no findings
- not verified: a panic on the real screen of `cardinal-ui` (needs a terminal).
- decisions needed: none
