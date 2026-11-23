---
priority: low
depends_on: [let_rebase_issue_pass_a_lock_whose_pid_another_process_took]
why: 'only a session that crashed without its SessionEnd hooks and whose pid a later process took, rare and settled by removing the lock by hand'
---
# Read a stale worktree lock alike in every reader

# Goal
The end of SUE-4 and `rebase-issue` (RBI-4) count the session of a worktree lock as ended when the process of its
`pid` started more than 2 seconds after the lock was last written (another process took the number). The other
readers of the lock (WSL-9) only ask whether that `pid` runs, so for the same stale lock the `f` merge refuses with
`session <id> works in the main worktree`, the UI shows the session live, and the `workspace-lock` hook refuses every
new session of a worktree with work in progress. The expected outcome: every reader reads a stale lock the same way.

# Findings
* `high` review of `let_rebase_issue_pass_a_lock_whose_pid_another_process_took` (2026-11-23), pre-existing:
  * `free_main_worktree` in `crates/cardinal-ui/src/merge.rs` filters `source::session(&main)` on `alive` only (SUM-2).
  * `resumable` in `crates/cardinal-ui/src/start.rs` (SUA-5) and the session state of `crates/cardinal-ui/src/panels.rs`
    (STU-7) read `alive` only.
  * `keeps` in `crates/cardinal/src/tools/workspace_lock.rs` frees the lock on `!alive(&holder.pid)` only (WSL-3).

# Approach
1. Read the session of a lock in one place per crate: `source::Session::alive` of `cardinal-ui` applying
   `started_after` (moved from `end.rs`), and `issue_claim::started_after` in `keeps`.
2. Amend STU-7 (the definition of an ended session) and WSL-3 so the specs say it once.

# Specs
* `specs/steering-ui.md`: STU-7; `specs/workspace-lock.md`: WSL-3.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/end.rs` (`started_after`, `running`), `crates/cardinal-ui/src/source.rs` (`session`)
* `crates/cardinal/src/tools/issue_claim.rs` (`started_after`), `crates/cardinal/src/tools/workspace_lock.rs` (`keeps`)

# Hints
* `started_after` reads `ps -o etimes=` against the wall clock: a forward clock step can make a live session look
  ended; reading `/proc/<pid>/stat` against the boot time avoids it on Linux.
