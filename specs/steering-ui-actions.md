---
id: SUA
status: accepted
adrs: [0022, 0023, 0030, 0031]
---

# Steering UI actions

## Intent

The keys and the flags of `cardinal-ui` (STU) that change the project: start a session on a ready issue, or on the
release of the milestones awaiting it, in a zellij tab of its own, and merge into the trunk an issue closed on its
branch, each refused with its reason when the project is not ready for it; the end of what the merge left is SUE.

## Requirements

### Behaviour

- **SUA-1** (deterministic) In a zellij session, the key `s` on the issue selected in the queue (moved with the arrow keys, `j` and `k`), when its state is `ready` and nothing takes it, must open a worktree for it with `cardinal run worktree open <stem>` and start in it, in a new zellij tab named after its stem, an interactive `claude` whose first message hands `issues/<stem>.md` to `issue-workflow`; when its state is `ready` and it is taken only by a claim in the state `working` whose session ended (CLM-5), it resumes it: open no worktree and start that tab in the worktree of the claim, so the new session takes the claim back (CLM-2); on another issue it starts nothing and says why on the status line.
- **SUA-2** (deterministic) The key `f` on the issue selected in the queue, when a claim in the state `merge pending` takes it (CLM-7), must fast-forward the trunk (the branch of the main worktree, the first one `git worktree list --porcelain` lists) to the branch of that claim with `git merge --ff-only refs/heads/<branch>` run in the main worktree, then run `cardinal check` there, and say on the status line `merged <stem> (<branch>) into <trunk>`; on another issue, or when SUA-7 refuses, it merges nothing and says why on the status line.
- **SUA-3** (deterministic) withdrawn: moved to SUE-1.
- **SUA-13** (deterministic) withdrawn: moved to SUE-2.
- **SUA-4** (deterministic) In a zellij session, the key `v`, when a milestone awaits its release (STU-1) or a release prepared is not tagged yet (an interrupted release that `skill:release` resumes), must start in the main worktree, in a new zellij tab named `release`, an interactive `claude` whose first message is `/release`, so the version, its reread, its review and its tag stay with `skill:release`; when SUA-11 refuses, it starts nothing and says why on the status line.
- **SUA-15** (deterministic) The help line under the panels must list `q quit  r refresh  ↑↓ select  tab panel`, then, in this order and each only when its key would act on what is selected in the read shown, `s start` (SUA-1: in a zellij session, the selected issue of the queue `ready` and not taken, or taken only by a `working` claim whose session ended), `f merge` (SUA-2: the selected issue of the queue taken by a claim `merge pending`), `e end` (SUE-1: the selected worktree a linked worktree whose row reads `merged`), `E end merged` (SUE-2: a row of the worktrees panel reads `merged`, whatever the selection), `v release` (SUA-4: in a zellij session, a milestone awaits its release, whatever the selection), then `g charts`, two spaces between two keys; it is drawn anew at each read and each move of the selection; a key left out still acts and refuses as its requirement says when pressed; `--print` (STU-4) shows the line of the selection the screen starts on, the first issue of the queue.

### Contract

- **SUA-5** (deterministic) `cardinal-ui --start <stem>` must start the session of SUA-1 on the issue `<stem>` and exit 0, or print on stderr why it does not and exit 1: outside a zellij session (no `ZELLIJ` variable: `start needs zellij: run cardinal-ui in a zellij session`), an issue that is not open, not `ready`, or taken other than by a claim whose session ended, a claim whose session ended in a worktree that `git worktree list` no longer lists (`<stem>: worktree <path> gone: cardinal run issue-claim --release <stem>`) or whose lock (STU-7) names a session that runs (`<stem>: session <id> works in <path>`), or a step that fails (its command, its exit code and the first line of its stderr), running no step after a failed one; the key `s` shows the same message on the status line.
- **SUA-6** (deterministic) `q`, `Esc` or `Ctrl+C` pressed during a start (of an issue or of the release), a merge or an end (SUE-1) must quit once it has ended, so a start never stops between the worktree and the tab, a merge before its `cardinal check` nor an end between the session and the worktree, whose result is then printed on stderr once the terminal is given back; `s`, `f`, `e`, `E` and `v` start nothing while one runs.
- **SUA-7** (deterministic) `cardinal-ui --merge <stem>` must do the merge of SUA-2 on the issue `<stem>` and exit 0, or print on stderr why it does not and exit 1, running no `git merge` when: `<stem>` is not open or no claim in the state `merge pending` takes it (`<stem>: not merge pending`); `<branch>` does not hold `issues/done/<stem>.md` (`<branch> does not hold the close of <stem>: commit it first`); the repository is bare (`no main worktree: the repository is bare`); the main worktree has no branch checked out (`main worktree not on a branch`); `git status --porcelain` lists anything there (`main worktree has uncommitted changes or untracked files`); its lock (STU-7) names a session whose process runs (`session <id> works in the main worktree`); the trunk is not an ancestor of `<branch>` (`<branch> diverged from <trunk>: rebase it in its worktree`); a git command it runs fails (its command, its exit code and the first line of its stderr). A `cardinal check` that fails after the merge keeps the merge, exits 1 and prints `merged <stem> (<branch>) into <trunk>; cardinal check: exit <n>: <first line of its stderr>`, the line that names the install it needs. The key `f` shows the same message on the status line.
- **SUA-8** (deterministic) withdrawn: moved to SUE-3.
- **SUA-9** (deterministic) withdrawn: moved to SUE-4.
- **SUA-10** (deterministic) The `zellij` it runs must be the program named by `CARDINAL_UI_ZELLIJ`, else `zellij` found on the `PATH`, the tab opened with `zellij action new-tab --cwd <worktree path> --name <stem> -- claude "/issue-workflow issues/<stem>.md"`; outside a zellij session, or when `list-tabs` fails, the queue shows no `tab` and no error.
- **SUA-11** (deterministic) `cardinal-ui --release` must start the session of SUA-4 and exit 0, or print on stderr why it does not and exit 1, opening no tab when: outside a zellij session (`release needs zellij: run cardinal-ui in a zellij session`); the repository is bare (`no main worktree: the repository is bare`); the main worktree has no branch checked out (`main worktree not on a branch`); `git status --porcelain` lists anything there (`main worktree has uncommitted changes or untracked files`); its lock (STU-7) names a session whose process runs (`session <id> works in the main worktree`); a tab named `release` holds a terminal pane whose command is `claude /release` and has not `exited` (read from `list-panes`, one that fails counting as no such pane) (`a release tab is already open`); `cardinal run release --json`, run in the main worktree, lists no milestone and is not `prepared` (REL-15) (`no milestone awaits its release`), or cannot be read (the line of STU-6); the main worktree has no `.claude/skills/release/SKILL.md`, the project having turned the skill off (`the skill release is turned off: run the steps cardinal run release prints`), checked once `release --json` found a release to start; a step that fails (its command, its exit code and the first line of its stderr). The key `v` shows the same message on the status line.
- **SUA-12** (deterministic) The tab of SUA-4 must be opened with `zellij action new-tab --cwd <main worktree path> --name release -- claude /release`, the `zellij` of SUA-10, run once every check of SUA-11 passed.
- **SUA-14** (deterministic) withdrawn: moved to SUE-5.

## Scenarios

The happy paths SUA-H3, SUA-H4 and SUA-H6 moved to SUE-H2, SUE-H1 and SUE-H3 of `specs/steering-ui-end.md`.

- **SUA-H1** (happy; SUA-1, SUA-5, SUA-10) Given the UI in a zellij session and a ready issue `foo` nothing
  takes, when the user selects it and presses `s`, then the worktree `<main worktree>-foo` is opened and a tab `foo`
  runs `claude` on `issues/foo.md` there; the next read shows `foo` taken with `tab`.
- **SUA-H2** (happy; SUA-2, SUA-7) Given an issue `foo` closed on branch `foo` of a linked worktree, its claim
  `merge pending`, and a clean main worktree on `trunk` with no live session, when the user selects `foo` and presses
  `f`, then `trunk` points at the commit of `foo`, `cardinal check` ran in the main worktree, the status line reads
  `merged foo (foo) into trunk`, and the next read no longer lists `foo` (CLM-6).
- **SUA-H5** (happy; SUA-4, SUA-11, SUA-12) Given the UI in a zellij session in a linked worktree, a milestone that
  reads `awaits release v0.3.0`, and a clean main worktree on `trunk` with no live session, when the user presses
  `v`, then a tab `release` runs `claude /release` in the main worktree, and the linked worktree is unchanged.
- **SUA-H7** (happy; SUA-15) Given a queue whose first issue `foo` is taken by a claim `merge pending` and a
  worktree of an issue `bar` whose row reads `merged`, when the user looks at the help line, then it reads `q quit  r
  refresh  ↑↓ select  tab panel  f merge  E end merged  g charts`; when they press `Tab` and select the worktree of `bar`, it
  reads `… tab panel  e end  E end merged  g charts`.
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
- (SUA-15) Given a ready issue nothing takes selected outside a zellij session, then the help line offers no `s start`
  nor `v release`, and `s` still says `start needs zellij: run cardinal-ui in a zellij session`.
- (SUA-15) Given an empty queue and no worktree `merged`, then the help line reads `q quit  r refresh  ↑↓ select  tab
  panel  g charts`.
- (SUA-15) Given the selected issue taken by a live `working` claim, then the help line offers no `s start` nor `f
  merge`.

## Non-goals

- Preparing or tagging a release in the UI itself (the session of SUA-4 runs `skill:release`, its stops included),
  starting it from a linked worktree's branch not merged yet, or showing a release prepared but not tagged yet (its
  milestones hold their `Released:` line, `roadmap --open` leaves them out): later issues of the milestone
  "Releases from the steering UI".
- Rebasing a diverged branch, merging anything but a fast-forward, or pushing the trunk: the rebase is the issue
  session's, the push the user's.
- Offering `v release` for a release prepared but not tagged yet, which the read of the panels does not see (`roadmap
  --open` leaves its milestones out): the key still resumes it (SUA-4), the help line does not list it.
