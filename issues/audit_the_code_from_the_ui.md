---
priority: low
depends_on: [write_a_skill_to_audit_the_code, rethink_the_commands_of_the_ui]
why: 'a shortcut to start the audit skill; the skill itself is what brings the value'
---
# Audit the code from the UI

# Goal
Once the skill of `write_a_skill_to_audit_the_code` exists, starting it still takes a `claude` session by hand. The
expected outcome is a command of `cardinal-ui` that starts the audit in a zellij tab of the main worktree.

# Findings
* `v` already starts a skill session in a zellij tab of the main worktree (SUA-4, SUA-11, SUA-12).

# Approach
1. The command starts `claude /<audit skill>` in a new zellij tab named `audit`, as SUA-4 does; refused (status line,
   SUS-2) outside zellij or while that tab is open.
2. Offered by the help line, menu or palette of `rethink_the_commands_of_the_ui`.

# Specs
* `specs/steering-ui-actions.md`: a new requirement (the command, its refusals), SUA-15; `specs/steering-ui.md`: STU-3.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/release.rs`, `crates/cardinal-ui/src/keys.rs`.
