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
   focus; with no `next`, nothing moves and the status line (SUS-2) says why.
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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
The key `n` moves the selection to the issue `next-issue` offers (focus to the queue), the status line saying why
when it offers none; the screen opens on that issue; the help line and the palette offer `n next` when it is not
selected. Out of scope: any other key or command of the UI.

## Plan
1. [ ] `n next`: `View` keeps the `next` of the read, `Action::Next` selects its row and focuses the queue (status
   line when none), `Selection::start` opens on it, `help::offered` lists `n next` (help line and palette); verified
   by unit tests of `keys`, `help`, `palette` and the `--print` test; makes true STU-36, STU-14, SUA-15, SUP-1.

## Decisions
- Mode: autonomous
- Base commit: cca5ba0b90c2e0834f636dbe3d7c0bb1af3124f2
- The screen opens on the issue `next-issue` offers (question 1 a).
- The palette lists `n next` too (question 2 a).

## Summary of deliverable 1

- done: `n` selects the issue `next-issue` offers (queue focused), the screen opens on it, the help line and the
  palette offer `n next` while it is not selected; `docs/usage.md` and `README.md` say so.
- checks: `just check` pass (tests, clippy, fmt, specs); docs updated; code review: see below.
- red: STU-36, STU-14, SUA-15, SUP-1: `no field next on type View`, `no variant Next`; the `--print` test: help line
  `… q quit  i summary` instead of `… i summary  s start` (the screen opened on `a`, not on `b`).
- `crates/cardinal-ui/src/screen.rs` stays over the file size limit (883 lines, 873 before): the overflow predates this
  deliverable, which adds only the `n` dispatch; its split is a move of its own.
- not verified: the key `n` in a real terminal (no interactive run).
