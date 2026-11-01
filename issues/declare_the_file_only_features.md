---
priority: low
why: 'a comfort: these files cost few tokens (skill descriptions) and no hook call'
---
# Declare the features made of managed files only

# Goal
Some managed files serve only part of the users: the skills `evaluate-harness` and `audit-instructions` (measuring
the harness), the rule `.claude/rules/reports-to-the-user.md` (a report style), and the GitHub CI
(`.github/workflows/cardinal.yml`, `.github/install_cardinal.sh`) for a project not on GitHub. The outcome: features
a project turns off, the fixed context shrinking by the descriptions of the skills it drops.

# Findings
* The mechanism copies or leaves out whole files (FEA-3): no code in `sync`.
* The retention warning (SYN-28) only matters to the measure of the harness: it belongs to the same feature.

# Approach
1. Declare `harness-measure` (the two skills), `report-style` (the rule) and `github-ci` (the workflow and its
   installer) in `features.toml`.
2. Gate the SYN-28 warning on `harness-measure` (an FEA-n, as FEA-10 does for the attribution warning).
3. Check that no other managed file cites a file of these features as required (`issue-workflow` names
   `skill:evaluate-harness` in `Expected-effect:` guidance): a citation of a skill turned off must stay harmless.

# Specs
* `specs/features.md` (the SYN-28 gate), `specs/sync.md` SYN-28 untouched.

# Milestone
Optional features

# Resources
* `specs/features.md`, ADR 0025, `features.toml`, `crates/cardinal/src/features.rs`.
* `issues/done/let_a_project_turn_off_a_feature_of_the_harness.md` (the pilot `no-ai-attribution`).
* `files/.claude/skills/evaluate-harness/`, `files/.claude/skills/audit-instructions/`, `files/.github/`.

# Hints
* The fixed context budget (SYN-38) shows the gain: compare `cardinal check` before and after on a consumer.
