---
priority: medium
why: 'the order of the queue is changed from another terminal today; the UI is where the user reads it'
---
# Prioritize the issues from the UI

# Goal
The order of the queue comes from the `priority` and `depends_on` of the issue files (NXT-1), reviewed by
`skill:prioritize-issues`; the UI shows that order but cannot change it. The expected outcome is a command of the
UI that prioritizes the issues.

# Findings
* The UI already starts a skill session in a zellij tab: `v` starts `claude /release` in the main worktree
  (SUA-4, SUA-11, SUA-12).
* STU-3 lists what the UI may change: no issue file today.

# Approach
1. A key starts `claude /prioritize-issues` in a new zellij tab of the main worktree, as SUA-4 does for the release
   (refused outside zellij, or while a tab of that name is open).
2. Or edit in place: raise or lower the `priority` of the selected issue with two keys, writing its front matter
   and committing it (an amendment of STU-3, a write the UI does not do today).
3. Both: the skill for the full review, the keys for one issue.

# Specs
* `specs/steering-ui-actions.md`: a new requirement (the key, its refusals), SUA-15 (help line); `specs/steering-ui.md`:
  STU-3 (what the UI changes).

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/release.rs`, `crates/cardinal-ui/src/start.rs`, `.claude/skills/prioritize-issues/SKILL.md`,
  ADR 0021 (priority and dependencies in the front matter).
