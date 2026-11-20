---
priority: high
why: 'a session started on the trunk for a release needs the steps it follows, today only in the close of issue-workflow'
---
# Write the release steps in a skill

# Goal
The steps of a release (the proposed version, `release <version>`, the reread of the section, its `low` review, the
commit, `release <version> --tag`, the push left to the user) live in the close rules of `issue-workflow`, which only
an issue session reads. The outcome: a managed skill `release` holds them, `issue-workflow` points at it, and a
session started with `/release` on the trunk follows them.

# Findings
* `files/.claude/skills/issue-workflow/SKILL.md`, the Rules: "on that line, propose a release to the user ...; they
  push."
* The `ask` rules `Bash(cardinal run release *.*)` and `Bash(just release *.*)` make Claude Code prompt before a
  release whatever the mode.
* An issue session runs in a linked worktree and never enters the main one (ADR 0011): the release is prepared on
  the trunk, in the main worktree.

# Approach
1. Write `files/.claude/skills/release/SKILL.md`: check it runs on the trunk with a clean tree, then the steps above.
2. Replace the steps in `issue-workflow` by a pointer to the skill (on the trunk) or to the steering UI (on a
   branch).
3. Decide whether `features.toml` lists it (a project that never releases turns it off).

# Milestone
Releases from the steering UI

# Resources
* `files/.claude/skills/issue-workflow/SKILL.md`, `specs/release.md`, `entries/.claude/settings.json`, `features.toml`

# Hints
* Managed content: edit `files/`, then `just sync`.
