---
priority: medium
why: 'a panicking task leaves the screen stuck until the process is killed; rare, but nothing recovers from it'
---
# End a task that panics in the UI

# Goal
When the task of a key (`s`, `f`, `e`, `E`, `v`, `p`, `c`) panics on its thread, `cardinal-ui` never learns it ended:
the screen stays busy, no other task starts, and `q` only waits for the task, so the user must kill the process. The
expected outcome is a panicking task reported on the status line as an error, the screen free again, and `q` quitting.

# Findings
* Found by the `high` review of `issues/done/split_the_screen_module_of_the_ui.md`, read from the code, not reproduced.
* `run` of `crates/cardinal-ui/src/terminal.rs` spawns `std::thread::spawn(move || send.send(run_task(job, &stem)))`:
  a panic in `run_task` sends nothing, so `busy` stays `true`.
* The loop keeps the `done` sender, so `task_result.try_recv()` never reports `Disconnected` either.
* `quits` of `keys.rs` returns `(false, true)` on `q` while busy: the user waits for a result that never comes.
* Ratatui's panic hook runs on the panicking thread and restores the terminal while the main loop keeps drawing.

# Approach
1. Wrap `run_task` in `std::panic::catch_unwind` on its thread and send the panic as an `Err` line.
2. Or detect the dead thread: give each task its own channel and treat `Disconnected` as a failed task.
3. Check what Ratatui's panic hook does to the terminal when the panic is caught, and keep the screen usable.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/terminal.rs` (`run`), `crates/cardinal-ui/src/keys.rs` (`quits`, `ended`).
* `specs/steering-ui-actions.md` (the tasks of the keys and their results on the status line).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
A task of a key that panics ends as a failed task: `the task panicked: <message>` on the status line, the keys free,
`q` quitting, the screen left drawn. Out of scope: a panic of the thread of the reads (it already quits). Means: approach
1 (`catch_unwind` on the task thread), plus a panic hook that skips the named task threads so Ratatui's restore and the
panic message never reach the screen; the requirement goes to `specs/steering-ui-screen.md` (SUS-7), the actions spec
being at its size bound.

## Plan
1. [x] Catch the panic of a task: the task thread named and run under `catch_unwind`, its panic sent as the `Err` line,
   the panic hook of `run` passing over that thread; verified by tests of `terminal.rs` (a panicking task gives its
   line on the channel, the hook leaves a task thread to nobody); makes true SUS-7. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 6c58f8bf622af40f40b7f84df315bec6b4bc4000
- Spec: SUS-7 in specs/steering-ui-screen.md (1 a)

## Summary of deliverable 1
- done: a task thread is named, runs under `catch_unwind` and sends its panic as `the task panicked: <message>`; the
  panic hook installed after Ratatui's passes over that thread; a thread that cannot spawn says so on the status line.

  | check | result |
  |---|---|
  | tests | pass (`cardinal run project-check`) |
  | lint | pass (clippy -D warnings) |
  | specs | SUS-7 traced, `steering-ui-screen.md` accepted |
  | docs | `docs/usage.md` (status line), `docs/architecture.md` (`terminal.rs` row) |
  | code review | low, clean |
- red: SUS-7: `left: Err(Disconnected)`, `right: Ok(Err("the task panicked: boom on foo"))`
- Code review: no findings
- not verified: the screen in a real terminal with a panicking task (no manual run).
- decisions needed: none.
