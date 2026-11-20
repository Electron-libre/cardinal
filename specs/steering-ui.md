---
id: STU
status: accepted
adrs: [0011, 0022, 0023, 0030, 0031]
---

# Steering UI

## Intent

`cardinal-ui` shows the user, in one terminal screen kept up to date, where a project driven by Cardinal stands: its
milestones, its queue of issues, the worktree each session works in and the trend of its measures, so they need not
run several commands nor open each worktree. It changes the project only to start a session on a ready issue in a
zellij tab of its own, to merge into the trunk an issue closed on its branch, and to end what a merged issue left.

## Requirements

### Behaviour

- **STU-1** (deterministic) `cardinal-ui`, run in a project, must show in one screen three panels, above the measures of STU-16: the milestones not yet released (each with its state, `on hold` when it is, `awaits release v<version>` when the milestones of `cardinal run release --json` (REL-15) hold one of its name and its document, `<version>` the `proposed` of that document, the bare `awaits release` when it is `null`, and its open issues over the listed ones), the queue (each open issue in the order of `next-issue`, the one it offers marked, with its priority, its state, its milestone and what takes it: a claim with its worktree and whether its session ended, or a branch) and the worktrees (each with its branch and the session that holds its lock, ended or not, or `free`).
- **STU-2** (deterministic) `cardinal-ui` must read the project again every 2 seconds and on the key `r`, one read at a time and apart from the keys, so a slow read never keeps a key from being answered, and quit on `q`, `Esc` or `Ctrl+C`, giving the terminal back as it found it.
- **STU-3** (deterministic) `cardinal-ui` must write no git ref of its own and no file outside the cache directory of STU-18: it reads the project only through `cardinal run next-issue --json`, `cardinal run roadmap --open --json`, `cardinal run release --json` when a milestone that `roadmap` lists has the state `done` (run again only once the commit of `HEAD`, a tag or the document of `roadmap` changed since its last run), the measure of STU-17, git commands that read and, in a zellij session, `zellij action list-tabs --json` and, at an end, `zellij action list-panes --json --all`, and changes it only through the start of STU-9 and STU-11, the merge of STU-20 and STU-21 and the end of STU-22 and STU-24.
- **STU-9** (deterministic) In a zellij session, the key `s` on the issue selected in the queue (moved with the arrow keys, `j` and `k`), when its state is `ready` and nothing takes it, must open a worktree for it with `cardinal run worktree open <stem>` and start in it, in a new zellij tab named after its stem, an interactive `claude` whose first message hands `issues/<stem>.md` to `issue-workflow`; on another issue it starts nothing and says why on the status line.
- **STU-10** (deterministic) The queue must show, after an issue, `tab` when a tab of the current zellij session is named after its stem.
- **STU-13** (deterministic) The worktrees panel must show, after the session of a worktree, `waiting since <HH:MM>: <reason>` (local time, its date `<YYYY-MM-DD>` before it when it is not today) when that session runs and the mark of SWT-3 of the worktree names it.
- **STU-16** (deterministic) `cardinal-ui` must show a fourth panel, `Measures`, from the last measure of STU-17: a line `measured <HH:MM>` (its date before it when it is not today, as STU-13), then, for each group of the efficiency table of `cardinal-report --project <project> --last 4 --json` (REP-17, REP-18, REP-21), its `group`, `sessions` and the medians `input_tokens` with `cache_read_share`, `output_tokens`, `main_context_peak` with `peak_sessions`, and `validations`, or `too few` after its `sessions`; then, for each group of the quality table of `cardinal-report --by milestone --json` (REP-20), its milestone, `issues`, `fixes_after_per_issue`, `findings_per_review` with `counted_reviews`, and `input_tokens_per_issue` with `token_issues`, each sample size written next to the number it bears on; `<project>` is the last component of `<main>` (STU-17), as EVA-7 names a project.
- **STU-17** (deterministic) A measure must run `cardinal run session-metrics` and `cardinal run delivered-quality <main> --summary`, `<main>` the repository its sessions resolve to (EVA-7: the directory that holds its git common directory when that is a `.git`, else the top level of the root), into the files `sessions.tsv` and `quality.tsv` of the cache directory, then the two `cardinal-report` of STU-16 with `--sessions` and `--quality` naming them, into `efficiency.json` and `milestones.json` beside them (each file written first under a temporary name, the reports reading the new `sessions.tsv` and `quality.tsv` under theirs); `cardinal-ui` starts one when it starts and the last measure is missing or more than one hour old, and on the key `m`, one at a time and apart from the keys and the reads of STU-2, the panel showing the last measure with `measuring` on its first line while one runs; `cardinal-ui --print` measures first when the last measure is missing or more than one hour old.
- **STU-20** (deterministic) The key `f` on the issue selected in the queue, when a claim in the state `merge pending` takes it (CLM-7), must fast-forward the trunk (the branch of the main worktree, the first one `git worktree list --porcelain` lists) to the branch of that claim with `git merge --ff-only refs/heads/<branch>` run in the main worktree, then run `cardinal check` there, and say on the status line `merged <stem> (<branch>) into <trunk>`; on another issue, or when STU-21 refuses, it merges nothing and says why on the status line.
- **STU-22** (deterministic) The key `e` on the worktree selected in the worktrees panel (`Tab` moves the selection between the queue and the worktrees, the arrow keys, `j` and `k` within the panel, kept on the same path across reads and visible as STU-14 keeps an issue), when it is a linked worktree whose branch `<stem>` has its close on the trunk, must end its session, close the zellij tab the UI opened for it, and remove the worktree with its branch through `cardinal run worktree close`, and say on the status line `ended <stem>`; the merge of STU-20 chains the same end on its issue, on the linked worktree of the branch of its claim, once its `cardinal check` passed, its status line `merged <stem> (<branch>) into <trunk>; ended <stem>`; on another worktree, or when STU-24 refuses, it ends nothing and says why on the status line.
- **STU-23** (deterministic) The worktrees panel must show `merged` after the branch of a linked worktree when the trunk holds `issues/done/<branch>.md`.
- **STU-4** (deterministic) `cardinal-ui --print` must print the screen of STU-1 once as text, whole whatever its number of lines, 100 columns wide or as wide as its longest line needs (in display columns), and exit, so a script or a test reads what the user would see.

### Contract

- **STU-5** (deterministic) The `cardinal` it runs must be the program named by the environment variable `CARDINAL_UI_CARDINAL`, else `cardinal` found on the `PATH`, run in the root of the work tree of the current directory (the current directory outside a git repository), as are the git commands.
- **STU-6** (deterministic) A tool of STU-3 that cannot be run, exits with a code other than 0 (other than 0 and 1 for `next-issue`), prints no JSON object, or one whose `version` is not `1`, must leave its panel showing one line naming the tool and the cause (the exit code and the first line of its stderr, `not JSON`, or `version <n> unsupported`), the other panels shown as usual; the exit code 1 of `next-issue` (a problem of the issue files, NXT-2) is no such error: the queue is shown with the lines of its stderr under it. A `release` in such an error leaves the milestones of `roadmap` shown, none of them marked `awaits release`, with its line under them, the panel counting as failed for STU-8.
- **STU-7** (deterministic) The worktrees must be those `git worktree list --porcelain` lists, in its order, each with its branch (`detached` when it has none); the session of a worktree is the `session` and `pid` of the file `cardinal-session` of its git directory (WSL-9), ended when no process of that `pid` runs, `free` when the file is missing or lacks the `session` or the `pid` (as `workspace-lock` reads it).
- **STU-8** (deterministic) `cardinal-ui --print` must exit 0 when every panel was read, 1 when a panel shows the error of STU-6, and `cardinal-ui` 2 on an unknown argument; `cardinal-ui` run outside a git repository shows the error of STU-6 in the worktrees panel.

- **STU-11** (deterministic) `cardinal-ui --start <stem>` must start the session of STU-9 on the issue `<stem>` and exit 0, or print on stderr why it does not and exit 1: outside a zellij session (no `ZELLIJ` variable: `start needs zellij: run cardinal-ui in a zellij session`), an issue that is not open, not `ready` or taken, or a step that fails (its command, its exit code and the first line of its stderr), running no step after a failed one; the key `s` shows the same message on the status line.
- **STU-14** (deterministic) The selection of STU-9 must stay on the same issue when a read changes the order of the queue, move to the line it had when its issue left the queue, and stay visible however long the queue, so `s` never starts an issue the user did not see selected.
- **STU-15** (deterministic) `q`, `Esc` or `Ctrl+C` pressed during a start, a merge or an end must quit once it has ended, so a start never stops between the worktree and the tab, a merge before its `cardinal check` nor an end between the session and the worktree, whose result is then printed on stderr once the terminal is given back; `s`, `f` and `e` start nothing while one runs.
- **STU-21** (deterministic) `cardinal-ui --merge <stem>` must do the merge of STU-20 on the issue `<stem>` and exit 0, or print on stderr why it does not and exit 1, running no `git merge` when: `<stem>` is not open or no claim in the state `merge pending` takes it (`<stem>: not merge pending`); `<branch>` does not hold `issues/done/<stem>.md` (`<branch> does not hold the close of <stem>: commit it first`); the repository is bare (`no main worktree: the repository is bare`); the main worktree has no branch checked out (`main worktree not on a branch`); `git status --porcelain` lists anything there (`main worktree has uncommitted changes or untracked files`); its lock (STU-7) names a session whose process runs (`session <id> works in the main worktree`); the trunk is not an ancestor of `<branch>` (`<branch> diverged from <trunk>: rebase it in its worktree`); a git command it runs fails (its command, its exit code and the first line of its stderr). A `cardinal check` that fails after the merge keeps the merge, exits 1 and prints `merged <stem> (<branch>) into <trunk>; cardinal check: exit <n>: <first line of its stderr>`, the line that names the install it needs. The key `f` shows the same message on the status line.
- **STU-24** (deterministic) `cardinal-ui --end <stem>` must do the end of STU-22 on the linked worktree whose branch is `<stem>` and exit 0, or print on stderr why it does not and exit 1, ending, closing and removing nothing when: no linked worktree has the branch `<stem>` checked out (`<stem>: no linked worktree on branch <stem>`); the main worktree has no branch checked out (`main worktree not on a branch`); the trunk does not hold `issues/done/<stem>.md` (`<trunk> does not hold the close of <stem>: merge it first`); `git status --porcelain` lists anything in that worktree (`<path> has uncommitted changes or untracked files`); the root `cardinal-ui` runs in is that worktree or inside it (`cardinal-ui runs in <path>: run it from another worktree`); its session (STU-7) runs and the mark of SWT-3 of the worktree does not name it (`session <id> is at work in <path>`). The key `e` shows the same message on the status line.
- **STU-25** (deterministic) The end must run its steps in this order, each only once the one before succeeded: in a zellij session, when a tab named `<stem>` holds a terminal pane whose command is `claude /issue-workflow issues/<stem>.md` (STU-12, read from `list-panes`), close that tab with `zellij action close-tab-by-id <tab id>` when that pane is its only terminal pane, else that pane alone with `zellij action close-pane --pane-id terminal_<pane id>` (a pane whose command `exited`, or a `list-panes` that fails, counts as no such pane); when no such pane was closed and the session runs, send `SIGHUP` to its `pid` (a `kill` that fails once that process ended is no failure); then wait at most 10 seconds for that process to end (`session <id> still runs after 10 s`); then run `cardinal run worktree close <path of the worktree>` in the main worktree. A `pid` whose process started more than 2 seconds after the lock was last written is another process that took the number: the session counts as ended, its process gets no signal. A step that fails ends it there with its command, its exit code and the first line of its stderr, after what the steps before it did (`closed tab <stem>` or `closed pane terminal_<pane id>`, `ended session <id>`), so the user knows what is left.
- **STU-12** (deterministic) The `zellij` it runs must be the program named by `CARDINAL_UI_ZELLIJ`, else `zellij` found on the `PATH`, the tab opened with `zellij action new-tab --cwd <worktree path> --name <stem> -- claude "/issue-workflow issues/<stem>.md"`; outside a zellij session, or when `list-tabs` fails, the queue shows no `tab` and no error.
- **STU-18** (deterministic) The cache directory must be `<cache>/cardinal-ui/<project>-<hash>`, `<cache>` the value of `XDG_CACHE_HOME` when it is an absolute path, else `$HOME/.cache`, and `<hash>` the 64-bit FNV-1a hash of the path of `<main>` in 16 hexadecimal digits, so two projects of one name keep their own; a measure writes each file whole (a temporary file of that directory, named after its process so two measures at once never share one, renamed over it, `efficiency.json` last), removes the temporary files of the directory over an hour old when it starts, and the age of the last measure is the modification time of `efficiency.json`.
- **STU-19** (deterministic) A step of a measure that cannot be run or exits with a code other than 0, or a report that prints no JSON object or one whose `version` is not `1`, must end the measure, leave the four files of the last measure unchanged (a measure renames its files into place only once every step succeeded), and leave the `Measures` panel showing the line of STU-6 naming the step (`session-metrics`, `delivered-quality`, `cardinal-report`) above the last measure, the panel counting as failed for STU-8.

## Scenarios

- **STU-H5** (happy; STU-20, STU-21) Given an issue `foo` closed on branch `foo` of a linked worktree, its claim
  `merge pending`, and a clean main worktree on `trunk` with no live session, when the user selects `foo` and presses
  `f`, then `trunk` points at the commit of `foo`, `cardinal check` ran in the main worktree, the status line reads
  `merged foo (foo) into trunk`, and the next read no longer lists `foo` (CLM-6).
- **STU-H6** (happy; STU-20, STU-22, STU-25) Given the issue `foo` of STU-H5 worked by a session waiting for the user
  in the tab `foo` the UI opened, when the user presses `f` on it, then the trunk holds `foo`, the tab `foo` is
  closed, the session ran its `SessionEnd` hooks and ended, the worktree and the branch `foo` are removed, and the
  status line reads `merged foo (foo) into trunk; ended foo`.
- **STU-H7** (happy; STU-22, STU-23, STU-24) Given an issue `foo` merged into the trunk by hand, its worktree still
  open, when the user moves the selection to the worktrees panel, the row of `foo` reads `merged`, and pressing `e`
  on it closes its tab and removes the worktree and the branch.
- **STU-H1** (happy; STU-1, STU-4, STU-7) Given a project with a milestone of two issues, one of them claimed by a
  live session in a second worktree, when the user runs `cardinal-ui --print`, then they read the milestone with
  `2/2 open`, the queue with the free issue marked next and the other one taken in that worktree, and the two
  worktrees, the second held by the live session.
- **STU-H2** (happy; STU-9, STU-11, STU-12) Given the UI in a zellij session and a ready issue `foo` nothing
  takes, when the user selects it and presses `s`, then the worktree `<main worktree>-foo` is opened and a tab `foo`
  runs `claude` on `issues/foo.md` there; the next read shows `foo` taken with `tab`.
- **STU-H3** (happy; STU-13) Given a session whose turn ended in a linked worktree, when the UI reads the project,
  then the row of that worktree reads `waiting since <HH:MM>: stop`.
- **STU-H4** (happy; STU-16, STU-17, STU-18) Given a project whose sessions span two groups of revisions and whose
  done issues belong to a milestone, when the user runs `cardinal-ui` with no measure cached, then the `Measures`
  panel reads `measuring`, and once the measure ends it shows the two groups with their sessions and medians and the
  milestone with its issues and ratios; run again within the hour, it shows that measure at once without measuring.
- (STU-1) Given an issue claimed by a session whose process ended, when `cardinal-ui --print` runs, then the queue shows its claim as ended.
- (STU-1) Given a milestone with `- On hold: later`, when `cardinal-ui --print` runs, then its row says `on hold`.
- (STU-6) Given a `cardinal` that exits 3 with `cannot run the tool` on stderr for `roadmap`, when `cardinal-ui --print` runs, then the milestones panel reads `roadmap: exit 3: cannot run the tool`, the queue and the worktrees are shown, and it exits 1.
- (STU-6) Given a `cardinal` whose `roadmap --json` prints `{"version":2,"milestones":[]}`, when `cardinal-ui --print` runs, then the milestones panel reads `roadmap: version 2 unsupported`.
- (STU-6) Given a `cardinal` whose `next-issue --json` prints text, when `cardinal-ui --print` runs, then the queue panel reads `next-issue: not JSON`.
- (STU-6) Given issue files of which one has no priority, when `cardinal-ui --print` runs, then the queue is shown with the line `issue without a priority: <stem>` under it, and it exits 0.
- (STU-1) Given `New` (`done`, document `ROADMAP.md`) listed by `roadmap --open --json`, and a `release --json` that
  prints `"proposed":"0.3.0"` and lists `New` of `ROADMAP.md`, when `cardinal-ui --print` runs, then the row of `New`
  reads `awaits release v0.3.0`.
- (STU-1) Given the same milestone and a `release --json` that prints `"proposed":null` and lists it, then its row
  reads `awaits release` with no version.
- (STU-1) Given a `release --json` that lists `New` of another document, then the row of `New` shows no `awaits
  release`.
- (STU-3) Given no milestone in the state `done`, when `cardinal-ui --print` runs, then it does not run `release`.
- (STU-6) Given a `done` milestone and a `cardinal` that exits 3 with `cannot read the tags` on stderr for `release`,
  when `cardinal-ui --print` runs, then the milestones panel shows the milestone without `awaits release` and the
  line `release: exit 3: cannot read the tags` under it, and it exits 1.
- (STU-6) Given a `release --json` that prints `{"version":2}`, then the milestones panel holds the line `release:
  version 2 unsupported` under its milestones.
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
- (STU-17) Given a measure 30 minutes old, when `cardinal-ui --print` runs, then it runs no `session-metrics` and
  shows that measure; given one two hours old, it measures first.
- (STU-17) Given the UI run in a linked worktree, when it measures, then `delivered-quality` reads the main worktree.
- (STU-17) Given a project whose git directory is separate (`git init --separate-git-dir`), when it measures, then
  `delivered-quality` reads its work tree and the project is named after it.
- (STU-18) Given two projects named `repo` in two directories, when each measures, then each has its own cache
  directory.
- (STU-18) Given a temporary file left two hours ago by a measure whose process ended, when a measure starts, then it
  is removed.
- (STU-18) Given `XDG_CACHE_HOME` set to `cache` (a relative path), then the measure is written under
  `$HOME/.cache/cardinal-ui/<project>`.
- (STU-19) Given a `cardinal` whose `session-metrics` exits 3 with `cannot read` on stderr and a measure cached, when
  `cardinal-ui --print` measures, then the `Measures` panel reads `session-metrics: exit 3: cannot read` above the
  cached groups, its `efficiency.json` is unchanged and it exits 1.
- (STU-16) Given a last group of 4 sessions, then its line reads its `sessions` and `too few`, no median.
- (STU-21) Given a ready issue `bar` nothing takes, when `cardinal-ui --merge bar` runs, then it exits 1 with `bar:
  not merge pending` and the trunk is unchanged.
- (STU-21) Given a tracked file modified, or an untracked file, in the main worktree, when `cardinal-ui --merge foo`
  runs, then it exits 1 saying the main worktree has uncommitted changes or untracked files, and runs no merge.
- (STU-21) Given `close-issue` run on `foo` but its move not committed yet, when `cardinal-ui --merge foo` runs, then
  it exits 1 with `foo does not hold the close of foo: commit it first` and the trunk is unchanged.
- (STU-21) Given a tag `foo` on an older commit than the branch `foo`, when it merges, then the trunk points at the
  branch.
- (STU-21) Given the main worktree detached, when `cardinal-ui --merge foo` runs, then it exits 1 with `main worktree
  not on a branch`.
- (STU-21) Given a lock of the main worktree naming a session whose process runs, when `cardinal-ui --merge foo`
  runs, then it exits 1 naming that session; given one whose process ended, it merges.
- (STU-21) Given a commit on `trunk` that `foo` lacks, when `cardinal-ui --merge foo` runs, then it exits 1 with `foo
  diverged from trunk: rebase it in its worktree` and the trunk is unchanged.
- (STU-21) Given a `cardinal check` that exits 1 with `the installed cardinal cannot run the hook ...: install it
  with ...`, when `cardinal-ui --merge foo` runs, then the trunk holds `foo`, and it exits 1 printing `merged foo
  (foo) into trunk; cardinal check: exit 1:` and that line.
- (STU-21) Given `cardinal-ui` run in a linked worktree, when it merges, then the trunk of the main worktree moves and
  the linked worktree is unchanged.
- (STU-15) Given a merge running, when the user presses `q`, then the screen quits once `cardinal check` has ended.
- (STU-24) Given the session of `foo` running with no wait mark, when `cardinal-ui --end foo` runs, then it exits 1
  with `session <id> is at work in <path>`, and the tab, the session and the worktree are kept.
- (STU-24) Given a trunk that does not hold `issues/done/foo.md`, when `cardinal-ui --end foo` runs, then it exits 1
  with `trunk does not hold the close of foo: merge it first` and runs no `zellij` nor `worktree close`.
- (STU-24) Given an untracked file in the worktree of `foo`, when `cardinal-ui --end foo` runs, then it exits 1 and
  the session still runs.
- (STU-24) Given `cardinal-ui` run in the worktree of `foo`, when it ends `foo`, then it exits 1 naming that
  worktree and ends nothing.
- (STU-24) Given no linked worktree on the branch `foo` (the main worktree is on it, or none is), when
  `cardinal-ui --end foo` runs, then it exits 1 with `foo: no linked worktree on branch foo`.
- (STU-25) Given the tab `foo` holding the session's pane and a shell the user opened, when the end runs, then only
  the session's pane is closed and the tab stays.
- (STU-25) Given a waiting session started by hand, with no tab of the UI, when the end runs, then its process gets
  `SIGHUP`, and the worktree is removed once it ended.
- (STU-25) Given a session whose process ignores `SIGHUP`, when the end runs, then it exits 1 with `session <id> still
  runs after 10 s` and the worktree is kept.
- (STU-25) Given a session that already ended and no tab, when the end runs, then it only runs `worktree close`.
- (STU-25) Given a `list-panes` that fails, or a pane of the session whose command exited, when the end runs, then it
  closes no tab, sends `SIGHUP` to the session and removes the worktree.
- (STU-25) Given a lock written ten minutes before the process of its `pid` started, when the end runs, then that
  process gets no signal and the worktree is removed.
- (STU-22) Given the claim of `foo` on the branch `feature-foo`, when `f` merges it, then the end runs on the worktree
  of `feature-foo`.
- (STU-25) Given a `worktree close` that exits 1 with `the branch foo is not merged into trunk ...`, when the end
  runs, then it exits 1 with `closed tab foo; ended session <id>; worktree close: exit 1:` and that line.
- (STU-22) Given a merge whose `cardinal check` fails, when the user pressed `f`, then no end is chained and the
  session still runs.
- (STU-23) Given a linked worktree whose branch has no close on the trunk, then its row shows no `merged`, and `e` on
  it ends nothing and says why.

## Non-goals

- Entering a session, ending a session whose issue is not merged, reusing a free worktree for a new issue, or editing
  an issue or a milestone: later issues of the milestone "Steering UI".
- Ending a session at work, removing a branch `worktree close` refuses (`--force`), or closing a tab or a pane the
  UI cannot tie to the issue by its name and its command: the user does it by hand.
- Running as a zellij plugin, or starting a session in another terminal multiplexer.
- Telling a question from a validation stop: SWT marks every end of a turn.
- The other tables of `cardinal-report` (rules, skills, fixed context, verdicts), its reports by setup and the
  verdict of a trend: `skill:evaluate-harness` reads them; the panel never turns a number into a pass or a fail.
- A faster reading of the transcripts: the measure takes the time of the readers, the cache hides it.
- Preparing or tagging a release from the UI, or showing a release prepared but not tagged yet (its milestones hold
  their `Released:` line, `roadmap --open` leaves them out): later issues of the milestone "Releases from the
  steering UI".
- Following the changes of the files as they happen: the interval of STU-2 bounds how stale the screen is.
- Rebasing a diverged branch, merging anything but a fast-forward, or pushing the trunk: the rebase is the issue
  session's, the push the user's.
- Pruning what `next-issue` prunes on its own (CLM-6): `cardinal-ui` runs the tool, which may delete the claims of
  issues the trunk closed, as it does for the agent.
- The layout, the colours and the keys beyond those of STU-2: they may change without a requirement.
