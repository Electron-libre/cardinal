---
priority: medium
why: 'one deny rule closes what the autonomous mode leaves open: a release written unasked'
---
# Keep writing the release the user's

# Goal
The Rules of `issue-workflow` say a release is the user's to trigger, and the agent only proposes the version
(`cardinal run release` with no argument). Yet the managed allow rule `Bash(cardinal run *)` also lets
`cardinal run release <version>` (writes `CHANGELOG.md`, runs `release_version_command`) and
`cardinal run release <version> --tag` (creates the tag) run without a prompt. The outcome: the bare proposal stays
allowed, the forms that write ask the user.

# Findings
* 2026-11-02, the `high` review of `issues/done/allow_the_release_proposal_in_auto_mode.md`: `entries/.claude/settings.json`
  allows `Bash(cardinal run *)`, which covers every form of `cardinal run release`; nothing denies the writing forms.

# Approach
1. Manage the deny rule `Bash(cardinal run release *)` in `entries/.claude/settings.json` (the bare form, matched by
   the allow rule only, stays allowed); check that a deny with a wildcard wins over the broader allow, and that it
   does not block the user typing it.
2. Or have `review-gate check` refuse the writing forms of `cardinal run release` and `just release` when an agent
   runs them.

# Milestone
Longer autonomy

# Resources
* `entries/.claude/settings.json`, `features.toml` (a deny rule may belong to a feature), `specs/release.md`,
  `specs/sync.md` (SYN-29), `files/.claude/skills/issue-workflow/SKILL.md` (Rules).

# Hints
* A rule on `just release <version>` alone is not enough: a project may redefine the recipe (`set
  allow-duplicate-recipes`).
