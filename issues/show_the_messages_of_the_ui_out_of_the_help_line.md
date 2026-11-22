---
priority: high
why: 'an error the user does not see leaves a refused merge or end unnoticed: the milestone is about reading the screen at a glance'
---
# Show the messages of the steering UI out of the help line

# Goal
`cardinal-ui` says what an action did or why it refused (a merge, an end, a start, a notification that failed, an
ignored theme line) in the status message, drawn at the start of the help line, in `text` on `surface`. There the
user misses it, an error looking like any other message. The expected outcome is a message area that stands out,
an error told from a success at a glance.

# Findings
* User feedback of 2026-11-22: the error and other messages are not visible enough; they should not be in the
  command bar.
* `crates/cardinal-ui/src/screen.rs` (`help_line`): the status message first, then the help in `muted`, both on
  `surface`; STU-31 and the theme table of `specs/steering-ui.md` give the status message the role `text`.
* The messages come from SUA (start, merge, release), SUE (end), STU-30 (notify) and STU-33 (theme).

# Approach
1. Give the messages a line or a box of their own, apart from the help line (above it, or a popup dismissed by a
   key or after a delay).
2. Tell an error from a success: a role for each (e.g. an `error` role in the theme, with a neutral default).
3. Keep the last message until the next action or a key, so a read of STU-2 never wipes it before it is seen.

# Specs
* `specs/steering-ui.md`: STU-31 and the theme table (the status message, a new role), STU-32 if the theme gains a
  role; the scenarios of SUA and SUE that say "the status line reads" keep their texts.

# Milestone
A steering UI to read at a glance

# Resources
* `crates/cardinal-ui/src/screen.rs` (`help_line`, `render`), `crates/cardinal-ui/src/theme.rs`
* `docs/usage.md` (the theme roles)

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
The status message leaves the help line for a row of its own above it, on `surface`: an error in `fail`, a success in
`pass`, a running task in `text`, each message of the line in its role (STU-34). It already survives the reads of STU-2.
Means: the roles `fail` and `pass` of the theme, no new role (STU-32 unchanged); the row only while a message is shown.

## Plan
1. [x] Status line of its own: a `Message` kind (error, done, other) kept by the screen loop from the task results, the refusals of the keys, the notifier and the theme lines, drawn above the help line on the panels and the charts; verified by render tests of the row, its roles and a read that leaves it; makes true STU-34 (STU-31 amended). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: cb54bbf94344d218bb56d973d9cb88f327f85d7d
- Colors: the roles `fail` and `pass`, no new role (answer 1 a).
- Row: drawn only while a message is shown (answer 2 a).

## Deliverable 1
done: the status line has a row of its own above the help line (panels and charts), only while it shows a message;
an error in `fail`, a task done in `pass`, the others in `text`, each message of the line in its role (`status.rs`).

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (STU-34 traced, `steering-ui.md` accepted) |
| docs | `docs/usage.md`, `docs/architecture.md` |
| code review | `low`: no findings |

red: STU-34: `left: (Rgb(0, 0, 4), Rgb(0, 0, 2)) right: (Rgb(0, 0, 8), Rgb(0, 0, 2))` (a success drawn in `text`)

Code review: no findings

not verified: that a read of STU-2 leaves the message (the terminal loop needs a real terminal; no read touches the
status, by reading of `screen::run`); the screen in a real terminal.

decisions needed: none
