---
priority: low
why: 'a comfort: the lock costs a hook call per tool call but harms no one working in a single session'
---
# Turn off the workspace lock

# Goal
The workspace lock runs on every tool call (`PreToolUse`), at the start and at the end of each session, to keep
one session per worktree. A user who never runs agents in parallel pays the calls for nothing. The outcome: a feature
`workspace-lock` made of its three hook entries.

# Findings
* The three hooks are `workspace-lock check`, `start` and `release` in `entries/.claude/settings.json`.
* `issue-workflow` claims each issue with `cardinal run issue-claim`, which also refuses uncommitted changes:
  whether the claim belongs to the feature is open.

# Approach
1. Declare the feature with the three entries.
2. Decide whether `issue-claim` stays (it orders `next-issue` across worktrees) or becomes a no-op when the lock is
   off, and amend `specs/issue-claim.md` if so.

# Specs
* `specs/features.md`, `specs/workspace-lock.md`, `specs/issue-claim.md` if the claim changes.

# Milestone
Optional features

# Resources
* `specs/features.md`, ADR 0025, `features.toml`, `crates/cardinal/src/features.rs`.
* `issues/done/let_a_project_turn_off_a_feature_of_the_harness.md` (the pilot `no-ai-attribution`).
* ADR 0022, ADR 0024, `docs/architecture.md` (Parallel agents).
