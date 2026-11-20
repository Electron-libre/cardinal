---
id: STU
status: draft
adrs: [0011, 0022, 0030]
---

# Steering UI

## Intent

`cardinal-ui` shows the user, in one terminal screen kept up to date, where a project driven by Cardinal stands: its
milestones, its queue of issues, and which worktree each session works in, so they need not run several commands and
open each worktree to know it. It reads the project and changes nothing.

## Requirements

### Behaviour

- **STU-1** (deterministic) `cardinal-ui`, run in a project, must show in one screen three panels: the milestones (each with its state, `on hold` when it is, and its open issues over the listed ones), the queue (each open issue in the order of `next-issue`, the one it offers marked, with its priority, its state, its milestone and what takes it: a claim with its worktree and whether its session ended, or a branch) and the worktrees (each with its branch and the session that holds its lock, ended or not, or `free`).
- **STU-2** (deterministic) `cardinal-ui` must read the project again every 2 seconds and on the key `r`, and quit on `q` or `Esc`, giving the terminal back as it found it.
- **STU-3** (deterministic) `cardinal-ui` must write no file and no git ref of its own: it reads the project only through `cardinal run next-issue --json`, `cardinal run roadmap --json` and git commands that read.
- **STU-4** (deterministic) `cardinal-ui --print` must print the screen of STU-1 once as text, 100 columns wide, and exit, so a script or a test reads what the user would see.

### Contract

- **STU-5** (deterministic) The `cardinal` it runs must be the program named by the environment variable `CARDINAL_UI_CARDINAL`, else `cardinal` found on the `PATH`, run in the current directory.
- **STU-6** (deterministic) A tool of STU-3 that cannot be run, exits with a code other than 0 and 1, prints no JSON object, or one whose `version` is not `1`, must leave its panel showing one line naming the tool and the cause (the exit code and the first line of its stderr, `not JSON`, or `version <n> unsupported`), the other panels shown as usual; the exit code 1 of `next-issue` (a problem of the issue files, NXT-2) is no such error: the queue is shown with the lines of its stderr under it.
- **STU-7** (deterministic) The worktrees must be those `git worktree list --porcelain` lists, in its order, each with its branch (`detached` when it has none); the session of a worktree is the `session` and `pid` of the file `cardinal-session` of its git directory (WSL-9), ended when no process of that `pid` runs, `free` when the file is missing or holds no `session`.
- **STU-8** (deterministic) `cardinal-ui --print` must exit 0 when every panel was read, 1 when a panel shows the error of STU-6, and 2 on an unknown argument; `cardinal-ui` run outside a git repository shows the error of STU-6 in the worktrees panel.

## Scenarios

- **STU-H1** (happy; STU-1, STU-4, STU-7) Given a project with a milestone of two issues, one of them claimed by a
  live session in a second worktree, when the user runs `cardinal-ui --print`, then they read the milestone with
  `2/2 open`, the queue with the free issue marked next and the other one taken in that worktree, and the two
  worktrees, the second held by the live session.
- (STU-1) Given an issue claimed by a session whose process ended, when `cardinal-ui --print` runs, then the queue shows its claim as ended.
- (STU-1) Given a milestone with `- On hold: later`, when `cardinal-ui --print` runs, then its row says `on hold`.
- (STU-6) Given a `cardinal` that exits 3 with `cannot run the tool` on stderr for `roadmap`, when `cardinal-ui --print` runs, then the milestones panel reads `roadmap: exit 3: cannot run the tool`, the queue and the worktrees are shown, and it exits 1.
- (STU-6) Given a `cardinal` whose `roadmap --json` prints `{"version":2,"milestones":[]}`, when `cardinal-ui --print` runs, then the milestones panel reads `roadmap: version 2 unsupported`.
- (STU-6) Given a `cardinal` whose `next-issue --json` prints text, when `cardinal-ui --print` runs, then the queue panel reads `next-issue: not JSON`.
- (STU-6) Given issue files of which one has no priority, when `cardinal-ui --print` runs, then the queue is shown with the line `issue without a priority: <stem>` under it, and it exits 0.
- (STU-7) Given a worktree with no `cardinal-session` file, when `cardinal-ui --print` runs, then its row reads `free`.
- (STU-8) Given a directory outside any git repository, when `cardinal-ui --print` runs, then the worktrees panel shows an error and it exits 1.
- (STU-8) Given `cardinal-ui --prnt`, then it prints a usage line on stderr and exits 2.

## Non-goals

- Starting, stopping or entering a session or a worktree, or editing an issue or a milestone: later issues of the
  milestone "Steering UI".
- The measures of `cardinal-report`: a later view.
- Following the changes of the files as they happen: the interval of STU-2 bounds how stale the screen is.
- Pruning what `next-issue` prunes on its own (CLM-6): `cardinal-ui` runs the tool, which may delete the claims of
  issues the trunk closed, as it does for the agent.
- The layout, the colours and the keys beyond those of STU-2: they may change without a requirement.
