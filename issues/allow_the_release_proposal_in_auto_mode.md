---
priority: medium
why: 'the release proposal at a milestone end is refused in auto mode, so the user gets no version number'
---
# Allow the release proposal in auto mode

# Goal
At the end of a milestone the Rules of `issue-workflow` have the agent propose a release with `just release`, which
with no argument only prints the version it would number. No allow rule covers it, and the auto mode classifier
refuses it (the name reads as a publication), so the agent reports no number. The outcome: the proposal runs without a
prompt or a refusal, and writing the changelog or the tag stays the user's.

# Findings
* 2026-11-02, a consumer project in auto mode: `just release` with no argument was refused; the agent could not give
  the proposed version (last tag `v0.4.0`).
* `entries/.claude/settings.json` allows `Bash(cardinal run *)` and nothing for `just release`; the recipe only runs
  `cardinal run release {{args}}` (`files/cardinal.just`).

# Approach
1. Manage the allow rule `Bash(just release)` in `entries/.claude/settings.json` (the exact form, no argument), so
   `sync` adds it to every project (SYN-29).
2. Or have the Rules of `issue-workflow` run `cardinal run release`, already allowed; it also allows the forms that
   write the changelog and tag, so 1 is closer to "theirs to trigger".

# Specs
* `specs/sync.md`: none amended if only the fragment changes; a test may prove the rule is managed.

# Milestone
Longer autonomy

# Resources
* `entries/.claude/settings.json`, `files/cardinal.just`, `files/.claude/skills/issue-workflow/SKILL.md` (Rules).
* `specs/release.md`.

# Hints
* Check whether `features.toml` should tie the rule to a feature (a project with no release flow).
