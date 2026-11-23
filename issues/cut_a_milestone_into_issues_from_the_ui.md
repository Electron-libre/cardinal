---
priority: medium
depends_on: [rethink_the_commands_of_the_ui]
why: 'turning a shaped milestone into issues is a steering step done from another terminal today; one more command for the new command system'
---
# Cut a milestone into issues from the UI

# Goal
A milestone of the roadmap becomes issues through `skill:create-issue` ("Cut a shaped milestone into issues"),
started by hand in a `claude` session. The expected outcome is a command of `cardinal-ui` that, on the selected
milestone, starts that session in a zellij tab of the main worktree.

# Findings
* `skill:create-issue` has the procedure "Cut a shaped milestone into issues": for a `shaped` milestone, or an
  `idea` one whose goal is agreed.
* The selection moves over the issue rows only, never on a heading (STU-14): a milestone cannot be selected today.
* `v` already starts a skill session in a zellij tab of the main worktree (SUA-4, SUA-11, SUA-12).

# Approach
1. Let the selection rest on a milestone heading (amending STU-14), or pick the milestone in the menu or palette of
   `rethink_the_commands_of_the_ui`.
2. The command starts `claude "/create-issue cut the milestone <name> into issues"` in a new zellij tab named after
   the milestone, as SUA-4 does; refused (status line, SUS-2) outside zellij, on a milestone that is neither `shaped`
   nor `idea`, or while that tab is open.
3. The help line or menu offers it only on such a milestone (SUA-15).

# Specs
* `specs/steering-ui.md`: STU-14 (selection), STU-3 (what the UI starts); `specs/steering-ui-actions.md`: a new
  requirement (the command, its refusals), SUA-15.

# Milestone
A steering UI to command

# Resources
* `.claude/skills/create-issue/SKILL.md`, `crates/cardinal-ui/src/release.rs`, `crates/cardinal-ui/src/keys.rs`,
  `ROADMAP.md` (milestone states).

# Hints
* An `idea` milestone needs its goal agreed by the user: the session asks, the UI does not decide.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
The selection of the queue also rests on the heading of a milestone `roadmap --open` lists; on a `shaped` or `idea`
one, `c` (and `cardinal-ui --cut <milestone>`) opens a zellij tab named after it in the main worktree running
`claude "/create-issue cut the milestone <name> into issues"`. Out of scope: writing the issues or agreeing an `idea`
goal in the UI (the session does it).

## Plan
1. [x] Selection on the milestone headings: the read keeps the selectable rows of the queue (issues and headings of
   the listed milestones) with their lines, the selection moves over them and follows its issue or milestone across
   reads, `s` and `f` on a heading say `s and f act on an issue: select one`, the summary and `i` act on an issue
   only; new code in `keys.rs` and `panels.rs`, `screen.rs` (over the file size limit) only takes the new field of
   `View`; verified by the unit tests of the selection, the help line and the summary, `--print` unchanged; makes
   true STU-14, SUQ-6 (~350 lines). (autonomous)
2. [x] The cut command: module `cut.rs` (the checks of SUA-20, the tab of SUA-21 through `skill_tab`), the flag
   `--cut <milestone>` in `main.rs`, the key `c` as a task, `c cut` on the help line and in the palette; verified by
   the binary tests `tests/cut.rs` (fake `zellij` and `cardinal`, as `tests/prioritize.rs`) and the unit tests of the
   keys, the help line and the palette; makes true SUA-19, SUA-20, SUA-21, SUA-15, SUA-6, SUP-1, STU-3 (~500 lines). (autonomous)
3. [x] The fixes of the `high` review: after a read with no row the selection starts on the first issue, and the
   doc of `screen.rs` names `c`; verified by the unit test of the selection; makes true STU-14. (autonomous)

## Decisions
- Mode: autonomous
- Answers: 1 a (the selection rests on the milestone headings), 2 a (only the milestones `roadmap --open` lists)
- Means: the selection rests on the milestone headings (approach 1 of the issue), so a milestone with no open issue
  yet, the usual `shaped` one, can be cut; the palette lists `c cut` as it lists the other keys of the selection.
- Refusals: also those of a dirty, detached or locked main worktree and a skill `create-issue` turned off, as
  SUA-11 and SUA-17 refuse them.
- Base commit: ff47356d3b59fb3558fba9890eb9b7c43fecbcf0

## Deliverable 1: selection on the milestone headings

- done: the read keeps the rows of the queue (`keys::Row`: an issue, or the heading of a milestone `roadmap` lists)
  with their lines; the selection moves over them, follows its row across the reads and starts on the first issue;
  `s` and `f` on a heading say `s and f act on an issue: select one`; no summary nor `i summary` on a heading.
- checks: tests pass, lint pass, specs pass, docs `docs/usage.md` and the doc of `screen.rs`, code review low pass.
- red: STU-14: `an_empty_queue_says_so_under_its_milestones` left `[]`, right `[(Milestone("New"), 0), (Milestone("Old"), 2)]` (the push of the heading rows taken out).
- Code review: no findings
- not verified: the screen in a real terminal (the tests draw it on a test backend).
- `screen.rs` (over the file size limit) grows by one line, the first read starting the selection: not split for it.

## Deliverable 2: the cut command

- done: `cut.rs` checks then opens the tab of the milestone in the main worktree (`skill_tab`); `c` on a heading runs
  it as a task, `--cut <milestone>` from the command line; `c cut` on the help line and in the palette on a `shaped`
  or `idea` milestone in zellij; the specs accepted.
- checks: tests pass, lint pass, specs pass (all four accepted), docs `docs/usage.md`, `docs/architecture.md`,
  `README.md`, code review low pass.
- red: SUA-19, SUA-20, SUA-21, STU-3: the 9 tests of `tests/cut.rs` failed with exit `Some(2)` (the usage line: `--cut` unknown).
- Code review: no findings
- not verified: a real zellij and `claude` session (the tests use fakes).

## Deliverable 3: the fixes of the high review

- done: a read that follows one with no row starts the selection on its first issue; the doc of `screen.rs` names `c`.
- checks: tests pass, lint pass, specs pass, docs the doc of `screen.rs`, code review low pass.
- red: STU-14: `the_selection_moves_over_the_issues_and_the_milestones`: `not on the heading of One`, left `Some((0, 0))`, right `Some((0, 1))`.
- Code review: no findings
- High review: 5 findings; fixed: the selection after an empty first read, the doc of `screen.rs`. Not fixed:
  - the match of a running tab on `claude <message>` with spaces: `end.rs` already matches the issue tab, whose
    argument holds a space too, on the same unquoted form;
  - two roadmap documents with a milestone of one name: the queue already keys its sections by name, out of the scope
    of this issue;
  - a milestone named like an issue stem, or `release` or `prioritize`, shares its tab name: SUA-21 names the tab
    after the milestone as validated; milestones are titles, stems snake case.
- not verified: none beyond those of deliverables 1 and 2.
