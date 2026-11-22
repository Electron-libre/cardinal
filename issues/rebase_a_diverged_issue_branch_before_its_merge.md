---
priority: medium
why: 'every issue closed on its branch while another one reached the trunk needs it, by hand today; the milestone steers from the UI, which refuses the merge without offering the way out'
---
# Rebase a diverged issue branch before its merge

# Goal
With `merge_back = "local"`, an issue closed on its branch is merged by a fast-forward of the trunk (the key `f` of
`cardinal-ui`, SUA-2, or `git merge --ff-only` in the main worktree). Once another issue reached the trunk since the
branch started, the fast-forward is refused (`<branch> diverged from <trunk>: rebase it in its worktree`, SUA-7) and the
user must go back to the issue session, ask for the rebase, wait for the checks, then merge. The expected outcome: a
diverged branch reaches the trunk in one step of the user, the rebase done and checked for them, and a conflict the
tool cannot settle stops with what it names.

# Findings
* Session of `wrap_the_status_line_of_the_ui_on_a_narrow_terminal` (2026-11-23): after the close, `git merge-base
  --is-ancestor main HEAD` failed, two `docs` commits having filed issues on the trunk; the agent stopped to propose
  the rebase (issue-workflow `details.md`, "Merge back"), the user answered, then the rebase stopped on a conflict in
  `ROADMAP.md`: both sides had added lines to the issue lists of the milestones (the branch a follow-up issue, the
  trunk a new milestone). Resolved by hand by keeping both, then `just check` and `cardinal check` ran again.
* `specs/steering-ui-actions.md` lists "Rebasing a diverged branch" as a non-goal: "the rebase is the issue
  session's".
* The rebase of an unpushed local branch is a history rewrite the autonomous mode never does unasked
  (issue-workflow, "Autonomous mode").

# Approach
1. Measure first how often a close meets a diverged trunk (the `merge pending` claims of the past sessions, or the
   rebases of their transcripts), and which files conflict.
2. In issue-workflow, "Merge back" with `"local"`: rebase before the close is committed (or right after it) with no
   stop when the branch was never pushed and the rebase applies cleanly, the checks run again; stop only on a
   conflict.
3. A tool (`cardinal run rebase-issue`?) that rebases the branch of an issue onto the trunk in its worktree and
   settles the conflicts that are only lines added to the same list on both sides (the issues of a milestone in the
   roadmap), refusing any other conflict with the files it names.
4. The key `f` of `cardinal-ui`, on a diverged branch, offers the rebase in the worktree of the issue (clean, held by
   no running session) then the fast-forward and `cardinal check`, the conflict reported on the status line with the
   rebase aborted.

# Specs
* `specs/steering-ui-actions.md`: SUA-7 (the refusal of a diverged branch) and the non-goal of the rebase, if the
  key `f` rebases.
* A spec of the tool of approach 3, if it is chosen, written at `issue-workflow` step 1.

# Milestone
A steering UI to command

# Resources
* `.claude/skills/issue-workflow/details.md` ("Merge back"), `files/` for its source
* `crates/cardinal-ui/src/merge.rs` (`free_main_worktree`, the merge of SUA-2), `specs/steering-ui-actions.md`
  (SUA-2, SUA-7, SUA-11)
* `issues/done/merge_back_locally_when_the_project_asks.md`, `issues/done/merge_a_closed_issue_from_the_steering_ui.md`

# Hints
* Never rebase a pushed branch, and never in a worktree held by a running session (STU-7): the session there would
  find its history rewritten under it.
* Git runs with the worktree of the issue as its directory, never the main one from a linked worktree (SUA-7).
