---
priority: low
depends_on: [claim_an_issue_from_the_session_that_works_it]
why: 'a comfort: a worktree already serves several issues through git worktree add; the recipes only lag behind'
---
# Reuse a worktree across issues

# Goal
The user's recipes `just start-issue` and `just finish-issue` tie a worktree to one issue: the first creates
`.claude/worktrees/<stem>` on a branch `<stem>`, the second removes it once that issue is merged. Since
`issues/done/claim_an_issue_from_the_session_that_works_it.md`, a session claims its issue by a ref and the worktree
lock keeps one session per worktree, so a worktree can serve several issues in turn, on a branch the user picks, and
keep its build. The recipes should open and remove a worktree with no issue attached. The worktree stays the user's:
the agent still never creates, enters or leaves one (ADR 0011); only these two recipes change.

# Findings
* `start-issue` refuses an issue not committed on the main branch, creates `.claude/worktrees/<stem>` on a branch
  `<stem>`, adds `/.claude/worktrees/` to `info/exclude` and copies `.claude/cardinal.local.toml` (PAR-1, PAR-2,
  PAR-13, PAR-17).
* `finish-issue <stem>` removes the worktree and deletes the branch only once merged, `--force` otherwise (PAR-6,
  PAR-7, PAR-10, PAR-11, PAR-15).
* A branch named after an issue still takes it in `next-issue` (PAR-3); a branch named otherwise takes
  nothing, the claim does (CLM-5).
* Each new worktree pays a cold build (239 MB of `target/`, 4 s on 24 cores on this repository; much more on a larger
  project).
* Decision Q2 of the claim issue left this work to a follow-up, to keep that issue under the size limit.

# Approach
1. `start-issue` becomes the opening of a worktree: any name, a branch the user names (default: the name) cut from
   the trunk, with the exclude line and the copy of `cardinal.local.toml` kept; no issue file required.
2. `finish-issue` becomes the removal of a worktree on demand, under the same safety (merged, clean, not from
   inside), with no issue stem.
3. Rename the recipes if the names mislead (`open-worktree`, `close-worktree`); a rename goes to
   `migrations/<n>.toml` (ADR 0017) and the docs that name them.
4. Keep PAR-3 for the branches made by the old `start-issue` until none is left, or withdraw it.

# Specs
* `specs/parallel-issues.md`: PAR-1, PAR-2, PAR-6, PAR-10, PAR-11, PAR-15 amended, maybe PAR-3.

# Milestone
Longer autonomy

# Resources
* `crates/cardinal/src/tools/issue_worktree.rs`, `files/cardinal.just` (`start-issue`, `finish-issue`).
* `docs/adr/0022-a-session-claims-its-issue-and-locks-its-workspace.md`, `docs/adr/0011-the-user-opens-the-issue-worktree.md`.
* `files/.claude/skills/issue-workflow/details.md` ("Worktree"), `docs/usage.md` (Parallel issues in worktrees),
  `docs/workflow.md` (Parallel issues).

# Hints
* The agent still never creates, enters or leaves a worktree (ADR 0011): these recipes stay the user's.
