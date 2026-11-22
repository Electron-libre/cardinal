---
priority: low
why: 'a shortcut: the next issue is already marked `> ` and the arrows reach it'
---
# Select the next issue with a key

# Goal
The queue marks the issue `next-issue` offers with `> ` (SUQ-3), but reaching it takes the arrow keys. The expected
outcome is one key that moves the selection to that issue, so `s` can start it right after.

# Findings
* The `next` of the document of NXT-19 names the offered issue; it is `null` when none is ready (NXT-12).
* The selection moves over the issue rows only and keeps its issue across reads (STU-14).

# Approach
1. A key (`n`?) selects the issue of `next` in the queue, switching to the queue panel if the worktrees one has the
   focus; with no `next`, nothing moves and the status line (STU-34) says why.
2. Optionally the screen opens on that issue instead of the first of the queue.
3. The help line (SUA-15) offers it when `next` is not the selected issue.

# Specs
* `specs/steering-ui.md`: STU-14 (the selection); `specs/steering-ui-actions.md`: SUA-15.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/keys.rs` (`action`, `select`), `crates/cardinal-ui/src/help.rs`.

# Hints
* Check the key against those taken (`q r j k s f e E v g`) and against `rethink_the_commands_of_the_ui`.
