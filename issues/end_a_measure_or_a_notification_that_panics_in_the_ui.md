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
