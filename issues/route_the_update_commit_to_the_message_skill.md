---
priority: low
depends_on: [find_why_commits_skip_the_commit_message_skill]
why: 'a few skips in consumer projects; the close line of the parent issue is measured first'
---
# Route the update commit to the message skill

# Goal
After `cardinal update`, the agent commits the new revision with a message written inline, without
`skill:commit-message`: nothing in the update output or the instructions points it there. The outcome: the update
names the next step, committing the change with `skill:commit-message`, and the harness update commits stop being
`skill-routing-commit-message` breaches.

# Findings
* Measured 2026-11-03, after BRE-16 left out the throwaway and checked commits: 9 harness update commits
  (`chore(cardinal): update Cardinal to …`, `chore(harness): update the shared harness to …`) in 5 sessions of one
  consumer project skipped the skill, the last on 2026-10-30; none ran the message checker.
* `cardinal update` reports the previous and new revisions and the files changed (SYN-16), no next step.

# Approach
1. Print, after the report of SYN-16, one line naming the next step: commit the update with
   `skill:commit-message` (not under `--dry-run`, nor when nothing changed).
2. Re-measure the harness update commits after a few updates.

# Specs
* `specs/sync.md`: SYN-16 amended, or a new requirement.

# Milestone
Lean agent loop

# Resources
* `specs/sync.md`, `crates/cardinal/src/main.rs`.
* `issues/find_why_commits_skip_the_commit_message_skill.md`.
