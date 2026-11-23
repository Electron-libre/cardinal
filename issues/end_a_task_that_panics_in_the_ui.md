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
