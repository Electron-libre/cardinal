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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
A diverged issue branch reaches the trunk in one step: a tool `cardinal run rebase-issue <stem>` rebases the branch
of a `merge pending` issue in its worktree, settles the conflicts that only add lines on both sides, refuses the rest
(pushed, dirty, held by another live session, other conflicts, aborted). The issue session runs it before printing the
merge; the key `f` runs it, then the project check, then the fast-forward. Measure (transcripts, 2026-11-23): 4 of 39
closes needed a rebase, 1 conflicted (`ROADMAP.md`, lines added to the same list on both sides).

## Plan
1. [x] Tool `rebase-issue`: the rebase of RBI in `crates/cardinal/src/tools/rebase_issue.rs`, registered in
   `cardinal run`; verified by `crates/cardinal/tests/rebase_issue.rs` on temp repositories; makes true RBI-1..RBI-5. (autonomous)
2. [x] issue-workflow "Merge back" (`files/.claude/skills/issue-workflow/details.md`, then `just sync`): with
   `"local"`, a diverged unpushed branch is rebased by `rebase-issue` with no stop, the project check and
   `cardinal check` run again; stop only on its refusal; verified by `crates/cardinal/tests/skills.rs` on the text. (autonomous)
3. [x] Split `specs/steering-ui-actions.md` (over `spec_max_bytes`): move the merge (SUA-2, SUA-7, SUA-22 and their
   scenarios) to `specs/steering-ui-merge.md` (SUM), the old ids `withdrawn: moved to SUM-n`, the tests retagged;
   no change of behaviour; verified by `cardinal run check-specs`. (autonomous)
4. [ ] Key `f` on a diverged branch (`crates/cardinal-ui/src/merge.rs`): `rebase-issue`, then `project-check` in the
   issue worktree, then the fast-forward; verified by the `--merge` tests of `crates/cardinal-ui/tests/`; makes true
   SUA-22 (SUM-n after 3), SUA-7 amended.

## Decisions
- Mode: autonomous
- Base commit: a4eec3a423edf8d9121a575fc95c610f35e47f15
- Split of `steering-ui-actions.md`: the merge moves to `specs/steering-ui-merge.md` (deliverable 3).
- Add-only conflicts are settled in every text file; the project check after the rebase catches the rest.
- The key `f` rebases a diverged branch with no second press.
- The UI runs `project-check` in the issue worktree after the rebase, before the fast-forward.

## Deliverable 1 summary
- done: `cardinal run rebase-issue <stem>` rebases the branch of a `merge pending` issue onto the trunk in its
  worktree, settles add-only conflicts, aborts the others, refuses dirty, locked and pushed; row in `docs/usage.md`.
- checks: tests pass (13 behaviour + 3 unit), clippy clean, `check-specs` passes (RBI accepted), docs updated.
- red: RBI-1, RBI-2, RBI-3, RBI-4, RBI-5: `cardinal: unknown tool `rebase-issue``
- Code review: no findings
- not verified: a rebase of a real issue branch of this repository (deliverable 2 runs it at the close).

## Deliverable 2 summary
- done: issue-workflow "Merge back" with `"local"` runs `rebase-issue` with no stop, then the project check and
  `cardinal check`; stops only on its refusal; the autonomous mode names that rebase as asked; `docs/workflow.md`.
- checks: tests pass, clippy clean, specs pass, docs updated, copies synced.
- red: none: `the_local_merge_back_rebases_a_diverged_branch_with_rebase_issue` failed on the missing
  `cardinal run rebase-issue <stem>` in "Merge back"
- Code review: no findings
- not verified: the session of a real close following the new text.

## Deliverable 3 summary
- done: the merge of the UI (SUA-2, SUA-7, SUA-22 and their scenarios) moved to `specs/steering-ui-merge.md` as
  SUM-1, SUM-2, SUM-3 (draft until SUM-3 is traced), the old ids withdrawn; every citation retagged; the docs link it.
- checks: tests pass, clippy clean, specs pass (`steering-ui-actions.md` back under `spec_max_bytes`), docs updated.
- red: none (no change of behaviour)
- Code review: no findings
- not verified: nothing.
