---
priority: medium
depends_on: [rebase_a_diverged_issue_branch_before_its_merge]
why: 'the rebase of the key f is refused in its usual case, the issue session still waiting in its tab, which costs the user one more step'
---
# End the waiting session before rebasing its branch

# Goal
The key `f` of `cardinal-ui` rebases a diverged branch before its merge (SUM-3), but `rebase-issue` refuses a worktree
whose lock names a running session other than the caller (RBI-4: `session <id> works in <path>`). After the close,
the issue session waits in its tab and holds that lock until the merge ends it (SUE-1, SUE-3); so when another issue
reached the trunk in the meantime, `f` is refused and the user must quit that session by hand first. The expected
outcome: `f` ends the session that only waits for the merge, then rebases and merges in one step, and still refuses
a session that is working.

# Findings
* `high` review of `rebase_a_diverged_issue_branch_before_its_merge` (2026-11-23): the refusal of RBI-4 meets the
  normal flow, since the issue session stops on the printed merge and stays alive in its tab (SUE-H2).
* The hint of that issue forbids a rebase in a worktree held by a running session (STU-7): the session there would
  find its history rewritten under it. A session ended first rewrites nothing under anyone.

# Approach
1. Read how SUE tells a session that waits (the mark of the `session-wait` hooks) from one that works, and how it
   ends it and its tab.
2. In the merge of SUM-3, when `rebase-issue` would refuse the lock of the worktree of the branch and that session
   waits for the user, end it (as SUE does) before running `rebase-issue`; a working session keeps the refusal.
3. Or, smaller: make the message of `f` say to quit the session of the tab `<stem>` first.

# Specs
* `specs/steering-ui-merge.md`: SUM-3 (the order of the end and the rebase).
* `specs/steering-ui-end.md`: SUE-1, SUE-3, if the end runs before the merge.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/merge.rs` (`rebase`), `crates/cardinal-ui/src/end.rs`
* `specs/rebase-issue.md` (RBI-4), `specs/steering-ui-merge.md`, `specs/steering-ui-end.md`

# Hints
* Never end a session that is working, nor one whose worktree has uncommitted changes (SUE-3).
* A rebase that then fails on a conflict leaves the session ended: say so on the status line.
