---
priority: high
depends_on: [merge_a_closed_issue_from_the_steering_ui]
why: 'without it each merged issue leaves a session, a tab, a worktree and a branch to clean by hand'
---
# End the session of a merged issue

# Goal
Once its close is on the trunk, an issue leaves behind the `claude` session that worked it, the zellij tab the UI
opened for it, its worktree and its branch. The outcome: the steering UI ends that session, closes its tab and
removes the worktree with its branch (`cardinal run worktree close <stem>`), right after the merge or on a key, and
refuses while the session is still at work.

# Findings
* STU-12 names the tab after the stem; STU-10 shows `tab` when one of that name exists (`zellij action
  list-tabs --json`).
* The session of a worktree is the `session` and `pid` of `cardinal-session` in its git directory (WSL-1, STU-7);
  SWT-3 marks a session waiting for the user (`cardinal-waiting`).
* `cardinal run worktree close` refuses a worktree with uncommitted changes, an unmerged branch, or a run from inside
  it (PAR-6, PAR-7, PAR-11).
* Not measured yet: whether closing a zellij tab ends `claude` so that its `SessionEnd` hooks run (SWT-2 removes the
  waiting mark there).

# Approach
1. Measure how `claude` ends when its tab closes (`zellij action close-tab` on the focused tab only? a tab id from
   `list-tabs`? SIGHUP to the pid?) and whether `SessionEnd` runs; pick the way that does.
2. Refuse when the session runs and has no waiting mark (it is at work), or when the trunk does not hold the close.
3. End the session, close the tab, then `cardinal run worktree close <stem>` from the main worktree; report each
   step that fails and stop there.
4. Chain it after the merge of `merge_a_closed_issue_from_the_steering_ui`, and offer it alone for an issue merged
   by hand.

# Specs
* `specs/steering-ui.md`: amends STU-3; new requirements for the end of a session and its refusals.

# Milestone
Merge back from the steering UI

# Resources
* `specs/steering-ui.md` (STU-7, STU-10, STU-12, STU-13), `specs/session-wait.md`, `specs/workspace-lock.md`,
  `specs/parallel-issues.md` (PAR-6, PAR-7, PAR-11)
* `crates/cardinal-ui/src/start.rs`, `crates/cardinal/src/tools/worktree.rs`

# Hints
* A tab may hold other panes the user opened: close only what the UI can tie to the issue.
