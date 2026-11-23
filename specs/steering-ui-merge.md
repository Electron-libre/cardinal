---
id: SUM
status: accepted
adrs: [0022, 0030, 0031]
---

# Steering UI merge

## Intent

The key `f` and the flag `--merge` of `cardinal-ui` (STU) bring an issue closed on its branch into the trunk by a
fast-forward, rebasing first a branch another merged issue left behind (RBI), once the session that only waits for that merge
ended, each refused with its reason when the
project is not ready for it; the end of what the merge left is SUE.

## Requirements

### Behaviour

- **SUM-1** (deterministic) The key `f` on the issue selected in the queue, when a claim in the state `merge pending` takes it (CLM-7), must fast-forward the trunk (the branch of the main worktree, the first one `git worktree list --porcelain` lists) to the branch of that claim with `git merge --ff-only refs/heads/<branch>` run in the main worktree, then run `cardinal check` there, and say on the status line `merged <stem> (<branch>) into <trunk>`; on another issue, or when SUM-2 refuses, it merges nothing and says why on the status line.

### Contract

- **SUM-2** (deterministic) `cardinal-ui --merge <stem>` must do the merge of SUM-1 on the issue `<stem>` and exit 0, or print on stderr why it does not and exit 1, running no `git merge` when: `<stem>` is not open or no claim in the state `merge pending` takes it (`<stem>: not merge pending`); `<branch>` does not hold `issues/done/<stem>.md` (`<branch> does not hold the close of <stem>: commit it first`); the repository is bare (`no main worktree: the repository is bare`); the main worktree has no branch checked out (`main worktree not on a branch`); `git status --porcelain` lists anything there (`main worktree has uncommitted changes or untracked files`); its lock (STU-7) names a session whose process runs (`session <id> works in the main worktree`); the trunk is not an ancestor of `<branch>` and SUM-3 refuses or fails (its line); a git command it runs fails (its command, its exit code and the first line of its stderr). A `cardinal check` that fails after the merge keeps the merge, exits 1 and prints `merged <stem> (<branch>) into <trunk>; cardinal check: exit <n>: <first line of its stderr>`, the line that names the install it needs. The key `f` shows the same message on the status line.
- **SUM-3** (deterministic) When the trunk is not an ancestor of `<branch>` once every other check of SUM-2 passed, the merge must first run `cardinal run rebase-issue <stem>` (RBI-1) in the main worktree, then `cardinal run project-check` in the worktree that has `<branch>` checked out, then the checks of the main worktree of SUM-2 again, and fast-forward only once all pass, saying `rebased and merged <stem> (<branch>) into <trunk>`; a `rebase-issue` that exits non-zero merges nothing and its first stderr line is the message (a conflict leaves the rebase aborted, RBI-3); a `project-check` that exits non-zero keeps the rebase, merges nothing and says `rebased <branch> onto <trunk>; project-check: exit <n>: <first line of its stderr>`.
- **SUM-4** (deterministic) Before the `rebase-issue` of SUM-3, when the lock (STU-7) of the worktree that has `<branch>` checked out names a session whose process runs, that the wait mark of SWT-3 of that worktree names, and `git status --porcelain` lists nothing there, the merge ends that session with the steps of SUE-4 that come before its `worktree close` (its tab or its pane closed, else `SIGHUP`, then the wait of 10 seconds), then run `rebase-issue`; a session no wait mark names, or one in a worktree `git status --porcelain` lists anything in, is not ended, and the refusal of `rebase-issue` (RBI-4) is the message. A step of that end that fails merges nothing, its line after what the steps before it did; once the session ended, a refusal or a failure of what follows (SUM-2, SUM-3) starts with what the end did (`closed tab <stem>; ended session <id>; <line>`, or `ended session <id>; <line>`), so the user knows the session must be resumed to settle it. The end of SUE-1 chained after the merge then finds the tab closed and the session ended.

## Scenarios

- **SUM-H1** (happy; SUM-1, SUM-2) Given an issue `foo` closed on branch `foo` of a linked worktree, its claim
  `merge pending`, and a clean main worktree on `trunk` with no live session, when the user selects `foo` and presses
  `f`, then `trunk` points at the commit of `foo`, `cardinal check` ran in the main worktree, the status line reads
  `merged foo (foo) into trunk`, and the next read no longer lists `foo` (CLM-6).
- **SUM-H2** (happy; SUM-1, SUM-3) Given an issue `foo` closed on branch `foo` of a clean linked worktree no
  session runs in, its claim `merge pending`, and a commit on `trunk` that `foo` lacks, when the user selects `foo` and
  presses `f`, then `foo` is rebased onto `trunk`, the project check ran in its worktree, `trunk` points at the
  rebased `foo`, and the status line reads `rebased and merged foo (foo) into trunk`.
- **SUM-H3** (happy; SUM-1, SUM-3, SUM-4) Given the issue `foo` of SUM-H2 whose session waits for the user in
  the tab `foo` the UI opened, when the user presses `f` on it, then the tab `foo` is closed and the session ended
  before `foo` is rebased, the trunk points at the rebased `foo`, the worktree is removed, and the status line reads
  `rebased and merged foo (foo) into trunk; ended foo`.
- (SUM-2) Given a ready issue `bar` nothing takes, when `cardinal-ui --merge bar` runs, then it exits 1 with `bar:
  not merge pending` and the trunk is unchanged.
- (SUM-2) Given a tracked file modified, or an untracked file, in the main worktree, when `cardinal-ui --merge foo`
  runs, then it exits 1 saying the main worktree has uncommitted changes or untracked files, and runs no merge.
- (SUM-2) Given `close-issue` run on `foo` but its move not committed yet, when `cardinal-ui --merge foo` runs, then
  it exits 1 with `foo does not hold the close of foo: commit it first` and the trunk is unchanged.
- (SUM-2) Given a tag `foo` on an older commit than the branch `foo`, when it merges, then the trunk points at the
  branch.
- (SUM-2) Given the main worktree detached, when `cardinal-ui --merge foo` runs, then it exits 1 with `main worktree
  not on a branch`.
- (SUM-2) Given a lock of the main worktree naming a session whose process runs, when `cardinal-ui --merge foo`
  runs, then it exits 1 naming that session; given one whose process ended, it merges.
- (SUM-3) Given a line changed differently on `trunk` and on `foo`, when `cardinal-ui --merge foo` runs, then it
  exits 1 with the conflict line of RBI-3, `foo` and `trunk` are unchanged and no rebase is in progress.
- (SUM-4) Given the worktree of `foo` locked by a session whose process runs with no wait mark, when
  `cardinal-ui --merge foo` runs on a diverged `foo`, then it exits 1 with `session <id> works in <path>`, the session
  still runs and nothing is rewritten.
- (SUM-4) Given a waiting session in the worktree of `foo` and an untracked file there, when `cardinal-ui --merge
  foo` runs on a diverged `foo`, then the session still runs and it exits 1 with the line of RBI-4 naming that
  worktree.
- (SUM-4) Given a waiting session with no tab of the UI and a rebase that meets a conflict, when `cardinal-ui --merge
  foo` runs, then the session got `SIGHUP` and ended, and it exits 1 with `ended session <id>; ` and the conflict line.
- (SUM-4) Given a waiting session whose process ignores `SIGHUP`, when `cardinal-ui --merge foo` runs on a diverged
  `foo`, then it exits 1 with `session <id> still runs after 10 s` and runs no `rebase-issue`.
- (SUM-3) Given a project check that exits 1 after the rebase, when `cardinal-ui --merge foo` runs, then `foo` holds
  `trunk`, `trunk` is unchanged, and it exits 1 with `rebased foo onto trunk; project-check: exit 1:` and its line.
- (SUM-2) Given a `cardinal check` that exits 1 with `the installed cardinal cannot run the hook ...: install it
  with ...`, when `cardinal-ui --merge foo` runs, then the trunk holds `foo`, and it exits 1 printing `merged foo
  (foo) into trunk; cardinal check: exit 1:` and that line.
- (SUM-2) Given `cardinal-ui` run in a linked worktree, when it merges, then the trunk of the main worktree moves and
  the linked worktree is unchanged.

## Non-goals

- Settling in the UI a conflict `rebase-issue` refuses (RBI-3), rebasing in a worktree a running session holds (a waiting one is ended first, SUM-4), ending a session at work,
  merging anything but a fast-forward, or pushing the trunk: the conflict is the issue session's, the push the user's.
