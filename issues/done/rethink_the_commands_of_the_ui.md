---
priority: medium
why: 'one letter per action does not scale with the commands this milestone adds, and the help line already crowds'
---
# Rethink the commands of the UI

# Goal
Every action of `cardinal-ui` is a single letter (`s f e E v g`, plus `q r tab`), found only in the help line
(SUA-15), which lists them on one row. The user finds this system impractical. The expected outcome is a way of
giving commands that is easy to discover and to use as commands are added.

# Findings
* The user, on 2026-11-23: the command system "n'est pas des plus pratique"; leads: a command palette, a menu to
  show and hide, context menus.
* The help line already shows up to ten keys and hides those that would be refused (SUA-15).
* Issues filed the same day add commands: `select_the_next_issue_with_a_key`, `prioritize_the_issues_from_the_ui`,
  `fold_the_summary_of_the_selected_issue`.

# Approach
Measure first: list the commands, how often each is used and what the user misses (ask the user), then compare:
1. A command palette (`:` or `Ctrl+P`): a filtered list of every command that applies, typed by name.
2. A menu shown and hidden by one key (`?` or `m`), listing the commands with their key.
3. A context menu (`Enter` on a selected issue or worktree) offering what applies to it.
4. Combinations: single letters kept as shortcuts, the palette or menu for discovery.
Cut the chosen design into issues if it exceeds the size limit of `skill:issue-workflow` step 1.

# Specs
* `specs/steering-ui-actions.md`: SUA-15 and the keys of SUA-1, SUA-2, SUA-4; `specs/steering-ui-end.md`: SUE-1,
  SUE-2; `specs/steering-ui-charts.md`: SUC-1.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/keys.rs`, `crates/cardinal-ui/src/help.rs`, `crates/cardinal-ui/src/screen.rs`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
A command palette on `:` lists the commands that act on the selection (same conditions as the help line), filtered by
typing their name, `Enter` runs one as its key would; the single letters stay as shortcuts. The help line shrinks to
`: commands`, the moves and the keys of the selection. Out of scope: a context menu, new key bindings, fuzzy ranking.

## Plan
1. [x] The palette (`crates/cardinal-ui/src/palette.rs`): the commands of SUP-1 from those of the help line
   (`help::offered`), the filter, the highlight and the keys inside it, `:` opening it over the panels, `Enter` running
   the action of the highlighted command through the path of its key, `Esc` closing it, nothing on the charts;
   `screen.rs` (over the file size limit, its split is `split_the_screen_module_of_the_ui`) only gets the dispatch;
   verified by the unit tests of `palette.rs`; makes true SUP-1 to SUP-5, SUP-H1, SUP-H2. (autonomous)
2. [x] The shorter help line: `: commands  ↑↓ select  tab panel  q quit` then `i`, `s`, `f`, `e` of the selection, the
   global keys moved to the palette, with `README.md`, `docs/usage.md` and `docs/architecture.md`; verified by the
   tests of `help.rs` and `--print`; makes true the amended SUA-15. (autonomous)
3. [x] The fixes of the `high` review: a letter held with `Ctrl` or `Alt` not typed, the list scrolled to keep the
   highlight drawn, STU-2 naming the palette, the palette spec linked from `docs/architecture.md`, the test of
   `Keys::of` traced to SUP-1; verified by the unit tests of `palette.rs`; makes true the amended SUP-2. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 4f63e1113ce4df9f81420bfcd524d9a56a9613e6
- Design: a palette on `:` with the single letters kept (1 a); the help line keeps `: commands`, the moves, `q` and
  the keys of the selection (2 a); the palette lists only the commands that act (3 a); no command reported missing (4 a).
- The `high` review of `4f63e11..5b0c410` gave 5 findings, all nits, fixed in deliverable 3.
- Deliverables 1 and 2 of the validated plan merged into one (about 640 lines): the model alone left dead code that
  `clippy -D warnings` refuses.

## Summary of deliverable 1
done: `:` opens a command palette over the panels, filtered by name, `Enter` runs the highlighted command as its key.

| check | result |
|---|---|
| tests | `just check` green, 10 new tests in `palette.rs` |
| lint | clippy `-D warnings` clean |
| specs | SUP-1 to SUP-5, SUP-H1, SUP-H2 traced (internal: the screen needs a real terminal) |
| docs | `README.md`, `docs/usage.md`, `docs/architecture.md`, `.claude/update-docs-map.md`, `main.rs` |
| code review | `low`: no findings |

red: SUP-1, SUP-2, SUP-3, SUP-4, SUP-5, SUP-H1, SUP-H2: `assertion left == right failed` (an empty list of commands,
an `Action::Nothing` instead of `Action::Start`, a blank screen with no palette drawn)

Code review: no findings

not verified: the palette in a real terminal (drawn only through the `TestBackend`).
decisions needed: none. `screen.rs` grows to 868 lines (limit 800): its split is the issue
`split_the_screen_module_of_the_ui`.

## Summary of deliverable 2
done: the help line reads `: commands  ↑↓ select  tab panel  q quit` then `i`, `s`, `f`, `e` of the selection; `E`, `v`,
`p`, `r` and `g` are found in the palette; the specs SUA and SUP set `accepted`.

| check | result |
|---|---|
| tests | `just check` green; `help.rs`, `screen.rs` and `tests/print_help.rs` on the new line; a new test of the flags of `Keys::of` |
| lint | clippy `-D warnings` clean |
| specs | SUA-15 amended and traced; SUA and SUP `accepted` |
| docs | `README.md`, `docs/usage.md`, `docs/architecture.md` |
| code review | `low`: no findings |

red: SUA-15: `assertion left == right failed`, left `q quit  r refresh  ↑↓ select  tab panel  i summary  s start  E end
merged  g charts`, right `: commands  ↑↓ select  tab panel  q quit  i summary  s start`

Code review: no findings

not verified: none.
decisions needed: none.

## Summary of deliverable 3
done: the 5 findings of the `high` review of `4f63e11..5b0c410` fixed (all nits).

| check | result |
|---|---|
| tests | `just check` green; 2 new tests in `palette.rs` |
| lint | clippy `-D warnings` clean |
| specs | SUP-2 amended (modifiers, scroll) and STU-2 (the palette) |
| docs | `docs/architecture.md`, `docs/usage.md` |
| code review | `low`: no findings |

red: SUP-2: `assertion left == right failed`, left `stwwR` right `stR`; the highlighted `g  charts` not drawn on a
screen 7 rows high

Code review: no findings

not verified: none.
decisions needed: none.
