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
keep its build. The recipes should open and remove a worktree with no issue attached, at a plain git path rather
than nested under `.claude/worktrees/`. The worktree stays the user's:
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
* ADR 0008 nested the worktrees under `.claude/worktrees/` only to spare the approval prompt of `EnterWorktree`;
  since ADR 0011 the agent never enters a worktree, so that reason is gone (ADR 0011, Consequences). The nesting
  still costs the `info/exclude` line (PAR-13), and a session launched in a nested worktree may load the main
  worktree's `CLAUDE.md` as an ancestor.
* `git worktree add` has no default path: it takes the one it is given.
* Decision Q2 of the claim issue left this work to a follow-up, to keep that issue under the size limit.

# Approach
1. `start-issue` becomes the opening of a worktree: any name, at the path the user gives (default: next to the main
   worktree, `../<repo>-<name>`, the place of ADR 0003), a branch the user names (default: the name) cut from the
   trunk, with the copy of `cardinal.local.toml` kept; no issue file, no `info/exclude` line.
2. A new ADR supersedes ADR 0008: worktrees no longer nest under `.claude/worktrees/`; PAR-13 is withdrawn, PAR-14
   stays (the scanners still read only git's files).
3. `finish-issue` becomes the removal of a worktree on demand, under the same safety (merged, clean, not from
   inside), with no issue stem, and still removes the worktrees left under `.claude/worktrees/`.
4. Rename the recipes if the names mislead (`open-worktree`, `close-worktree`); a rename goes to
   `migrations/<n>.toml` (ADR 0017) and the docs that name them.
5. Keep PAR-3 for the branches made by the old `start-issue` until none is left, or withdraw it.

# Specs
* `specs/parallel-issues.md`: PAR-1, PAR-2, PAR-6, PAR-10, PAR-11, PAR-15 amended, PAR-13 withdrawn, maybe PAR-3.

# Milestone
Longer autonomy

# Resources
* `crates/cardinal/src/tools/issue_worktree.rs`, `files/cardinal.just` (`start-issue`, `finish-issue`).
* `docs/adr/0022-a-session-claims-its-issue-and-locks-its-workspace.md`, `docs/adr/0011-the-user-opens-the-issue-worktree.md`,
  `docs/adr/0008-issue-worktrees-nest-under-claude-worktrees.md`, `docs/adr/0003-one-worktree-per-issue.md`.
* `files/.claude/skills/issue-workflow/details.md` ("Worktree"), `docs/usage.md` (Parallel issues in worktrees),
  `docs/workflow.md` (Parallel issues).

# Hints
* The agent still never creates, enters or leaves a worktree (ADR 0011): these recipes stay the user's.
