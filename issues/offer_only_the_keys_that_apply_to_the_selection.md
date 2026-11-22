---
priority: medium
depends_on: [show_the_messages_of_the_ui_out_of_the_help_line]
why: 'the help line lists keys that do nothing on most selections; it reshapes the line the messages leave'
---
# Offer only the keys that apply to the selection

# Goal
The help line of `cardinal-ui` lists every key whatever is selected: `f merge` on an issue that is not `merge
pending`, `s start` on one already taken, `e end` on a worktree whose branch is not merged. The user reads commands
that would only be refused. The expected outcome is a help line that offers the actions the selection allows, plus
the keys that always work.

# Findings
* User feedback of 2026-11-22: the commands should be contextual; no use offering a merge on an issue that is not
  ready to merge.
* `crates/cardinal-ui/src/screen.rs`: the help line is a constant, `HELP`
  (`q quit  r refresh  ↑↓ select  tab panel  s start  f merge  e end  E end merged  v release  m measure  g charts`).
* The conditions already exist: SUA-1 (`s`: `ready`, nothing takes it, in zellij), SUA-2 (`f`: a claim `merge
  pending`), SUA-4 (`v`: a milestone awaits its release), SUE-1 (`e`: a linked worktree merged), SUE-2 (`E`: a row
  `merged`).

# Approach
1. Build the help line at each draw from the panel and the line selected, with the conditions of SUA and SUE.
2. Keep the key refusing as today when pressed out of context (a key pressed just before a read changed the
   state), so the help line is a hint, never the guard.
3. Decide whether `E` and `v`, which do not depend on the selection, show only when a worktree is merged or a
   milestone awaits its release.

# Specs
* `specs/steering-ui.md` (the help line), `specs/steering-ui-actions.md` and `specs/steering-ui-end.md` if the
  conditions are named there.

# Milestone
A steering UI to read at a glance

# Resources
* `crates/cardinal-ui/src/screen.rs` (`HELP`, `help_line`), `crates/cardinal-ui/src/keys.rs`
