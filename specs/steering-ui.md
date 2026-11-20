---
id: STU
status: accepted
adrs: [0011, 0022, 0023, 0030, 0031]
---

# Steering UI

## Intent

`cardinal-ui` shows the user, in one terminal screen kept up to date, where a project driven by Cardinal stands: its
milestones, its queue of issues, and which worktree each session works in, so they need not run several commands and
open each worktree to know it, and starts an interactive Claude Code session on a ready issue in a zellij tab of its own. It
changes the project only when the user starts a session.

## Requirements

### Behaviour

- **STU-1** (deterministic) `cardinal-ui`, run in a project, must show in one screen three panels: the milestones not yet released (each with its state, `on hold` when it is, and its open issues over the listed ones), the queue (each open issue in the order of `next-issue`, the one it offers marked, with its priority, its state, its milestone and what takes it: a claim with its worktree and whether its session ended, or a branch) and the worktrees (each with its branch and the session that holds its lock, ended or not, or `free`).
- **STU-2** (deterministic) `cardinal-ui` must read the project again every 2 seconds and on the key `r`, one read at a time and apart from the keys, so a slow read never keeps a key from being answered, and quit on `q`, `Esc` or `Ctrl+C`, giving the terminal back as it found it.
- **STU-3** (deterministic) `cardinal-ui` must write no file and no git ref of its own: it reads the project only through `cardinal run next-issue --json`, `cardinal run roadmap --open --json`, git commands that read and, in a zellij session, `zellij action list-tabs --json`, and changes it only through the start of STU-9 and STU-11.
- **STU-9** (deterministic) In a zellij session, the key `s` on the issue selected in the queue (moved with the arrow keys, `j` and `k`), when its state is `ready` and nothing takes it, must open a worktree for it with `cardinal run worktree open <stem>` and start in it, in a new zellij tab named after its stem, an interactive `claude` whose first message hands `issues/<stem>.md` to `issue-workflow`; on another issue it starts nothing and says why on the status line.
- **STU-10** (deterministic) The queue must show, after an issue, `tab` when a tab of the current zellij session is named after its stem.
- **STU-13** (deterministic) The worktrees panel must show, after the session of a worktree, `waiting since <HH:MM>: <reason>` (local time, its date `<YYYY-MM-DD>` before it when it is not today) when that session runs and the mark of SWT-3 of the worktree names it.
- **STU-4** (deterministic) `cardinal-ui --print` must print the screen of STU-1 once as text, whole whatever its number of lines, 100 columns wide or as wide as its longest line needs (in display columns), and exit, so a script or a test reads what the user would see.

### Contract

- **STU-5** (deterministic) The `cardinal` it runs must be the program named by the environment variable `CARDINAL_UI_CARDINAL`, else `cardinal` found on the `PATH`, run in the root of the work tree of the current directory (the current directory outside a git repository), as are the git commands.
- **STU-6** (deterministic) A tool of STU-3 that cannot be run, exits with a code other than 0 (other than 0 and 1 for `next-issue`), prints no JSON object, or one whose `version` is not `1`, must leave its panel showing one line naming the tool and the cause (the exit code and the first line of its stderr, `not JSON`, or `version <n> unsupported`), the other panels shown as usual; the exit code 1 of `next-issue` (a problem of the issue files, NXT-2) is no such error: the queue is shown with the lines of its stderr under it.
- **STU-7** (deterministic) The worktrees must be those `git worktree list --porcelain` lists, in its order, each with its branch (`detached` when it has none); the session of a worktree is the `session` and `pid` of the file `cardinal-session` of its git directory (WSL-9), ended when no process of that `pid` runs, `free` when the file is missing or lacks the `session` or the `pid` (as `workspace-lock` reads it).
- **STU-8** (deterministic) `cardinal-ui --print` must exit 0 when every panel was read, 1 when a panel shows the error of STU-6, and `cardinal-ui` 2 on an unknown argument; `cardinal-ui` run outside a git repository shows the error of STU-6 in the worktrees panel.

- **STU-11** (deterministic) `cardinal-ui --start <stem>` must start the session of STU-9 on the issue `<stem>` and exit 0, or print on stderr why it does not and exit 1: outside a zellij session (no `ZELLIJ` variable: `start needs zellij: run cardinal-ui in a zellij session`), an issue that is not open, not `ready` or taken, or a step that fails (its command, its exit code and the first line of its stderr), running no step after a failed one; the key `s` shows the same message on the status line.
- **STU-14** (deterministic) The selection of STU-9 must stay on the same issue when a read changes the order of the queue, move to the line it had when its issue left the queue, and stay visible however long the queue, so `s` never starts an issue the user did not see selected.
- **STU-15** (deterministic) `q`, `Esc` or `Ctrl+C` pressed during a start must quit once the start has ended, so a start never stops between the worktree and the tab.
- **STU-12** (deterministic) The `zellij` it runs must be the program named by `CARDINAL_UI_ZELLIJ`, else `zellij` found on the `PATH`, the tab opened with `zellij action new-tab --cwd <worktree path> --name <stem> -- claude "/issue-workflow issues/<stem>.md"`; outside a zellij session, or when `list-tabs` fails, the queue shows no `tab` and no error.

## Scenarios

- **STU-H1** (happy; STU-1, STU-4, STU-7) Given a project with a milestone of two issues, one of them claimed by a
  live session in a second worktree, when the user runs `cardinal-ui --print`, then they read the milestone with
  `2/2 open`, the queue with the free issue marked next and the other one taken in that worktree, and the two
  worktrees, the second held by the live session.
- **STU-H2** (happy; STU-9, STU-11, STU-12) Given the UI in a zellij session and a ready issue `foo` nothing
  takes, when the user selects it and presses `s`, then the worktree `<main worktree>-foo` is opened and a tab `foo`
  runs `claude` on `issues/foo.md` there; the next read shows `foo` taken with `tab`.
- **STU-H3** (happy; STU-13) Given a session whose turn ended in a linked worktree, when the UI reads the project,
  then the row of that worktree reads `waiting since <HH:MM>: stop`.
- (STU-1) Given an issue claimed by a session whose process ended, when `cardinal-ui --print` runs, then the queue shows its claim as ended.
- (STU-1) Given a milestone with `- On hold: later`, when `cardinal-ui --print` runs, then its row says `on hold`.
- (STU-6) Given a `cardinal` that exits 3 with `cannot run the tool` on stderr for `roadmap`, when `cardinal-ui --print` runs, then the milestones panel reads `roadmap: exit 3: cannot run the tool`, the queue and the worktrees are shown, and it exits 1.
- (STU-6) Given a `cardinal` whose `roadmap --json` prints `{"version":2,"milestones":[]}`, when `cardinal-ui --print` runs, then the milestones panel reads `roadmap: version 2 unsupported`.
- (STU-6) Given a `cardinal` whose `next-issue --json` prints text, when `cardinal-ui --print` runs, then the queue panel reads `next-issue: not JSON`.
- (STU-6) Given issue files of which one has no priority, when `cardinal-ui --print` runs, then the queue is shown with the line `issue without a priority: <stem>` under it, and it exits 0.
- (STU-7) Given a worktree with no `cardinal-session` file, when `cardinal-ui --print` runs, then its row reads `free`.
- (STU-8) Given a directory outside any git repository, when `cardinal-ui --print` runs, then the worktrees panel shows an error and it exits 1.
- (STU-4) Given 700 open issues, when `cardinal-ui --print` runs, then the first and the last issues are both printed.
- (STU-5) Given the user in a subdirectory of the project, when `cardinal-ui --print` runs, then the tools run at the root and the panels are those of the project.
- (STU-7) Given a lock file that holds a `session` and no `pid`, when `cardinal-ui --print` runs, then the worktree reads `free`.
- (STU-8) Given `cardinal-ui --print | head -1`, then it exits 0 or 1, never on a panic.
- (STU-11) Given no `ZELLIJ` variable, when `cardinal-ui --start foo` runs, then it prints `start needs zellij` on
  stderr, exits 1 and runs neither `cardinal run worktree open` nor `zellij`.
- (STU-11) Given an issue `foo` claimed by a session, when `cardinal-ui --start foo` runs, then it exits 1 saying
  `foo` is taken, and opens nothing.
- (STU-11) Given a `worktree open` that exits 1 with `branch foo is already checked out`, when `cardinal-ui --start
  foo` runs, then it exits 1 with that line and runs no `zellij`.
- (STU-11) Given a `zellij` that fails, when `cardinal-ui --start foo` runs, then it exits 1 naming `zellij` and the
  worktree stays open.
- (STU-13) Given a session waiting since the day before, when the UI reads the project, then the worktree reads
  `waiting since <YYYY-MM-DD> <HH:MM>`.
- (STU-14) Given `foo` selected on the second line, when the next read puts `foo` first, then `foo` is still selected.
- (STU-14) Given 40 issues in a queue panel of 15 rows, when the user moves the selection to the 30th, then it is
  shown.
- (STU-13) Given a mark naming a session whose process ended, when the UI reads the project, then the worktree reads
  `ended` with no `waiting`.
- (STU-13) Given a mark naming another session than the lock's, when the UI reads the project, then no `waiting` is
  shown.
- (STU-12) Given no zellij session, when `cardinal-ui --print` runs, then no issue shows `tab` and it exits 0.
- (STU-8) Given `cardinal-ui --prnt`, then it prints a usage line on stderr and exits 2.

## Non-goals

- Stopping or entering a session, reusing a free worktree for a new issue, closing a worktree, or editing an issue or
  a milestone: later issues of the milestone "Steering UI".
- Running as a zellij plugin, or starting a session in another terminal multiplexer.
- Telling a question from a validation stop: SWT marks every end of a turn.
- The measures of `cardinal-report`: a later view.
- Following the changes of the files as they happen: the interval of STU-2 bounds how stale the screen is.
- Pruning what `next-issue` prunes on its own (CLM-6): `cardinal-ui` runs the tool, which may delete the claims of
  issues the trunk closed, as it does for the agent.
- The layout, the colours and the keys beyond those of STU-2: they may change without a requirement.
