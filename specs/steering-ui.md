---
id: STU
status: accepted
adrs: [0011, 0022, 0023, 0030, 0031]
---

# Steering UI

## Intent

`cardinal-ui` shows the user, in one terminal screen kept up to date, where a project driven by Cardinal stands: its
milestones, its queue of issues, the worktree each session works in and the trend of its measures, so they need not
run several commands nor open each worktree. It changes the project only through the actions of SUA.

## Requirements

### Behaviour

- **STU-1** (deterministic) `cardinal-ui`, run in a project, must show in one screen three panels, above the measures of STU-16: the milestones not yet released (each with its state, `on hold` when it is, `awaits release v<version>` when the milestones of `cardinal run release --json` (REL-15) hold one of its name and its document, `<version>` the `proposed` of that document, the bare `awaits release` when it is `null`, and its open issues over the listed ones), the queue (each open issue in the order of `next-issue`, the one it offers marked, with its priority, its state, its milestone and what takes it: a claim with its worktree and whether its session ended, or a branch) and the worktrees (each with its branch and the session that holds its lock, ended or not, or `free`).
- **STU-2** (deterministic) `cardinal-ui` must read the project again every 2 seconds and on the key `r`, one read at a time and apart from the keys, so a slow read never keeps a key from being answered, and quit on `q`, `Esc` or `Ctrl+C`, giving the terminal back as it found it.
- **STU-3** (deterministic) `cardinal-ui` must write no git ref of its own and no file outside the cache directory of STU-18: it reads the project only through `cardinal run next-issue --json`, `cardinal run roadmap --open --json`, `cardinal run release --json` when a milestone that `roadmap` lists has the state `done` (run again only once the commit of `HEAD`, a tag or the document of `roadmap` changed since its last run) and, in the main worktree, at each start of SUA-4, the measure of STU-17, git commands that read and, in a zellij session, `zellij action list-tabs --json` and, at an end or a start of SUA-4, `zellij action list-panes --json --all`, and changes it only through the start of SUA-1, SUA-5 and SUA-4, the merge of SUA-2 and SUA-7 and the end of SUA-3 and SUA-8.
- **STU-9** (deterministic) withdrawn: moved to SUA-1.
- **STU-10** (deterministic) The queue must show, after an issue, `tab` when a tab of the current zellij session is named after its stem.
- **STU-13** (deterministic) The worktrees panel must show, after the session of a worktree, `waiting since <HH:MM>: <reason>` (local time, its date `<YYYY-MM-DD>` before it when it is not today) when that session runs and the mark of SWT-3 of the worktree names it.
- **STU-16** (deterministic) `cardinal-ui` must show a fourth panel, `Measures`, from the last measure of STU-17: a line `measured <HH:MM>` (its date before it when it is not today, as STU-13), then, for each group of the efficiency table of `cardinal-report --project <project> --last 4 --json` (REP-17, REP-18, REP-21), its `group`, `sessions` and the medians `input_tokens` with `cache_read_share`, `output_tokens`, `main_context_peak` with `peak_sessions`, and `validations`, or `too few` after its `sessions`; then, for each group of the quality table of `cardinal-report --by milestone --json` (REP-20), its milestone, `issues`, `fixes_after_per_issue`, `findings_per_review` with `counted_reviews`, and `input_tokens_per_issue` with `token_issues`, each sample size written next to the number it bears on; `<project>` is the last component of `<main>` (STU-17), as EVA-7 names a project.
- **STU-17** (deterministic) A measure must run `cardinal run session-metrics` and `cardinal run delivered-quality <main> --summary`, `<main>` the repository its sessions resolve to (EVA-7: the directory that holds its git common directory when that is a `.git`, else the top level of the root), into the files `sessions.tsv` and `quality.tsv` of the cache directory, then the two `cardinal-report` of STU-16 with `--sessions` and `--quality` naming them, into `efficiency.json` and `milestones.json` beside them (each file written first under a temporary name, the reports reading the new `sessions.tsv` and `quality.tsv` under theirs); `cardinal-ui` starts one when it starts and the last measure is missing or more than one hour old, and on the key `m`, one at a time and apart from the keys and the reads of STU-2, the panel showing the last measure with `measuring` on its first line while one runs; `cardinal-ui --print` measures first when the last measure is missing or more than one hour old.
- **STU-20** (deterministic) withdrawn: moved to SUA-2.
- **STU-22** (deterministic) withdrawn: moved to SUA-3.
- **STU-26** (deterministic) withdrawn: moved to SUA-4.
- **STU-23** (deterministic) The worktrees panel must show `merged` after the branch of a linked worktree when the trunk holds `issues/done/<branch>.md`.
- **STU-4** (deterministic) `cardinal-ui --print` must print the screen of STU-1 once as text, whole whatever its number of lines, 100 columns wide or as wide as its longest line needs (in display columns), and exit, so a script or a test reads what the user would see.

### Contract

- **STU-5** (deterministic) The `cardinal` it runs must be the program named by the environment variable `CARDINAL_UI_CARDINAL`, else `cardinal` found on the `PATH`, run in the root of the work tree of the current directory (the current directory outside a git repository), as are the git commands.
- **STU-6** (deterministic) A tool of STU-3 that cannot be run, exits with a code other than 0 (other than 0 and 1 for `next-issue`), prints no JSON object, or one whose `version` is not `1`, must leave its panel showing one line naming the tool and the cause (the exit code and the first line of its stderr, `not JSON`, or `version <n> unsupported`), the other panels shown as usual; the exit code 1 of `next-issue` (a problem of the issue files, NXT-2) is no such error: the queue is shown with the lines of its stderr under it. A `release` in such an error leaves the milestones of `roadmap` shown, none of them marked `awaits release`, with its line under them, the panel counting as failed for STU-8.
- **STU-7** (deterministic) The worktrees must be those `git worktree list --porcelain` lists, in its order, each with its branch (`detached` when it has none); the session of a worktree is the `session` and `pid` of the file `cardinal-session` of its git directory (WSL-9), ended when no process of that `pid` runs, `free` when the file is missing or lacks the `session` or the `pid` (as `workspace-lock` reads it).
- **STU-8** (deterministic) `cardinal-ui --print` must exit 0 when every panel was read, 1 when a panel shows the error of STU-6, and `cardinal-ui` 2 on an unknown argument; `cardinal-ui` run outside a git repository shows the error of STU-6 in the worktrees panel.

- **STU-11** (deterministic) withdrawn: moved to SUA-5.
- **STU-14** (deterministic) The selection of SUA-1 must stay on the same issue when a read changes the order of the queue, move to the line it had when its issue left the queue, and stay visible however long the queue, so `s` never starts an issue the user did not see selected.
- **STU-15** (deterministic) withdrawn: moved to SUA-6.
- **STU-21** (deterministic) withdrawn: moved to SUA-7.
- **STU-24** (deterministic) withdrawn: moved to SUA-8.
- **STU-25** (deterministic) withdrawn: moved to SUA-9.
- **STU-12** (deterministic) withdrawn: moved to SUA-10.
- **STU-18** (deterministic) The cache directory must be `<cache>/cardinal-ui/<project>-<hash>`, `<cache>` the value of `XDG_CACHE_HOME` when it is an absolute path, else `$HOME/.cache`, and `<hash>` the 64-bit FNV-1a hash of the path of `<main>` in 16 hexadecimal digits, so two projects of one name keep their own; a measure writes each file whole (a temporary file of that directory, named after its process so two measures at once never share one, renamed over it, `efficiency.json` last), removes the temporary files of the directory over an hour old when it starts, and the age of the last measure is the modification time of `efficiency.json`.
- **STU-19** (deterministic) A step of a measure that cannot be run or exits with a code other than 0, or a report that prints no JSON object or one whose `version` is not `1`, must end the measure, leave the four files of the last measure unchanged (a measure renames its files into place only once every step succeeded), and leave the `Measures` panel showing the line of STU-6 naming the step (`session-metrics`, `delivered-quality`, `cardinal-report`) above the last measure, the panel counting as failed for STU-8.

- **STU-27** (deterministic) withdrawn: moved to SUA-11.
- **STU-28** (deterministic) withdrawn: moved to SUA-12.

## Scenarios

The happy paths STU-H2 and STU-H5 to STU-H8 moved to SUA-H1 to SUA-H5 of `specs/steering-ui-actions.md`.

- **STU-H1** (happy; STU-1, STU-4, STU-7) Given a project with a milestone of two issues, one of them claimed by a
  live session in a second worktree, when the user runs `cardinal-ui --print`, then they read the milestone with
  `2/2 open`, the queue with the free issue marked next and the other one taken in that worktree, and the two
  worktrees, the second held by the live session.
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
- (STU-13) Given a session waiting since the day before, when the UI reads the project, then the worktree reads
  `waiting since <YYYY-MM-DD> <HH:MM>`.
- (STU-14) Given `foo` selected on the second line, when the next read puts `foo` first, then `foo` is still selected.
- (STU-14) Given 40 issues in a queue panel of 15 rows, when the user moves the selection to the 30th, then it is
  shown.
- (STU-13) Given a mark naming a session whose process ended, when the UI reads the project, then the worktree reads
  `ended` with no `waiting`.
- (STU-13) Given a mark naming another session than the lock's, when the UI reads the project, then no `waiting` is
  shown.
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
- (STU-23) Given a linked worktree whose branch has no close on the trunk, then its row shows no `merged`.

## Non-goals

- Entering a session, ending a session whose issue is not merged, reusing a free worktree for a new issue, or editing
  an issue or a milestone: later issues of the milestone "Steering UI".
- Running as a zellij plugin, or starting a session in another terminal multiplexer.
- Telling a question from a validation stop: SWT marks every end of a turn.
- The other tables of `cardinal-report` (rules, skills, fixed context, verdicts), its reports by setup and the
  verdict of a trend: `skill:evaluate-harness` reads them; the panel never turns a number into a pass or a fail.
- A faster reading of the transcripts: the measure takes the time of the readers, the cache hides it.
- Following the changes of the files as they happen: the interval of STU-2 bounds how stale the screen is.
- Pruning what `next-issue` prunes on its own (CLM-6): `cardinal-ui` runs the tool, which may delete the claims of
  issues the trunk closed, as it does for the agent.
- The layout, the colours and the keys beyond those of STU-2, STU-17 and SUA: they may change without a requirement.
