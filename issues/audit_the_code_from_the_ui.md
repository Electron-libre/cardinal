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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
The key `a` (palette `a audit`, flag `--audit`) opens a zellij tab `audit` in the main worktree running
`claude /audit-code`, with the refusals of `p prioritize` (zellij, free main worktree, skill present, tab open).
Out of scope: the audit itself (`skill:audit-code`), a help-line entry (palette only, like `p` and `v`).

## Plan
1. [x] `a` / `--audit` start the audit session: `audit.rs` on the model of `prioritize.rs` (shared `skill_tab`),
   the key in `keys.rs`, its palette entry in `help.rs`, the flag in `main.rs`, docs (`README.md`, `docs/usage.md`,
   `docs/architecture.md`); verified by `tests/audit.rs` (binary, fake `zellij` and `git`) and the unit tests of
   `help.rs`/`keys.rs`; makes true SUA-23, SUA-24, SUA-25, SUP-1, STU-3. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: c9b1121a5d9e85f87e03f4e8129822d6066e54b6
- Key `a`; missing skill says `the skill audit-code is missing: run cardinal sync`; palette only, not the help line (answers 1 a, 2 a, 3 a).
- `screen.rs` is over the file size limit; it is not touched here (`split_the_screen_module_of_the_ui` splits it).

## Deliverable 1
done: `a` (palette `a audit`) and `cardinal-ui --audit` open a zellij tab `audit` in the main worktree running
`claude /audit-code`, with the refusals of SUA-24.

| check | result |
|---|---|
| tests | `cargo test -p cardinal-ui` green (8 new in `tests/audit.rs`, 1 in `keys.rs`) |
| lint | `project-check` green |
| specs | SUA-23, SUA-24, SUA-25, SUA-H10 traced; SUA, SUP, STU back to `accepted` |
| docs | README, `docs/usage.md`, `docs/architecture.md` |
| code review | `low`: no findings |

red: SUA-H10, SUA-23, SUA-24, SUA-25: `usage: cardinal-ui [--print | … | --cut <milestone>]` (exit 2, flag unknown)
Code review: no findings
not verified: the key `a` in a real zellij session (the fake `zellij` of `tests/audit.rs` stands for it).
decisions needed: none
