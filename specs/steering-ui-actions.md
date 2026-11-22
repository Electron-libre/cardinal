---
id: SUA
status: accepted
adrs: [0022, 0023, 0030, 0031]
---

# Steering UI actions

## Intent

The keys and the flags of `cardinal-ui` (STU) that change the project: start a session on a ready issue, or on the
release of the milestones awaiting it, in a zellij tab of its own, merge into the trunk an issue closed on its branch,
and end what a merged issue left, each refused with its reason when the project is not ready for it.

## Requirements

### Behaviour

- **SUA-1** (deterministic) In a zellij session, the key `s` on the issue selected in the queue (moved with the arrow keys, `j` and `k`), when its state is `ready` and nothing takes it, must open a worktree for it with `cardinal run worktree open <stem>` and start in it, in a new zellij tab named after its stem, an interactive `claude` whose first message hands `issues/<stem>.md` to `issue-workflow`; when its state is `ready` and it is taken only by a claim in the state `working` whose session ended (CLM-5), it resumes it: open no worktree and start that tab in the worktree of the claim, so the new session takes the claim back (CLM-2); on another issue it starts nothing and says why on the status line.
- **SUA-2** (deterministic) The key `f` on the issue selected in the queue, when a claim in the state `merge pending` takes it (CLM-7), must fast-forward the trunk (the branch of the main worktree, the first one `git worktree list --porcelain` lists) to the branch of that claim with `git merge --ff-only refs/heads/<branch>` run in the main worktree, then run `cardinal check` there, and say on the status line `merged <stem> (<branch>) into <trunk>`; on another issue, or when SUA-7 refuses, it merges nothing and says why on the status line.
- **SUA-3** (deterministic) The key `e` on the worktree selected in the worktrees panel (`Tab` moves the selection between the queue and the worktrees, the arrow keys, `j` and `k` within the panel, kept on the same path across reads and visible as STU-14 keeps an issue), when it is a linked worktree whose branch `<stem>` has its close on the trunk, must end its session, close the zellij tab the UI opened for it, and remove the worktree with its branch through `cardinal run worktree close`, and say on the status line `ended <stem>`; the merge of SUA-2 chains the same end on its issue, on the linked worktree of the branch of its claim, once its `cardinal check` passed, its status line `merged <stem> (<branch>) into <trunk>; ended <stem>`; on another worktree, or when SUA-8 refuses, it ends nothing and says why on the status line.
- **SUA-13** (deterministic) The key `E`, whatever the selection, must run the end of SUA-3 on each linked worktree whose row reads `merged` (STU-23), in the order of the worktrees panel, going on after one it refuses or fails on, and say on the status line one part per such worktree, `ended <stem>` or `kept <stem>: <why>` (the line of SUA-8 or SUA-9), joined by ` | `, or `no merged worktree`; like `s`, `f`, `e` and `v`, it starts nothing while another task runs, and `q`, `Esc` or `Ctrl+C` pressed during it quit once it has ended (SUA-6).
- **SUA-4** (deterministic) In a zellij session, the key `v`, when a milestone awaits its release (STU-1) or a release prepared is not tagged yet (an interrupted release that `skill:release` resumes), must start in the main worktree, in a new zellij tab named `release`, an interactive `claude` whose first message is `/release`, so the version, its reread, its review and its tag stay with `skill:release`; when SUA-11 refuses, it starts nothing and says why on the status line.

### Contract

- **SUA-5** (deterministic) `cardinal-ui --start <stem>` must start the session of SUA-1 on the issue `<stem>` and exit 0, or print on stderr why it does not and exit 1: outside a zellij session (no `ZELLIJ` variable: `start needs zellij: run cardinal-ui in a zellij session`), an issue that is not open, not `ready`, or taken other than by a claim whose session ended, a claim whose session ended in a worktree that `git worktree list` no longer lists (`<stem>: worktree <path> gone: cardinal run issue-claim --release <stem>`) or whose lock (STU-7) names a session that runs (`<stem>: session <id> works in <path>`), or a step that fails (its command, its exit code and the first line of its stderr), running no step after a failed one; the key `s` shows the same message on the status line.
- **SUA-6** (deterministic) `q`, `Esc` or `Ctrl+C` pressed during a start (of an issue or of the release), a merge or an end must quit once it has ended, so a start never stops between the worktree and the tab, a merge before its `cardinal check` nor an end between the session and the worktree, whose result is then printed on stderr once the terminal is given back; `s`, `f`, `e` and `v` start nothing while one runs.
- **SUA-7** (deterministic) `cardinal-ui --merge <stem>` must do the merge of SUA-2 on the issue `<stem>` and exit 0, or print on stderr why it does not and exit 1, running no `git merge` when: `<stem>` is not open or no claim in the state `merge pending` takes it (`<stem>: not merge pending`); `<branch>` does not hold `issues/done/<stem>.md` (`<branch> does not hold the close of <stem>: commit it first`); the repository is bare (`no main worktree: the repository is bare`); the main worktree has no branch checked out (`main worktree not on a branch`); `git status --porcelain` lists anything there (`main worktree has uncommitted changes or untracked files`); its lock (STU-7) names a session whose process runs (`session <id> works in the main worktree`); the trunk is not an ancestor of `<branch>` (`<branch> diverged from <trunk>: rebase it in its worktree`); a git command it runs fails (its command, its exit code and the first line of its stderr). A `cardinal check` that fails after the merge keeps the merge, exits 1 and prints `merged <stem> (<branch>) into <trunk>; cardinal check: exit <n>: <first line of its stderr>`, the line that names the install it needs. The key `f` shows the same message on the status line.
- **SUA-8** (deterministic) `cardinal-ui --end <stem>` must do the end of SUA-3 on the linked worktree whose branch is `<stem>` and exit 0, or print on stderr why it does not and exit 1, ending, closing and removing nothing when: no linked worktree has the branch `<stem>` checked out (`<stem>: no linked worktree on branch <stem>`); the main worktree has no branch checked out (`main worktree not on a branch`); the trunk does not hold `issues/done/<stem>.md` (`<trunk> does not hold the close of <stem>: merge it first`); the branch has commits the trunk lacks, made after its close was merged (`<branch> has commits <trunk> lacks: merge them first`); the worktree is locked by `git worktree lock`, its directory gone or not (``<path> is locked: run `git worktree unlock '<path>'` first``); `git status --porcelain` lists anything in that worktree (`<path> has uncommitted changes or untracked files`); another worktree lies inside it (`<path> holds the worktree <inner>: remove that one first`); the root `cardinal-ui` runs in is that worktree or inside it (`cardinal-ui runs in <path>: run it from another worktree`); its session (STU-7) runs and the mark of SWT-3 of the worktree does not name it (`session <id> is at work in <path>`). A linked worktree whose directory is gone (removed by hand, `prunable` for git) gets neither the check of its `git status --porcelain` nor that of the root, and runs nothing in that path; its session is read from the lock its git directory keeps (STU-7), refused while at work and ended as SUA-9 says. The key `e` shows the same message on the status line.
- **SUA-9** (deterministic) The end must run its steps in this order, each only once the one before succeeded: in a zellij session, when a tab named `<stem>` holds a terminal pane whose command is `claude /issue-workflow issues/<stem>.md` (SUA-10, read from `list-panes`), close that tab with `zellij action close-tab-by-id <tab id>` when that pane is its only terminal pane, else that pane alone with `zellij action close-pane --pane-id terminal_<pane id>` (a pane whose command `exited`, or a `list-panes` that fails, counts as no such pane); when no such pane was closed and the session runs, send `SIGHUP` to its `pid` (a `kill` that fails once that process ended is no failure); then wait at most 10 seconds for that process to end (`session <id> still runs after 10 s`); then run `cardinal run worktree close <path of the worktree>` in the main worktree. A `pid` whose process started more than 2 seconds after the lock was last written is another process that took the number: the session counts as ended, its process gets no signal. A step that fails ends it there with its command, its exit code and the first line of its stderr, after what the steps before it did (`closed tab <stem>` or `closed pane terminal_<pane id>`, `ended session <id>`), so the user knows what is left.
- **SUA-10** (deterministic) The `zellij` it runs must be the program named by `CARDINAL_UI_ZELLIJ`, else `zellij` found on the `PATH`, the tab opened with `zellij action new-tab --cwd <worktree path> --name <stem> -- claude "/issue-workflow issues/<stem>.md"`; outside a zellij session, or when `list-tabs` fails, the queue shows no `tab` and no error.
- **SUA-11** (deterministic) `cardinal-ui --release` must start the session of SUA-4 and exit 0, or print on stderr why it does not and exit 1, opening no tab when: outside a zellij session (`release needs zellij: run cardinal-ui in a zellij session`); the repository is bare (`no main worktree: the repository is bare`); the main worktree has no branch checked out (`main worktree not on a branch`); `git status --porcelain` lists anything there (`main worktree has uncommitted changes or untracked files`); its lock (STU-7) names a session whose process runs (`session <id> works in the main worktree`); a tab named `release` holds a terminal pane whose command is `claude /release` and has not `exited` (read from `list-panes`, one that fails counting as no such pane) (`a release tab is already open`); `cardinal run release --json`, run in the main worktree, lists no milestone and is not `prepared` (REL-15) (`no milestone awaits its release`), or cannot be read (the line of STU-6); the main worktree has no `.claude/skills/release/SKILL.md`, the project having turned the skill off (`the skill release is turned off: run the steps cardinal run release prints`), checked once `release --json` found a release to start; a step that fails (its command, its exit code and the first line of its stderr). The key `v` shows the same message on the status line.
- **SUA-12** (deterministic) The tab of SUA-4 must be opened with `zellij action new-tab --cwd <main worktree path> --name release -- claude /release`, the `zellij` of SUA-10, run once every check of SUA-11 passed.
- **SUA-14** (deterministic) `cardinal-ui --end-merged` must run the end of SUA-13 and print on stdout one line per linked worktree whose branch `<branch>` has `issues/done/<branch>.md` on the trunk, in the order `git worktree list` gives, `ended <branch>` or `kept <branch>: <why>`, or the single line `no merged worktree`, and exit 0 when it kept none, 1 when it kept one; it ends nothing, prints on stderr why and exits 1 when the repository is bare (`no main worktree: the repository is bare`), the main worktree has no branch checked out (`main worktree not on a branch`) or `git worktree list` fails (its command, its exit code and the first line of its stderr). A linked worktree that is detached, or whose branch has no close on the trunk, gets no line and is left as it is.

## Scenarios

- **SUA-H1** (happy; SUA-1, SUA-5, SUA-10) Given the UI in a zellij session and a ready issue `foo` nothing
  takes, when the user selects it and presses `s`, then the worktree `<main worktree>-foo` is opened and a tab `foo`
  runs `claude` on `issues/foo.md` there; the next read shows `foo` taken with `tab`.
- **SUA-H2** (happy; SUA-2, SUA-7) Given an issue `foo` closed on branch `foo` of a linked worktree, its claim
  `merge pending`, and a clean main worktree on `trunk` with no live session, when the user selects `foo` and presses
  `f`, then `trunk` points at the commit of `foo`, `cardinal check` ran in the main worktree, the status line reads
  `merged foo (foo) into trunk`, and the next read no longer lists `foo` (CLM-6).
- **SUA-H3** (happy; SUA-2, SUA-3, SUA-9) Given the issue `foo` of SUA-H2 worked by a session waiting for the user
  in the tab `foo` the UI opened, when the user presses `f` on it, then the trunk holds `foo`, the tab `foo` is
  closed, the session ran its `SessionEnd` hooks and ended, the worktree and the branch `foo` are removed, and the
  status line reads `merged foo (foo) into trunk; ended foo`.
- **SUA-H4** (happy; SUA-3, SUA-8) Given an issue `foo` merged into the trunk by hand, its worktree still
  open, when the user moves the selection to the worktrees panel, the row of `foo` reads `merged` (STU-23), and pressing `e`
  on it closes its tab and removes the worktree and the branch.
- **SUA-H5** (happy; SUA-4, SUA-11, SUA-12) Given the UI in a zellij session in a linked worktree, a milestone that
  reads `awaits release v0.3.0`, and a clean main worktree on `trunk` with no live session, when the user presses
  `v`, then a tab `release` runs `claude /release` in the main worktree, and the linked worktree is unchanged.
- **SUA-H6** (happy; SUA-13, SUA-14, SUA-8, SUA-9) Given the issues `foo` and `bar` merged into the trunk by hand,
  their worktrees still open with no live session, and a linked worktree on `baz` whose issue is still open, when the
  user presses `E`, then the worktrees and the branches `foo` and `bar` are removed, the status line reads
  `ended foo | ended bar`, and the worktree of `baz` is unchanged.
- (SUA-8, SUA-14) Given the issue `foo` merged into the trunk by hand and the directory of its worktree removed by hand,
  when `cardinal-ui --end-merged` runs, then it runs `cardinal run worktree close <path>` in the main worktree,
  prints `ended foo` and exits 0.
- (SUA-8) Given the directory of the worktree of `foo` removed by hand while its session runs at work, when
  `cardinal-ui --end foo` runs, then it exits 1 with `session s1 is at work in <path>` and closes nothing.
- (SUA-5) Given no `ZELLIJ` variable, when `cardinal-ui --start foo` runs, then it prints `start needs zellij` on
  stderr, exits 1 and runs neither `cardinal run worktree open` nor `zellij`.
- (SUA-5) Given an issue `foo` claimed by a live session, when `cardinal-ui --start foo` runs, then it exits 1 saying
  `foo` is taken, and opens nothing.
- (SUA-1) Given an issue `foo` claimed in the worktree `w` by a session whose process ended, when `cardinal-ui --start foo` runs in a zellij session, then it runs no `worktree open`, opens the tab `foo` in `w` with `claude "/issue-workflow issues/foo.md"`, prints `resumed foo in w` and exits 0.
- (SUA-5) Given an issue `foo` claimed by a session whose process ended in a worktree `w` that was removed, when `cardinal-ui --start foo` runs, then it exits 1 with `foo: worktree w gone: cardinal run issue-claim --release foo` and opens nothing.
- (SUA-5) Given an issue `foo` claimed by a session whose process ended in a directory `w` that is no worktree of the repository, when `cardinal-ui --start foo` runs, then it exits 1 with `foo: worktree w gone`.
- (SUA-5) Given an issue `foo` claimed by a session whose process ended in the worktree `w`, whose lock names the session `s` that runs, when `cardinal-ui --start foo` runs, then it exits 1 with `foo: session s works in w` and opens no tab.
- (SUA-5) Given an issue `foo` in the state `merge pending`, when `cardinal-ui --start foo` runs, then it exits 1 saying `foo` is taken.
- (SUA-5) Given a `worktree open` that exits 1 with `branch foo is already checked out`, when `cardinal-ui --start
  foo` runs, then it exits 1 with that line and runs no `zellij`.
- (SUA-5) Given a `zellij` that fails, when `cardinal-ui --start foo` runs, then it exits 1 naming `zellij` and the
  worktree stays open.
- (SUA-10) Given no zellij session, when `cardinal-ui --print` runs, then no issue shows `tab` and it exits 0.
- (SUA-7) Given a ready issue `bar` nothing takes, when `cardinal-ui --merge bar` runs, then it exits 1 with `bar:
  not merge pending` and the trunk is unchanged.
- (SUA-7) Given a tracked file modified, or an untracked file, in the main worktree, when `cardinal-ui --merge foo`
  runs, then it exits 1 saying the main worktree has uncommitted changes or untracked files, and runs no merge.
- (SUA-7) Given `close-issue` run on `foo` but its move not committed yet, when `cardinal-ui --merge foo` runs, then
  it exits 1 with `foo does not hold the close of foo: commit it first` and the trunk is unchanged.
- (SUA-7) Given a tag `foo` on an older commit than the branch `foo`, when it merges, then the trunk points at the
  branch.
- (SUA-7) Given the main worktree detached, when `cardinal-ui --merge foo` runs, then it exits 1 with `main worktree
  not on a branch`.
- (SUA-7) Given a lock of the main worktree naming a session whose process runs, when `cardinal-ui --merge foo`
  runs, then it exits 1 naming that session; given one whose process ended, it merges.
- (SUA-7) Given a commit on `trunk` that `foo` lacks, when `cardinal-ui --merge foo` runs, then it exits 1 with `foo
  diverged from trunk: rebase it in its worktree` and the trunk is unchanged.
- (SUA-7) Given a `cardinal check` that exits 1 with `the installed cardinal cannot run the hook ...: install it
  with ...`, when `cardinal-ui --merge foo` runs, then the trunk holds `foo`, and it exits 1 printing `merged foo
  (foo) into trunk; cardinal check: exit 1:` and that line.
- (SUA-7) Given `cardinal-ui` run in a linked worktree, when it merges, then the trunk of the main worktree moves and
  the linked worktree is unchanged.
- (SUA-6) Given a merge running, when the user presses `q`, then the screen quits once `cardinal check` has ended.
- (SUA-8) Given the session of `foo` running with no wait mark, when `cardinal-ui --end foo` runs, then it exits 1
  with `session <id> is at work in <path>`, and the tab, the session and the worktree are kept.
- (SUA-8) Given a trunk that does not hold `issues/done/foo.md`, when `cardinal-ui --end foo` runs, then it exits 1
  with `trunk does not hold the close of foo: merge it first` and runs no `zellij` nor `worktree close`.
- (SUA-8) Given the session of `foo` waiting after a commit on the branch `foo` made once its close was merged,
  when `cardinal-ui --end foo` or `--end-merged` runs, then it keeps `foo` with `foo has commits trunk lacks: merge
  them first`, and the tab and the session are kept.
- (SUA-8) Given an untracked file in the worktree of `foo`, when `cardinal-ui --end foo` runs, then it exits 1 and
  the session still runs.
- (SUA-8) Given `cardinal-ui` run in the worktree of `foo`, when it ends `foo`, then it exits 1 naming that
  worktree and ends nothing.
- (SUA-8) Given no linked worktree on the branch `foo` (the main worktree is on it, or none is), when
  `cardinal-ui --end foo` runs, then it exits 1 with `foo: no linked worktree on branch foo`.
- (SUA-8) Given a linked worktree whose branch has no close on the trunk, when the user presses `e` on it, then it
  ends nothing and says why on the status line.
- (SUA-8) Given the worktree of `foo` locked by `git worktree lock` (its directory gone or not), or holding another
  in an ignored directory, its session waiting in its tab, when `cardinal-ui --end foo` runs, then it exits 1 naming
  `git worktree unlock` or the inner worktree, and the tab, the session and the worktree are kept.
- (SUA-9) Given the tab `foo` holding the session's pane and a shell the user opened, when the end runs, then only
  the session's pane is closed and the tab stays.
- (SUA-9) Given a waiting session started by hand, with no tab of the UI, when the end runs, then its process gets
  `SIGHUP`, and the worktree is removed once it ended.
- (SUA-9) Given a session whose process ignores `SIGHUP`, when the end runs, then it exits 1 with `session <id> still
  runs after 10 s` and the worktree is kept.
- (SUA-9) Given a session that already ended and no tab, when the end runs, then it only runs `worktree close`.
- (SUA-9) Given a `list-panes` that fails, or a pane of the session whose command exited, when the end runs, then it
  closes no tab, sends `SIGHUP` to the session and removes the worktree.
- (SUA-9) Given a lock written ten minutes before the process of its `pid` started, when the end runs, then that
  process gets no signal and the worktree is removed.
- (SUA-3) Given the claim of `foo` on the branch `feature-foo`, when `f` merges it, then the end runs on the worktree
  of `feature-foo`.
- (SUA-9) Given a `worktree close` that exits 1 with `the branch foo is not merged into trunk ...`, when the end
  runs, then it exits 1 with `closed tab foo; ended session <id>; worktree close: exit 1:` and that line.
- (SUA-3) Given a merge whose `cardinal check` fails, when the user pressed `f`, then no end is chained and the
  session still runs.
- (SUA-11) Given no `ZELLIJ` variable, when `cardinal-ui --release` runs, then it prints `release needs zellij` on
  stderr, exits 1 and runs neither `release --json` nor `zellij`.
- (SUA-11) Given a project that turned the skill `release` off, when `cardinal-ui --release` runs, then it exits 1
  with `the skill release is turned off` and opens no tab.
- (SUA-11) Given a `release --json` that lists no milestone, when `cardinal-ui --release` runs, then it exits 1 with
  `no milestone awaits its release` and opens no tab.
- (SUA-11) Given an untracked file in the main worktree, or the main worktree detached, when `cardinal-ui --release`
  runs, then it exits 1 saying why and opens no tab.
- (SUA-11) Given a lock of the main worktree naming a session whose process runs, when `cardinal-ui --release` runs,
  then it exits 1 naming that session; given one whose process ended, it opens the tab.
- (SUA-11) Given a tab `release` whose pane runs `claude /release`, when `cardinal-ui --release` runs, then it exits
  1 with `a release tab is already open`; given that pane exited, or a tab `release` of an issue of that stem, it
  opens the tab.
- (SUA-4) Given a release committed but not tagged (`release --json` prints `"prepared":true` and no milestone),
  when `cardinal-ui --release` runs, then it opens the tab, so `skill:release` resumes it.
- (SUA-11) Given a project that turned the skill `release` off and nothing to release, when `cardinal-ui --release`
  runs, then it exits 1 with `no milestone awaits its release`.
- (SUA-11) Given a `release --json` that exits 3 with `cannot read the tags`, when `cardinal-ui --release` runs, then
  it exits 1 with `release: exit 3: cannot read the tags`.
- (SUA-11) Given a `zellij new-tab` that fails, when `cardinal-ui --release` runs, then it exits 1 naming `zellij
  action new-tab`.
- (SUA-6) Given a release start running, when the user presses `q`, then the screen quits once it has ended.
- (SUA-14) Given `foo` merged with an untracked file in its worktree and `bar` merged and clean, when
  `cardinal-ui --end-merged` runs, then it prints `kept foo: <path> has uncommitted changes or untracked files` and
  `ended bar`, removes the worktree of `bar` only, and exits 1.
- (SUA-14) Given `cardinal-ui --end-merged` run in the merged worktree of `foo`, when it runs, then it keeps `foo`
  with `cardinal-ui runs in <path>: run it from another worktree` and ends the other merged worktrees.
- (SUA-14) Given no linked worktree whose branch has its close on the trunk, when `cardinal-ui --end-merged` runs,
  then it prints `no merged worktree`, runs no `worktree close` and exits 0.
- (SUA-14) Given the main worktree detached, when `cardinal-ui --end-merged` runs, then it exits 1 with `main
  worktree not on a branch` and ends nothing.
- (SUA-14) Given a worktree that served the issue `foo`, merged, and now serves the open issue `bar` on the branch
  `bar` (ADR 0023), when `cardinal-ui --end-merged` runs, then it leaves that worktree as it is.
- (SUA-13) Given a merge running, when the user presses `E`, then it starts nothing.

## Non-goals

- Ending a merged worktree on the read that first sees it merged, without a key: the user ends them with `E`.
- Ending a session at work, removing a branch `worktree close` refuses (`--force`), or closing a tab or a pane the
  UI cannot tie to the issue by its name and its command: the user does it by hand.
- Preparing or tagging a release in the UI itself (the session of SUA-4 runs `skill:release`, its stops included),
  starting it from a linked worktree's branch not merged yet, or showing a release prepared but not tagged yet (its
  milestones hold their `Released:` line, `roadmap --open` leaves them out): later issues of the milestone
  "Releases from the steering UI".
- Rebasing a diverged branch, merging anything but a fast-forward, or pushing the trunk: the rebase is the issue
  session's, the push the user's.
