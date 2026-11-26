---
id: STU
status: draft
adrs: [0011, 0022, 0023, 0030, 0031]
---

# Steering UI

## Intent

`cardinal-ui` shows the user, in one terminal screen kept up to date, where a project driven by Cardinal stands: its
milestones, its queue of issues, the worktree each session works in and the trend of its measures, so they need not
run several commands nor open each worktree. It changes the project only through the actions of SUA.

## Requirements

### Behaviour

- **STU-1** (deterministic) `cardinal-ui`, run in a project, must show in one screen two panels, no measure (SUC-6), the summary of SUQ-6 between them while it is unfolded: the queue (the milestones not yet released, each with its state, `on hold` when it is, `awaits release v<version>` when the milestones of `cardinal run release --json` (REL-15) hold one of its name and its document, `<version>` the `proposed` of that document, the bare `awaits release` when it is `null`, and its open issues over the listed ones, each followed by its open issues as SUQ-1 draws them, the one `next-issue` offers marked, with its priority, its state and what takes it: a claim with its worktree and whether its session ended, or a branch) and the worktrees (each with its branch and the session that holds its lock, ended or not, or `free`).
- **STU-2** (deterministic) `cardinal-ui` must read the project again every 2 seconds and on the key `r`, one read at a time and apart from the keys, so a slow read never keeps a key from being answered, and quit on `q`, `Esc` (on the panels, SUC-1, the command palette closed, SUP-4) or `Ctrl+C`, giving the terminal back as it found it.
- **STU-3** (deterministic) `cardinal-ui` must write no git ref of its own and no file outside the cache directory of STU-18: it reads the project only through `cardinal run next-issue --json`, `cardinal run roadmap --open --json`, `cardinal run release --json` when a milestone that `roadmap` lists has the state `done` (run again only once the commit of `HEAD`, a tag or the document of `roadmap` changed since its last run) and, in the main worktree, at each start of SUA-4 or SUA-16, the measure of STU-17, `cardinal run roadmap --open --json` at each start of SUA-19, git commands that read and, in a zellij session, `zellij action list-tabs --json` and, at an end or a start of SUA-4, SUA-16, SUA-19 or SUA-23, `zellij action list-panes --json --all`, and changes it only through the start of SUA-1, SUA-5, SUA-4, SUA-16, SUA-19 and SUA-23, the merge of SUM-1 and SUM-2 and the end of SUE-1 and SUE-3.
- **STU-9** (deterministic) withdrawn: moved to SUA-1.
- **STU-10** (deterministic) The queue must show, after an issue, `tab` when a tab of the current zellij session is named after its stem.
- **STU-13** (deterministic) The worktrees panel must show, after the session of a worktree, `waiting since <HH:MM>: <reason>` (local time, its date `<YYYY-MM-DD>` before it when it is not today) when that session runs and the mark of SWT-3 of the worktree names it.
- **STU-16** (deterministic) `cardinal-ui` must show the panel `Measures` on the screen of SUC-6 and under the panels of `--print`, from the last measure of STU-17: a line `measured <HH:MM>` (its date before it when it is not today, as STU-13), then, for each group of the efficiency table of `cardinal-report --project <project> --last 4 --json` (REP-17, REP-18, REP-21), its `group`, `sessions` and the medians `input_tokens` with `cache_read_share`, `output_tokens`, `main_context_peak` with `peak_sessions`, and `validations`, or `too few` after its `sessions`; then, for each group of the quality table of `cardinal-report --by milestone --json` (REP-20), its milestone, `issues`, `fixes_after_per_issue`, `findings_per_review` with `counted_reviews`, and `input_tokens_per_issue` with `token_issues`, each sample size written next to the number it bears on; `<project>` is the last component of `<main>` (STU-17), as EVA-7 names a project.
- **STU-17** (deterministic) A measure must run `cardinal run session-metrics` and `cardinal run delivered-quality <main> --summary --rows <issues.tsv>`, `<main>` the repository its sessions resolve to (EVA-7: the directory that holds its git common directory when that is a `.git`, else the top level of the root), into the files `sessions.tsv` and `quality.tsv` of the cache directory, its rows per issue (QUA-23) into `issues.tsv` beside them, then the two `cardinal-report` of STU-16 with `--sessions` and `--quality` naming them, into `efficiency.json` and `milestones.json` beside them, and the one of SUC-4 (each file written first under a temporary name, `issues.tsv` included, the reports reading the new `sessions.tsv` and `quality.tsv` under theirs); `cardinal-ui` starts one when it starts and the last measure is missing or more than one hour old, and on the key `m` (SUC-1), one at a time and apart from the keys and the reads of STU-2, the panel showing the last measure with `measuring` on its first line while one runs; `cardinal-ui --print` measures first when the last measure is missing or more than one hour old.
- **STU-20** (deterministic) withdrawn: moved to SUM-1.
- **STU-22** (deterministic) withdrawn: moved to SUE-1.
- **STU-26** (deterministic) withdrawn: moved to SUA-4.
- **STU-29** (deterministic) `cardinal-ui`, run as a screen, must send one desktop notification when a read finds a worktree waiting (STU-13) with a mark (its session and its time) no read before had found, titled `<branch> waits` (the path of the worktree when it has no branch) and whose body is the reason of the mark, and none for the waits its first read finds.
- **STU-23** (deterministic) The worktrees panel must show `merged` after the branch of a linked worktree when the trunk holds `issues/done/<branch>.md`.
- **STU-31** (deterministic) withdrawn: moved to SUS-1.
- **STU-36** (deterministic) The key `n` on the panels must move the selection of STU-14 to the row of the issue `next-issue` offers (the `next` of NXT-19), the focus moving to the queue when the worktrees have it (SUE-1), so `s` starts it next; when it offers none (`next` is `null`, or names an issue no row of the queue holds, or the queue could not be read), the selection and the focus stay as they are and the status line (SUS-2) says `n: no issue is ready to start`, until a later `n` that moves the selection drops that message.
- **STU-34** (deterministic) withdrawn: moved to SUS-2.
- **STU-35** (deterministic) withdrawn: moved to SUS-3.
- **STU-4** (deterministic) `cardinal-ui --print` must print the screen of STU-1 then the panel `Measures` of STU-16 once as text, whole whatever its number of lines, 100 columns wide or as wide as its longest line needs (in display columns), and exit, so a script or a test reads what the user would see.

- **STU-37** (deterministic) The screen of STU-1 and `--print` must show the usage limits of the Claude plan at the right end of the top border of the worktrees panel, as SUS-6 places the mark: `5h used <n>% · resets <time> · 7d used <n>% · resets <time> · read <HH:MM>`, from the usage file of STL-3 read at each read of STU-2, `<n>` the `used_percentage` rounded to the nearest integer, each `<time>` its `resets_at` in local time, `HH:MM` when it is today, else `<weekday> HH:MM` (`Mon 09:00`), and `read` its `written_at` in local time with its date before it when it is not today (as STU-13); a limit the file lacks, or whose `resets_at` has passed, reads `<5h|7d> unknown`, and a usage file that is missing, cannot be read, is not JSON or whose `version` is not `1` reads `limits unknown`; a border too narrow to hold its title, the limits and one cell of border between them shows them without their ` · read <HH:MM>`, and its title alone when that does not fit either; `--print` is as wide as its border needs.

### Contract

- **STU-5** (deterministic) The `cardinal` it runs must be the program named by the environment variable `CARDINAL_UI_CARDINAL`, else `cardinal` found on the `PATH`, run in the root of the work tree of the current directory (the current directory outside a git repository), as are the git commands.
- **STU-6** (deterministic) A tool of STU-3 that cannot be run, exits with a code other than 0 (other than 0 and 1 for `next-issue`), prints no JSON object, or one whose `version` is not `1`, must leave its panel showing one line naming the tool and the cause (the exit code and the first line of its stderr, `not JSON`, or `version <n> unsupported`), the other panels shown as usual; the exit code 1 of `next-issue` (a problem of the issue files, NXT-2) is no such error: the queue is shown with the lines of its stderr under it. A `release` in such an error leaves the milestones of `roadmap` shown, none of them marked `awaits release`, with its line above them (SUQ-1), the panel counting as failed for STU-8.
- **STU-7** (deterministic) The worktrees must be those `git worktree list --porcelain` lists, in its order, each with its branch (`detached` when it has none); the session of a worktree is the `session` and `pid` of the file `cardinal-session` of its git directory (WSL-9), ended when the lock is stale as `workspace-lock` reads it (WSL-1: no process of that `pid` runs, or it started more than 2 seconds after the file was last written), `free` when the file is missing or lacks the `session` or the `pid` (as `workspace-lock` reads it).
- **STU-8** (deterministic) `cardinal-ui --print` must exit 0 when every panel was read, 1 when a panel shows the error of STU-6, and `cardinal-ui` 2 on an unknown argument; `cardinal-ui` run outside a git repository shows the error of STU-6 in the worktrees panel.

- **STU-11** (deterministic) withdrawn: moved to SUA-5.
- **STU-14** (deterministic) The selection of SUA-1 must move over the issue rows of the queue and the headings of the milestones `roadmap --open` lists, in the order of their lines, never on another heading (a milestone `roadmap` does not list, `No milestone`) nor an empty or error line, start on the row of the issue `next-issue` offers (STU-36), else on the first issue row (else the first such heading), stay on the same issue or milestone when a read changes the order or the tree of the queue, move to the row of the rank it had when its issue or milestone left the queue, and stay visible however long the queue, so a key never acts on a row the user did not see selected; on a heading, `s` and `f` start nothing and say `s and f act on an issue: select one` on the status line.
- **STU-15** (deterministic) withdrawn: moved to SUA-6.
- **STU-21** (deterministic) withdrawn: moved to SUM-2.
- **STU-24** (deterministic) withdrawn: moved to SUE-3.
- **STU-25** (deterministic) withdrawn: moved to SUE-4.
- **STU-12** (deterministic) withdrawn: moved to SUA-10.
- **STU-30** (deterministic) The notification of STU-29 must run the program named by the environment variable `CARDINAL_UI_NOTIFY`, else `notify-send` found on the `PATH`, with the title and the body as its two arguments, apart from the keys and the reads of STU-2; a program that cannot be run or exits with a code other than 0 leaves the screen as it was but for its status line, which shows, after what it already shows, `notify: ` and the cause (as STU-6 words it) the first time only.
- **STU-18** (deterministic) The cache directory must be `<cache>/cardinal-ui/<project>-<hash>`, `<cache>` the value of `XDG_CACHE_HOME` when it is an absolute path, else `$HOME/.cache`, and `<hash>` the 64-bit FNV-1a hash of the path of `<main>` in 16 hexadecimal digits, so two projects of one name keep their own; a measure writes each file whole (a temporary file of that directory, named after its process so two measures at once never share one, renamed over it, `efficiency.json` last), removes the temporary files of the directory over an hour old when it starts, and the age of the last measure is the modification time of `efficiency.json`.
- **STU-19** (deterministic) A step of a measure that cannot be run or exits with a code other than 0, or a report that prints no JSON object or one whose `version` is not `1`, must end the measure, leave the files of the last measure unchanged (a measure renames its files into place only once every step succeeded), and leave the `Measures` panel showing the line of STU-6 naming the step (`session-metrics`, `delivered-quality`, `cardinal-report`) above the last measure, the panel counting as failed for STU-8.

- **STU-32** (deterministic) withdrawn: moved to SUS-4.
- **STU-33** (deterministic) withdrawn: moved to SUS-5.

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
  done issues belong to a milestone, when the user runs `cardinal-ui` with no measure cached and opens the screen of SUC-6,
  then the `Measures` panel reads `measuring`, and once the measure ends it shows the two groups with their sessions and medians and the
  milestone with its issues and ratios; run again within the hour, it shows that measure at once without measuring.
- **STU-H9** (happy; STU-29, STU-30) Given `cardinal-ui` running and a session in the worktree of branch `foo`, when
  its turn ends, then within one read the desktop shows `foo waits` with the body `stop`, once, however many reads
  follow while it waits.
- **STU-H10** (happy; STU-37) Given a session that answered at 12:40 with the 5-hour limit used at 42% until
  14:30 and the weekly one at 13% until Monday 09:00, when the user looks at `cardinal-ui`, then the worktrees panel
  reads `5h used 42% · resets 14:30 · 7d used 13% · resets Mon 09:00 · read 12:40`.
- (STU-1) Given an issue claimed by a session whose process ended, when `cardinal-ui --print` runs, then the queue shows its claim as ended.
- (STU-1) Given a milestone with `- On hold: later`, when `cardinal-ui --print` runs, then its row says `on hold`.
- (STU-6) Given a `cardinal` that exits 3 with `cannot run the tool` on stderr for `roadmap`, when `cardinal-ui --print` runs, then the queue panel reads `roadmap: exit 3: cannot run the tool` above the issues, grouped by the milestones `next-issue` names, the worktrees are shown, and it exits 1.
- (STU-6) Given a `cardinal` whose `roadmap --json` prints `{"version":2,"milestones":[]}`, when `cardinal-ui --print` runs, then the queue panel reads `roadmap: version 2 unsupported`.
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
  when `cardinal-ui --print` runs, then the queue panel shows the milestone without `awaits release` and the
  line `release: exit 3: cannot read the tags` above it, and it exits 1.
- (STU-6) Given a `release --json` that prints `{"version":2}`, then the queue panel holds the line `release:
  version 2 unsupported` above its milestones.
- (STU-7) Given a worktree with no `cardinal-session` file, when `cardinal-ui --print` runs, then its row reads `free`.
- (STU-8) Given a directory outside any git repository, when `cardinal-ui --print` runs, then the worktrees panel shows an error and it exits 1.
- (STU-4) Given 700 open issues, when `cardinal-ui --print` runs, then the first and the last issues are both printed.
- (STU-5) Given the user in a subdirectory of the project, when `cardinal-ui --print` runs, then the tools run at the root and the panels are those of the project.
- (STU-7) Given a lock file that holds a `session` and no `pid`, when `cardinal-ui --print` runs, then the worktree reads `free`.
- (STU-7) Given a lock whose pid a process started after the last write of the lock took, when `cardinal-ui --print`
  runs, then the session of the worktree reads ended, and `--merge` and `--start` count it ended.
- (STU-8) Given `cardinal-ui --print | head -1`, then it exits 0 or 1, never on a panic.
- (STU-13) Given a session waiting since the day before, when the UI reads the project, then the worktree reads
  `waiting since <YYYY-MM-DD> <HH:MM>`.
- (STU-14) Given the queue `a` then `b`, `a` taken and `b` offered by `next-issue`, when the screen opens, then `b` is
  selected.
- (STU-36) Given `a` selected and `c` the issue `next-issue` offers, when the user presses `n`, then `c` is selected
  and `s` would start it.
- (STU-36) Given the worktrees panel focused, when the user presses `n`, then the queue has the focus and the issue
  `next-issue` offers is selected.
- (STU-36) Given every open issue taken, when the user presses `n`, then the selection stays where it was and the
  status line reads `n: no issue is ready to start`.
- (STU-36) Given that line shown, when a read offers `b` and the user presses `n`, then `b` is selected and the line
  is gone.
- (STU-14) Given `foo` selected on the second line, when the next read puts `foo` first, then `foo` is still selected.
- (STU-14) Given the last issue of the milestone `One` selected and the milestone `Two` listed after it, when the
  user moves down, then the heading `■ Two` is selected; given the section `No milestone` after it instead, then its
  first issue is selected.
- (STU-14) Given the milestone `Two` listed with no open issue, then the selection rests on its heading; given the
  heading `■ Two` selected, when the user presses `s`, then the status line reads `s and f act on an issue: select
  one`.
- (STU-14) Given the heading `■ Two` selected, when the next read puts another milestone before it, then `■ Two` is
  still selected.
- (STU-14) Given `b` drawn under `a`, when `a` is closed and `b` moves to the root, then `b` is still selected.
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
- (STU-29) Given a worktree already waiting when `cardinal-ui` starts, then no notification is sent for it.
- (STU-29) Given a waiting session whose idle notification changes the reason of its mark, then no second
  notification is sent.
- (STU-29) Given a session that waited, worked again, then waits anew, then a second notification is sent.
- (STU-29) Given a waiting worktree that one read could not read, when the next read finds the same mark, then no
  second notification is sent.
- (STU-29) Given a mark naming another session than the lock's, then no notification is sent.
- (STU-37) Given no usage file, then the worktrees panel reads `limits unknown`.
- (STU-37) Given a usage file whose 5-hour limit reset an hour ago, then it reads `5h unknown` before the weekly one.
- (STU-37) Given a worktrees border 80 columns wide, then the limits are shown without their `read` time.
- (STU-37) Given a `resets_at` written as a decimal number, then its limit is shown like an integer one.
- (STU-30) Given a `CARDINAL_UI_NOTIFY` that exits 3 with `no bus` on stderr, when two waits are notified, then the
  status line reads `notify: exit 3: no bus` after the first only.

## Non-goals

- Entering a session, ending a session whose issue is not merged, reusing a free worktree for a new issue, or editing
  an issue or a milestone: later issues of the milestone "Steering UI".
- Running as a zellij plugin, or starting a session in another terminal multiplexer.
- Notifying without the screen running (`--print`, a hook), grouping several waits into one notification, a sound,
  or a notification when a wait ends.
- Telling a question from a validation stop: SWT marks every end of a turn.
- The other tables of `cardinal-report` (rules, skills, fixed context, verdicts), its reports by setup and the
  verdict of a trend: `skill:evaluate-harness` reads them; the panel never turns a number into a pass or a fail.
- A faster reading of the transcripts: the measure takes the time of the readers, the cache hides it.
- Following the changes of the files as they happen: the interval of STU-2 bounds how stale the screen is.
- Pruning what `next-issue` prunes on its own (CLM-6): `cardinal-ui` runs the tool, which may delete the claims of
  issues the trunk closed, as it does for the agent.
- The layout and the keys beyond those of STU-2, STU-17 and SUA: they may change without a requirement; the drawing
  of the screen (its theme, its status line) is SUS.
