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
