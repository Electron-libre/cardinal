# Changelog

The notable changes of each version, newest first ([Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
[Semantic Versioning](https://semver.org/)). Each section is written by `cardinal run release <version>` from the commits,
then reread before the release is committed.

## [6.3.0] - 2026-11-25

### Milestones

- Specs and docs true to the code

### Added

- rebase a diverged issue branch with rebase-issue
- rebase a diverged branch at the local merge back
- rebase a diverged branch before its merge with f
- show the mark of Cardinal on the border of the queue
- end the waiting session before rebasing its branch
- select the issue next-issue offers with the key n
- start the audit of the code with a key
- file only the review findings with an observed case

### Fixed

- settle only real additions and recheck before the merge
- draw the mark of Cardinal on the first panel only
- name the ended session when the check after the merge fails
- drop the refusal of n once it selects the next issue
- give the happy path of the audit a free id
- end a task that panics as a failed one
- give the terminal back on a second panic of a task
- let rebase-issue pass a lock whose pid another process took
- end a measure or a notification that panics as a failure
- set the panic hook before the measure that starts the screen
- free a worktree lock whose pid a later process took
- read a worktree lock whose pid a later process took as ended
- kill a zellij call that has not answered within 5 seconds
- address a renamed zellij session by the name zellij records
- kill what a zellij call leaves behind before its reap
- release a claim whose pid a later process took
- count a claim by pid only while its pid was not taken
- read a claim time only for the claims a hand-over moves
- end by SIGPIPE when the reader of stdout closes
- raise SIGPIPE only for a closed stdout
- name the target in the refusal of worktree close
- refuse a broken configuration in close-issue
- refuse a wrong-typed release key and check --relink
- show the failures git no longer prints to the terminal
- drop the last copies and the panic of an early clock
- read the goal of an issue with the shared fence grammar
- read the title of an issue with the shared fence grammar
- read the headings of check-issue outside fenced blocks
- read the milestone of an issue outside its fenced blocks
- settle the review of the section fence grammar
- read a blank first heading of an issue as no title
- report a fenced block left open in an issue
- read a first `# Goal` heading as the sections do
- read the headings of an issue through one reader
- name a section by its heading with or without a note
- read the sections of delivered-quality by their name
- take only a note in parentheses after a section name
- end the tracking of delivered-quality at every heading

## [6.2.0] - 2026-11-23

### Milestones

- A steering UI to watch
- A steering UI to read at a glance

### Added

- end every merged worktree with the key E or --end-merged
- notify the desktop when a worktree starts to wait
- read the colors of the screen from a theme file
- draw the states of the panels in the roles of the theme
- keep twelve groups of revisions for the charts of the measures
- chart the measures on a screen of their own behind g
- show the issues of each milestone as a tree in the queue
- mark the issues whose session works or waits in the queue
- turn the mark of an issue at work at each tick
- show the status message on a row of its own
- move the measures off the main screen to a screen of their own
- offer only the keys that apply to the selection
- set the milestone headings apart in the queue
- wrap the status line on a narrow terminal
- give each issue its goal in the JSON of next-issue
- fold the summary of the selected issue under the queue
- offer the key of the summary in the help line
- start the review of the priorities with a key
- open a command palette on `:`
- shorten the help line to the keys of the selection
- let the selection rest on the milestone headings
- cut a milestone into issues with c
- audit the code of a repository with skill audit-code

### Fixed

- keep a merged worktree whose branch went on after the merge
- notify a wait once even when a read missed it
- keep the status line readable and name a missing named theme
- keep the newest rows and the status line readable on the charts
- draw a long chain of dependencies of the queue at once
- mark no issue while the worktrees cannot be read
- end a merged worktree whose directory is gone
- read the session of a worktree whose directory is gone
- highlight the selected line across its panel
- close only the gone worktree it targets
- keep the checks of a worktree close cannot read
- refuse a locked worktree before the end closes anything
- quote the path of the unlock hint of a locked worktree
- refuse a worktree holding another before the end closes it
- check the root before the worktrees inside the one to end
- keep the worktree lock across a fork of the session
- hand the lock only to the session a holder was last continued in
- hand the issue claims to the session a holder was continued in
- move only the claims of the holder in its worktree, before the lock
- give the worktree back to the session a user returns to
- give the lock back only to a return after the fork took it
- keep a failed notification past the task that ends after it
- hide the task keys of the help line while a task runs
- cut a wrapped status message between graphemes
- keep the ellipsis of the summary on a short screen
- complete the docs and the key test of the priorities review
- ignore modified letters and scroll the palette list
- start on the first issue after a read with no row
- keep audit-code useful without issue files or specs

## [6.1.0] - 2026-11-20

### Milestones

- Steering UI
- Merge back from the steering UI
- Releases from the steering UI

### Added

- print the queue of next-issue as JSON for a program
- print the milestones of roadmap as JSON for a program
- read the milestones, queue and worktrees of a project
- draw the panels and run the screen in the terminal
- mark the worktree of a session that waits for the user
- show the sessions that wait and the tabs of the issues
- start an issue session in a zellij tab
- give the milestone of each issue in delivered-quality
- print cardinal-report by milestone and as JSON
- show the measures of the project in the steering UI
- let a project merge its issue branches locally
- merge a merge-pending issue into the trunk with --merge
- merge the selected issue with the key f
- end what a merged issue left with --end
- end a merged issue after f and mark merged worktrees
- end the selected worktree of a merged issue with the key e
- print the release proposal as JSON for a program
- mark the milestones awaiting their release
- write the release steps in a skill of their own
- let a project that never releases turn its skill off
- start a release session on the trunk
- release a claim whose session ended
- say how to resume an issue whose session ended
- resume an issue whose session ended

### Fixed

- settle the review of the JSON of next-issue and roadmap
- name the package in each cargo install of a git source
- settle the review of the steering UI
- settle the review of the start of an issue session
- settle the review of the measures in the steering UI
- check that a local merge can fast-forward
- merge only a committed close, by the full branch name
- end the claimed worktree and never signal a recycled pid
- say when the release proposal is prepared or already tagged
- resume an interrupted release, fall back without its skill
- resume a prepared release and find its tab by its pane
- resume only a free worktree of the repository
- move the workflow actions off the node20 runtime

## [6.0.0] - 2026-11-20

### Breaking

- the format 11: the managed recipes move under `cardinal/` (`cardinal/cardinal.just` and the `cardinal/<part>.just`
  it imports); `sync` rewrites the `justfile` line `import 'cardinal.just'` and removes the five root `.just` files,
  so a project needs a `cardinal` that reads the format 11

### Added

- each session gets a scratch directory, `<tmp>/claude-scratch-<session id>/`, exported as `TMPDIR` to its `Bash`
  commands by the `SessionStart` hook `session-scratch start` and removed by `session-scratch end` unless it holds
  `cardinal-keep`; the start also removes the directories of dead sessions unchanged for a day; the rule
  `.claude/rules/scratch-files.md` asks for `$TMPDIR` over a literal `/tmp/<name>`; the feature `session-scratch`
  turns it off
- rule `scratch-outside-tmpdir` of `rule-breaches` counts the literal `/tmp/<name>` paths a `Bash` command writes to

## [5.2.0] - 2026-11-13

### Added

- the `SessionStart` hook `update-notice` tells the user when the source has a `v<X.Y.Z>` release the pinned `rev`
  lacks, with the `cardinal update --rev` to run (one query of the source a day, silent under CI, in a resumed session,
  a linked or dirty worktree and on any failure); the feature `update-notice` turns it off

## [5.1.0] - 2026-11-13

### Added

- `cardinal run file-size [<rev> | <a>..<b> | <a>...<b> | --all]` lists the files a change grows over
  `file_size_limit` (800 lines, 0 turns it off), or every file over it
- `issue-workflow` says in the plan where the code goes for a touched file over the limit and runs `file-size` at
  the review step; `prioritize-issues` proposes a restructuring issue for the largest files

## [5.0.0] - 2026-11-12

### Milestones

- Measures
- Specs that read as a product

### Breaking

- the format 10: a cited happy path needs a cardinal that reads it

### Added

- read the report within one project
- read a saved output in parts
- trace the happy paths that open the scenarios of a spec
- warn of a spec that states no behaviour
- warn of a spec that outgrows its capability
- measure the shape of the specs each done issue changed
- give the report the shape of the specs per spec
- count the happy path ids in delivered-quality

### Fixed

- name the refusals of read-hint
- count the review findings per review line
- tie a session to the issues it claims or closes
- give the commits of an issue to the session claiming it
- count of a later issue only the fix that starts it
- route a spec of an issue through the write-spec skill
- name the notes between tool calls in the language rule
- tie rule-breaches commits to the session of their claim

## [4.1.1] - 2026-11-05

### Fixed

- `check-specs` reads a requirement as CommonMark does: a statement wrapped onto several lines, or whose line ends in
  spaces, ends where its paragraph ends, and its wording is checked whole; fenced lines, HTML comments and an
  unclosed fence of the requirements section are handled.
- Specs and roadmaps read their code blocks, code spans and HTML comments from one CommonMark walk: a commented or
  fenced heading cuts no section.
- The memory imports skip every code block, code span and HTML as Claude Code does (list items and processing
  instructions read as `marked` reads them), and keep an escaped `@` apart.
- The language rule reads the prose of a reply as CommonMark.
- `close-issue` takes its link targets from the CommonMark parser: a reference definition in a quote or a list item
  is relinked; a link in HTML, an autolink or escaped brackets is left as written.

## [4.1.0] - 2026-11-04

### Added

- put an issue (`on_hold: '<reason>'`) or a milestone (`- On hold: <reason>`) on hold: `next-issue` never offers
  it and lists it as `On hold:` with its reason
- show the hold in roadmap and check it in check-issue

### Fixed

- settle the review of putting an issue or a milestone on hold

## [4.0.0] - 2026-11-03

### Breaking

- name cardinal run in the managed content: the format 9 needs a `cardinal` that reads it, and a project sets
  `check_command` in `[project]` for the check the skills run (`cardinal run project-check`); `just` is optional

### Added

- archive the released milestones on release
- print the state of the milestones with `roadmap`
- name the process and the paths that keep the worktree lock
- run the project check through cardinal
- count a failed project-check in delivered-quality
- waive the review through cardinal
- check the message of a revision with commit-check

### Fixed

- resolve the roadmap documents before archiving milestones
- report a broken configuration and escape the cells of `roadmap`
- tell what keeps the worktree lock and how to free it
- settle the review of the project check
- settle the review of the waiver tool
- settle the review of naming cardinal run

## [3.1.0] - 2026-11-03

### Milestones

- Lean agent loop

### Added

- count only the fixes of the lines an issue wrote
- tell the oversized results of the reviewers apart
- print only the last groups of the report
- print a review target without the issues and copies
- hand the reviews a target without the issues and copies
- fold the delegated exploration back into the main loop
- name the commit skill as the next step of an update
- send the agent back once before a whole read of a large file

### Fixed

- blame the fixes whatever the git configuration
- count only the project commits skipping the message skill
- read the commands of a commit call in their order
- run the setup report of evaluate-harness without --last
- review the uncommitted edits and untracked files too
- know the four new line types of the transcripts
- judge the update next step where the update writes
- estimate the result of a read as Claude Code returns it

## [3.0.0] - 2026-11-02

### Milestones

- Optional features
- Longer autonomy

### Breaking

- let a project turn off a feature of the harness
- ask the user before an agent writes a release

### Added

- make the ban on AI attribution a feature to turn off
- let a project turn off the lint, format and language hooks
- let a feature need another
- leave the commit format to a project with its own
- remove the native gate the source no longer manages
- let a project turn off the review gate
- keep the language rule out of the report style
- declare the features of the managed files only
- set the milestone done when its last issue closes
- let the close commit the milestone it sets done
- allow the release proposal in auto mode
- let a project turn off the issue files
- let a project turn off the specs
- let a project turn off the workspace lock
- release a project that turns off conventional-commits
- review the issues directory with the issue files off
- count the issues directory with the issue files off
- skip the validation stop rule with the issue files off
- sign the validations of an issue with the git identity
- upgrade the installed cardinal to the binary of a commit
- name cardinal upgrade in the install hint
- stage the close of an issue for a bare commit
- chain a queue of a few issues from one session

### Fixed

- hand over the worktree lock of a session with no work
- judge a changed lock holder instead of refusing it
- leave the features turned off out of update and the seeds
- remove the gate only once the lock records none
- quiet the review rules of rule-breaches with the gate off
- keep the warnings on the project's own retention
- read the roadmap of a close as the user writes it
- propose the release with cardinal run release
- stop citing issue-workflow in the review refusal
- gate the specs check on cardinal-specs.just
- find the project root of an issue checked in issues/done
- resolve the root of an issue like cardinal run does
- head the unsorted release section Changed
- tie the issues exceptions of the gates to the issue files
- sign a deliverable only once a person validated it
- upgrade the cardinal of the PATH, not the running one
- read each commit of a release with its own convention
- warn of the commits release reads without the convention
- read the roadmap milestones outside fences and comments
- close a roadmap fence and comment as Markdown does
- review the release commit and tag it with cardinal run
- refuse a done issue in check-issue with one problem
- resolve the directory of an issue once in check-issue
- refuse a dangling pre-commit.user at the gate install
- keep the hooks unchanged when the gate install fails
- never replace a hook at the gate install
- name the chained hook conflict in check
- share the chained hook conflict text of sync and check
- stage only the close of an issue, names read literally
- finish a gate install a killed sync left half done
- keep a queue of issues resumable and its diffs apart

## [2.0.0] - 2026-11-01

### Breaking

- stop reading the format 4 issues/queue.md in next-issue
- sync the format 5 and higher only
- order the issues by priority and dependencies

### Added

- review the existing instructions in audit-instructions
- count the changed lines of a deliverable with a tool
- count the staging of every file in rule-breaches
- refuse a git add of every file before a commit
- add a recipe that replays the fixture issue headless
- stop editing issues/queue.md in close-issue
- check the priority and dependencies of a new issue
- prioritize the issues instead of ranking them
- claim an issue for the session that works it
- free the claim of an issue when it is closed
- keep one Claude Code session per worktree
- claim the issue at the start of issue-workflow
- open a worktree with no issue attached
- close a worktree by its name, path or branch

### Fixed

- settle what the high review of the audit found
- let git count the untracked files of deliverable-size
- read a shell line once for the staging gate and rule
- grade a replay run on its end, commits, review and test
- keep the age of a renamed issue in next-issue
- free merged claims and keep claims to their worktree
- refuse the worktree closes that could drop the wrong work
- count a removed worktree with its repository by name
- prefer the branch match for a removed worktree

## [1.2.0] - 2026-10-31

### Milestones

- Advisor

### Added

- count the advisor calls and tokens of each session
- show the advisor columns in the efficiency table
- give a verdict on the expected advisor effects
- let the close of an issue commit without a review
- name when the issue workflow consults the advisor
- show the cache read share in the efficiency table
- sum the tokens spent on each closed issue
- report the tokens per closed issue
- leave out the issues whose tokens are unknown
- require enough issues for a per-issue verdict
- narrow the delegated exploration to wide searches
- require enough issues and reviews for a ratio verdict
- flag the skills a session should have loaded
- flag the tool results that waste tokens
- weigh the bytes of the waste rules per revision
- give the bytes per session of a waste rule a verdict
- keep the waste rules out of the hook candidates
- group the report by the setup of the sessions
- split the delivered quality by setup
- compare the model setups in the harness evaluation

### Fixed

- confound an advisor verdict when the advisor set changed
- refuse a close commit that holds more than rewrites
- measure the main context peak of an advised message
- take the iteration peak only on an advised message
- settle the advisor moments the high review found loose
- cover the verdict on the cache read share
- count a resumed message once in the tokens of an issue
- shorten the delegated search and tell it from a git log
- read the skill routing triggers more strictly
- accept the waste bytes metric for a waste rule only
- read a model advised by itself as a solo setup
- settle the setup comparison the high review found loose

## [1.1.0] - 2026-10-30

### Added

- end close-issue on the next issue to work on
- end the close report on a link to the next issue

## [1.0.0] - 2026-10-30

### Milestones

- Team use

### Breaking

- rank each issue in its own file

### Added

- read the user language from a personal file
- measure the user language from the personal file
- carry the personal configuration into issue worktrees
- order the issues by the rank of each issue file
- convert the issue queue into the ranks of the files

## [0.2.0] - 2026-10-30

### Milestones

- Plan and release

First numbered release.
