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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
`rebase-issue` must read a lock whose `pid` another process took (that process started more than 2 s after the lock
was last written) as ended, like the end of SUE-4, so it no longer refuses a merge for a session that is gone.
Out of scope: the `workspace-lock` hook (a holder with no work in progress already frees it) and the UI message (Approach 2).

## Plan
1. [x] Pid-reuse rule in the lock check of `rebase-issue`: a `started_after(pid, lock)` beside `issue_claim::alive`
   (`ps -o etimes=` against the mtime of the lock), used by `refusal`; verified by a test of
   `crates/cardinal/tests/rebase_issue.rs` (lock mtime set ten minutes back, pid of a running process: it rebases);
   makes true RBI-4. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 6c58f8bf622af40f40b7f84df315bec6b4bc4000
- The rule is copied into the `cardinal` crate, not shared with `cardinal-ui` (which does not depend on it).
- Answers of the validation: 1 a (copy in the `cardinal` crate), 2 a (`workspace-lock` unchanged).

## Deliverable 1 summary
- done: `refusal` of `rebase-issue` counts the lock's session as ended when its `pid` started more than 2 s after the
  lock was last written (`issue_claim::started_after`); RBI-4 and `docs/usage.md` say so.
- red: RBI-4: `session gone works in <path>` (exit 1, expected 0) for a lock written ten minutes before its process started.
- Code review: no findings
- checks: `project-check` green, `deliverable-size` 76 lines, `file-size` none.
- not verified: a `ps` without `etimes` (BSD) keeps the old behaviour (the lock counts as live).
