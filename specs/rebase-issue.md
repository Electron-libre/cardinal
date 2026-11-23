---
id: RBI
status: accepted
adrs: [0022]
---

# Rebasing an issue branch

## Intent

`rebase-issue` brings an issue closed on its branch back onto the trunk when another issue reached the trunk first,
so the fast-forward of its merge can run: the issue session runs it before printing the merge (issue-workflow,
"Merge back") and the key `f` of `cardinal-ui` runs it on a diverged branch (SUM-3).

## Requirements

### Behaviour

- **RBI-1** (deterministic) `cardinal run rebase-issue <stem>`, on an issue whose claim is in the state `merge pending` (CLM-7), must rebase the branch of that claim onto the trunk (the branch of the main worktree, the first one `git worktree list --porcelain` lists) with git run in the worktree that has that branch checked out, never in another one, print `rebased <branch> onto <trunk>` and exit 0, or, when the trunk is already an ancestor of the branch, rewrite nothing, print `<branch> already holds <trunk>` and exit 0.
- **RBI-2** (deterministic) A file a replayed commit leaves in conflict, whose every conflict hunk adds lines on both sides at the same place of a common base that held none there (lines added to the same list on both sides), must be settled by keeping the lines of the trunk then those of the branch, the lines both sides added alike at its edges kept once, the rebase going on, and `settled <file>` printed for it.
- **RBI-3** (deterministic) Any other conflict (a hunk that changes or removes a line of the base, a file deleted, renamed or added on one side and changed on the other, a binary file) must abort the rebase with `git rebase --abort`, leaving the branch and its worktree as they were, print on stderr `<branch>: conflict in <file>, <file>: rebase aborted, settle it in <worktree path>` naming each file it could not settle, and exit 1.

### Contract

- **RBI-4** (deterministic) `rebase-issue` must print on stderr why it does not rebase and exit 1, running no `git rebase`, when: `<stem>` has no claim in the state `merge pending` (`<stem>: not merge pending`); the repository is bare (`no main worktree: the repository is bare`); the main worktree has no branch checked out (`main worktree not on a branch`); the branch is the trunk (`<branch> is the trunk`); no worktree has the branch checked out (`<branch> is checked out in no worktree`); `git status --porcelain` lists anything in that worktree (`worktree <path> has uncommitted changes or untracked files`); the lock of that worktree (WSL-9) names a session whose process runs, started no more than 2 seconds after the lock was last written (a later one is another process that took the number, as SUE-4 reads it), and that is not the calling one (`CLAUDE_CODE_SESSION_ID`) (`session <id> works in <path>`); a commit of the branch the trunk lacks is reachable from a remote-tracking ref (`<branch> was pushed: rebase it by hand`); a git command it runs fails (its command, its exit code and the first line of its stderr).
- **RBI-5** (deterministic) The commits it replays must run no git hook (`core.hooksPath` pointed at a path holding none, for the rebase and its continuations) and open no editor, so a commit gate (REV) never refuses a commit that already passed it.

## Scenarios

- **RBI-H1** (happy; RBI-1) Given an issue `foo` closed on branch `foo` of a clean linked worktree, its claim
  `merge pending`, and a commit on `trunk` touching another file, when `rebase-issue foo` runs, then `foo` holds
  `trunk` with its own commits on top, its worktree is clean, and it prints `rebased foo onto trunk`.
- **RBI-H2** (happy; RBI-1, RBI-2) Given `ROADMAP.md` whose milestone lists gained `issues/a.md` on `trunk` and
  `issues/b.md` on `foo` at the end of the same list, when `rebase-issue foo` runs, then the list holds `issues/a.md`
  then `issues/b.md`, it prints `settled ROADMAP.md` and `rebased foo onto trunk`, and exits 0.
- (RBI-1) Given `foo` already on top of `trunk`, when `rebase-issue foo` runs, then the commit of `foo` is unchanged
  and it prints `foo already holds trunk`.
- (RBI-3) Given a line of `README.md` changed differently on `trunk` and on `foo`, when `rebase-issue foo` runs, then
  it exits 1 naming `README.md`, no rebase is in progress in the worktree of `foo`, and `foo` points at its former commit.
- (RBI-3) Given a conflict settled in `ROADMAP.md` and another one in `README.md` that cannot be, when `rebase-issue
  foo` runs, then it exits 1 naming `README.md` only, and the rebase is aborted.
- (RBI-3) Given a file deleted on `trunk` and changed on `foo`, when `rebase-issue foo` runs, then it exits 1 naming it.
- (RBI-4) Given an untracked file in the worktree of `foo`, when `rebase-issue foo` runs, then it exits 1 naming that
  worktree and `foo` is unchanged.
- (RBI-4) Given the lock of the worktree of `foo` naming a session whose process runs, when `rebase-issue foo` runs
  from another session, then it exits 1 naming it; run from that session (its `CLAUDE_CODE_SESSION_ID`), or once its
  process ended, it rebases.
- (RBI-4) Given the lock of the worktree of `foo` written ten minutes before the process of its `pid` started, when
  `rebase-issue foo` runs from another session, then it rebases.
- (RBI-4) Given `refs/remotes/origin/foo` pointing at a commit of `foo` the trunk lacks, when `rebase-issue foo` runs,
  then it exits 1 with `foo was pushed: rebase it by hand`.
- (RBI-4) Given a ready issue `bar` nothing claims, when `rebase-issue bar` runs, then it exits 1 with `bar: not merge
  pending`.
- (RBI-5) Given a `pre-commit` hook that exits 1 and a conflict that RBI-2 settles, when `rebase-issue foo` runs, then
  it rebases.

## Non-goals

- Running the project check or `cardinal check` after the rebase: the caller does (issue-workflow, SUM-3).
- Settling a conflict that changes a line of the base, even when both sides look compatible, or ordering the lines
  it keeps otherwise than trunk then branch.
- Rebasing a branch with no `merge pending` claim (an issue still worked), or rebasing onto anything but the local
  trunk; pushing anything.
