---
id: SUE
status: draft
adrs: [0022, 0023, 0030, 0031]
---

# Steering UI end

## Intent

The keys and the flags of `cardinal-ui` (STU) that end what a merged issue left, its session, its zellij tab, its
worktree and its branch, on one worktree or on every merged one, each refused with its reason when the project is not ready for it.

## Requirements

### Behaviour

- **SUE-1** (deterministic) The key `e` on the worktree selected in the worktrees panel (`Tab` moves the selection between the queue and the worktrees, the arrow keys, `j` and `k` within the panel, kept on the same path across reads and visible as STU-14 keeps an issue), when it is a linked worktree whose branch `<stem>` has its close on the trunk, must end its session, close the zellij tab the UI opened for it, and remove the worktree with its branch through `cardinal run worktree close`, and say on the status line `ended <stem>`; the merge of SUM-1 chains the same end on its issue, on the linked worktree of the branch of its claim, once its `cardinal check` passed, its status line `merged <stem> (<branch>) into <trunk>; ended <stem>`; on another worktree, or when SUE-3 refuses, it ends nothing and says why on the status line.
- **SUE-2** (deterministic) The key `E`, whatever the selection, must run the end of SUE-1 on each linked worktree whose row reads `merged` (STU-23), in the order of the worktrees panel, going on after one it refuses or fails on, and say on the status line one part per such worktree, `ended <stem>` or `kept <stem>: <why>` (the line of SUE-3 or SUE-4), joined by ` | `, or `no merged worktree`; like `s`, `f`, `e` and `v`, it starts nothing while another task runs, and `q`, `Esc` or `Ctrl+C` pressed during it quit once it has ended (SUA-6).

### Contract

- **SUE-3** (deterministic) `cardinal-ui --end <stem>` must do the end of SUE-1 on the linked worktree whose branch is `<stem>` and exit 0, or print on stderr why it does not and exit 1, ending, closing and removing nothing when: no linked worktree has the branch `<stem>` checked out (`<stem>: no linked worktree on branch <stem>`); the main worktree has no branch checked out (`main worktree not on a branch`); the trunk does not hold `issues/done/<stem>.md` (`<trunk> does not hold the close of <stem>: merge it first`); the branch has commits the trunk lacks, made after its close was merged (`<branch> has commits <trunk> lacks: merge them first`); the worktree is locked by `git worktree lock`, its directory gone or not (``<path> is locked: run `git worktree unlock '<path>'` first``); `git status --porcelain` lists anything in that worktree (`<path> has uncommitted changes or untracked files`); the root `cardinal-ui` runs in is that worktree or inside it (`cardinal-ui runs in <path>: run it from another worktree`); another worktree lies inside it (`<path> holds the worktree <inner>: remove that one first`); its session (STU-7) runs and the mark of SWT-3 of the worktree does not name it (`session <id> is at work in <path>`). A linked worktree whose directory is gone (removed by hand, `prunable` for git) gets neither the check of its `git status --porcelain`, nor that of the worktrees inside it, nor that of the root, and runs nothing in that path; its session is read from the lock its git directory keeps (STU-7), refused while at work and ended as SUE-4 says. The key `e` shows the same message on the status line.
- **SUE-4** (deterministic) The end must run its steps in this order, each only once the one before succeeded: in a zellij session, when a tab named `<stem>` holds a terminal pane whose command is `claude /issue-workflow issues/<stem>.md` (SUA-10, read from `list-panes`), close that tab with `zellij action close-tab-by-id <tab id>` when that pane is its only terminal pane, else that pane alone with `zellij action close-pane --pane-id terminal_<pane id>` (a pane whose command `exited`, or a `list-panes` that fails, counts as no such pane); when no such pane was closed and the session runs, send `SIGHUP` to its `pid` (a `kill` that fails once that process ended is no failure); then wait at most 10 seconds for that process to end (`session <id> still runs after 10 s`); then run `cardinal run worktree close <path of the worktree>` in the main worktree. A session ended as STU-7 says, its `pid` taken by another process, gets no signal. A step that fails ends it there with its command, its exit code and the first line of its stderr (a `zellij` step with no answer: the line of SUZ-4; each addressing the session as SUZ-3 says), after what the steps before it did (`closed tab <stem>` or `closed pane terminal_<pane id>`, `ended session <id>`), so the user knows what is left.
- **SUE-5** (deterministic) `cardinal-ui --end-merged` must run the end of SUE-2 and print on stdout one line per linked worktree whose branch `<branch>` has `issues/done/<branch>.md` on the trunk, in the order `git worktree list` gives, `ended <branch>` or `kept <branch>: <why>`, or the single line `no merged worktree`, and exit 0 when it kept none, 1 when it kept one; it ends nothing, prints on stderr why and exits 1 when the repository is bare (`no main worktree: the repository is bare`), the main worktree has no branch checked out (`main worktree not on a branch`) or `git worktree list` fails (its command, its exit code and the first line of its stderr). A linked worktree that is detached, or whose branch has no close on the trunk, gets no line and is left as it is.

## Scenarios

- **SUE-H1** (happy; SUE-1, SUE-3) Given an issue `foo` merged into the trunk by hand, its worktree still
  open, when the user moves the selection to the worktrees panel, the row of `foo` reads `merged` (STU-23), and pressing `e`
  on it closes its tab and removes the worktree and the branch.
- **SUE-H2** (happy; SUE-1, SUE-4) Given the issue `foo` of SUM-H1 worked by a session waiting for the user
  in the tab `foo` the UI opened, when the user presses `f` on it (SUM-1), then the trunk holds `foo`, the tab `foo` is
  closed, the session ran its `SessionEnd` hooks and ended, the worktree and the branch `foo` are removed, and the
  status line reads `merged foo (foo) into trunk; ended foo`.
- **SUE-H3** (happy; SUE-2, SUE-5, SUE-3, SUE-4) Given the issues `foo` and `bar` merged into the trunk by hand,
  their worktrees still open with no live session, and a linked worktree on `baz` whose issue is still open, when the
  user presses `E`, then the worktrees and the branches `foo` and `bar` are removed, the status line reads
  `ended foo | ended bar`, and the worktree of `baz` is unchanged.
- (SUE-3, SUE-5) Given the issue `foo` merged into the trunk by hand and the directory of its worktree removed by hand,
  when `cardinal-ui --end-merged` runs, then it runs `cardinal run worktree close <path>` in the main worktree,
  prints `ended foo` and exits 0.
- (SUE-3) Given the directory of the worktree of `foo` removed by hand while its session runs at work, when
  `cardinal-ui --end foo` runs, then it exits 1 with `session s1 is at work in <path>` and closes nothing.
- (SUE-3) Given the session of `foo` running with no wait mark, when `cardinal-ui --end foo` runs, then it exits 1
  with `session <id> is at work in <path>`, and the tab, the session and the worktree are kept.
- (SUE-3) Given a trunk that does not hold `issues/done/foo.md`, when `cardinal-ui --end foo` runs, then it exits 1
  with `trunk does not hold the close of foo: merge it first` and runs no `zellij` nor `worktree close`.
- (SUE-3) Given the session of `foo` waiting after a commit on the branch `foo` made once its close was merged,
  when `cardinal-ui --end foo` or `--end-merged` runs, then it keeps `foo` with `foo has commits trunk lacks: merge
  them first`, and the tab and the session are kept.
- (SUE-3) Given an untracked file in the worktree of `foo`, when `cardinal-ui --end foo` runs, then it exits 1 and
  the session still runs.
- (SUE-3) Given `cardinal-ui` run in the worktree of `foo`, when it ends `foo`, then it exits 1 naming that
  worktree and ends nothing.
- (SUE-3) Given no linked worktree on the branch `foo` (the main worktree is on it, or none is), when
  `cardinal-ui --end foo` runs, then it exits 1 with `foo: no linked worktree on branch foo`.
- (SUE-3) Given a linked worktree whose branch has no close on the trunk, when the user presses `e` on it, then it
  ends nothing and says why on the status line.
- (SUE-3) Given the worktree of `foo` locked by `git worktree lock` (its directory gone or not), or holding another
  in an ignored directory, its session waiting in its tab, when `cardinal-ui --end foo` runs, then it exits 1 naming
  `git worktree unlock` or the inner worktree, and the tab, the session and the worktree are kept.
- (SUE-4) Given the tab `foo` holding the session's pane and a shell the user opened, when the end runs, then only
  the session's pane is closed and the tab stays.
- (SUE-4) Given a waiting session started by hand, with no tab of the UI, when the end runs, then its process gets
  `SIGHUP`, and the worktree is removed once it ended.
- (SUE-4) Given a session whose process ignores `SIGHUP`, when the end runs, then it exits 1 with `session <id> still
  runs after 10 s` and the worktree is kept.
- (SUE-4) Given a session that already ended and no tab, when the end runs, then it only runs `worktree close`.
- (SUE-4) Given a `list-panes` that fails, or a pane of the session whose command exited, when the end runs, then it
  closes no tab, sends `SIGHUP` to the session and removes the worktree.
- (SUE-4) Given a lock written ten minutes before the process of its `pid` started, when the end runs, then that
  process gets no signal and the worktree is removed.
- (SUE-1) Given the claim of `foo` on the branch `feature-foo`, when `f` merges it, then the end runs on the worktree
  of `feature-foo`.
- (SUE-4) Given a `worktree close` that exits 1 with `the branch foo is not merged into trunk ...`, when the end
  runs, then it exits 1 with `closed tab foo; ended session <id>; worktree close: exit 1:` and that line.
- (SUE-1) Given a merge whose `cardinal check` fails, when the user pressed `f`, then no end is chained and the
  session still runs.
- (SUE-5) Given `foo` merged with an untracked file in its worktree and `bar` merged and clean, when
  `cardinal-ui --end-merged` runs, then it prints `kept foo: <path> has uncommitted changes or untracked files` and
  `ended bar`, removes the worktree of `bar` only, and exits 1.
- (SUE-5) Given `cardinal-ui --end-merged` run in the merged worktree of `foo`, when it runs, then it keeps `foo`
  with `cardinal-ui runs in <path>: run it from another worktree` and ends the other merged worktrees.
- (SUE-5) Given no linked worktree whose branch has its close on the trunk, when `cardinal-ui --end-merged` runs,
  then it prints `no merged worktree`, runs no `worktree close` and exits 0.
- (SUE-5) Given the main worktree detached, when `cardinal-ui --end-merged` runs, then it exits 1 with `main
  worktree not on a branch` and ends nothing.
- (SUE-5) Given a worktree that served the issue `foo`, merged, and now serves the open issue `bar` on the branch
  `bar` (ADR 0023), when `cardinal-ui --end-merged` runs, then it leaves that worktree as it is.
- (SUE-2) Given a merge running, when the user presses `E`, then it starts nothing.

## Non-goals

- Ending a merged worktree on the read that first sees it merged, without a key: the user ends them with `E`.
- Ending a session at work, removing a branch `worktree close` refuses (`--force`), or closing a tab or a pane the
  UI cannot tie to the issue by its name and its command: the user does it by hand.
