---
priority: low
depends_on: [end_the_waiting_session_before_rebasing_its_branch]
why: 'only a session that crashed without its SessionEnd hooks and whose pid a later process took blocks the merge, rare and settled by removing the lock by hand'
---
# Let rebase-issue pass a lock whose pid another process took

# Goal
The end of SUE-4 counts a session as ended when the process of its `pid` started more than 2 seconds after the lock
was last written (another process took the number). `rebase-issue` (RBI-4) only asks whether that `pid` runs, so for
the same stale lock the UI ends nothing (SUM-4) and `rebase-issue` refuses with `session <id> works in <path>`, for a
session that is gone; the merge of `f` stays blocked until the user removes the lock by hand. The expected outcome:
both read a stale lock the same way.

# Findings
* `high` review of `end_the_waiting_session_before_rebasing_its_branch` (2026-11-23), pre-existing: `running` in
  `crates/cardinal-ui/src/end.rs` applies the `started_after` rule; `refusal` in
  `crates/cardinal/src/tools/rebase_issue.rs` calls `issue_claim::alive` only.

# Approach
1. Apply the pid-reuse rule of SUE-4 to the lock check of RBI-4 (the start time of the process against the mtime of
   the lock), shared with the other readers of the lock (`issue_claim::alive`, WSL-9) if they need it too.
2. Or, smaller: make the UI say that the lock is stale and name the file to remove.

# Specs
* `specs/rebase-issue.md`: RBI-4.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal/src/tools/rebase_issue.rs` (`refusal`), `crates/cardinal/src/tools/issue_claim.rs` (`alive`)
* `crates/cardinal-ui/src/end.rs` (`started_after`, `running`), `specs/steering-ui-end.md` (SUE-4)
