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
