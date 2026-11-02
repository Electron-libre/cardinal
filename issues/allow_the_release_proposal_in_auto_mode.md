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

# Tracking

Plan: validated

## Understanding
Manage the exact allow rule `Bash(just release)` (approach 1), so the proposal runs in auto mode; the forms with
arguments (changelog, tag) stay unallowed. No feature ties it: the `release` recipe of `files/cardinal.just` is shipped
to every project. SYN-29 already proves the mechanism; no spec amended.

## Plan
1. [x] Allow the release proposal: add `Bash(just release)` to `entries/.claude/settings.json`, `just sync`; verified by
   a test that the repository fragment allows the bare form and no form with arguments.

## Decisions
- Mode: autonomous
- Base commit: bcb15d094af4c7c11eeeb6dcab9001edf72e6eae

## Deliverable 1

done: `entries/.claude/settings.json` allows `Bash(just release)` (bare form only); `just sync` added it here.

| check | result |
|---|---|
| tests | `just check` green |
| lint | clean |
| specs | none amended (SYN-29 covers the mechanism) |
| docs | no document lists the managed allow rules |
| code review | low, see below |

red: none traced: `the_repository_settings_allow_the_release_proposal_only`: `left: [] right: ["Bash(just release)"]`

Code review: no findings

not verified: the auto mode classifier itself (only the allow rule is checked).
