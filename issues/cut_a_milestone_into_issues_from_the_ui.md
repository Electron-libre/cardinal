---
priority: medium
depends_on: [rethink_the_commands_of_the_ui]
why: 'turning a shaped milestone into issues is a steering step done from another terminal today; one more command for the new command system'
---
# Cut a milestone into issues from the UI

# Goal
A milestone of the roadmap becomes issues through `skill:create-issue` ("Cut a shaped milestone into issues"),
started by hand in a `claude` session. The expected outcome is a command of `cardinal-ui` that, on the selected
milestone, starts that session in a zellij tab of the main worktree.

# Findings
* `skill:create-issue` has the procedure "Cut a shaped milestone into issues": for a `shaped` milestone, or an
  `idea` one whose goal is agreed.
* The selection moves over the issue rows only, never on a heading (STU-14): a milestone cannot be selected today.
* `v` already starts a skill session in a zellij tab of the main worktree (SUA-4, SUA-11, SUA-12).

# Approach
1. Let the selection rest on a milestone heading (amending STU-14), or pick the milestone in the menu or palette of
   `rethink_the_commands_of_the_ui`.
2. The command starts `claude "/create-issue cut the milestone <name> into issues"` in a new zellij tab named after
   the milestone, as SUA-4 does; refused (status line, SUS-2) outside zellij, on a milestone that is neither `shaped`
   nor `idea`, or while that tab is open.
3. The help line or menu offers it only on such a milestone (SUA-15).

# Specs
* `specs/steering-ui.md`: STU-14 (selection), STU-3 (what the UI starts); `specs/steering-ui-actions.md`: a new
  requirement (the command, its refusals), SUA-15.

# Milestone
A steering UI to command

# Resources
* `.claude/skills/create-issue/SKILL.md`, `crates/cardinal-ui/src/release.rs`, `crates/cardinal-ui/src/keys.rs`,
  `ROADMAP.md` (milestone states).

# Hints
* An `idea` milestone needs its goal agreed by the user: the session asks, the UI does not decide.
