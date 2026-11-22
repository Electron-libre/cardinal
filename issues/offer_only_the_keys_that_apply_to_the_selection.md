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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
The help line under the panels lists the actions the current selection allows (`s`, `f`, `e`) and the global ones
when they apply (`E`, `v`), plus the keys that always work; the keys keep refusing when pressed out of context.
Out of scope: the help lines of the charts and of the measures, `v` for a release prepared but not tagged.

## Plan
1. [x] The contextual help line: the read gives the view what each key needs (per issue: may start, may merge; per
   worktree: merged; any worktree merged; a milestone awaiting its release; in zellij), a new module `help.rs` builds
   the line from the view and the selection, the screen draws it at each draw, `--print` with the selection it starts
   on; verified by unit tests of `help.rs` and the `--print` tests of `tests/print_help.rs`; makes true SUA-15. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 6aeed8c64dc9933070157b5f50b59b7ebb28d4a4
- `s` and `v` hidden outside a zellij session (they would be refused).
- `E` and `v` shown only when a row reads `merged` or a milestone awaits its release.
- `v` not offered for a release prepared but not tagged (non-goal of SUA).

## Deliverable 1
- done: the help line under the panels is built at each draw from the selection and the keys the read allows
  (`help.rs`); `--print` shows the one of the first issue of the queue.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (SUA-15, SUA-H7 traced, `steering-ui-actions` accepted) |
| docs | `docs/usage.md`, `docs/architecture.md`, `README.md`, `screen.rs` module doc |
| code review | see below |

- red: SUA-15: `left: "q quit  r refresh  ↑↓ select  tab panel  s start  f merge  e end  E end merged  v release  g charts"`
  `right: "q quit  r refresh  ↑↓ select  tab panel  g charts"`
- Code review: no findings
- not verified: the help line in a live terminal (drawn by the same `render` the `--print` tests read)
- decisions needed: none
