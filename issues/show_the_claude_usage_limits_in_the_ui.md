---
priority: medium
why: 'starting an issue near a usage limit leaves it half done; the limits are read only inside a Claude Code session'
---
# Show the Claude usage limits in the UI

# Goal
Before starting a session from the UI, the user cannot see how much of the 5-hour and weekly limits of their Claude
plan is used. The expected outcome is the used share and the reset time of each limit on the screen.

# Findings
* Nothing in `cardinal-ui` nor in the tools reads a usage limit today.
* The source of these numbers is not established yet.

# Approach
Measure first: where the numbers can be read without network in the tests and without a credential in the UI.
1. The JSON Claude Code hands to a status line command, if it carries the limits: a status line script of the
   harness writes them to the cache directory (STU-18), the UI reads that file and shows its age.
2. The `/usage` view of Claude Code, if it can be run headless.
3. An estimate from the session transcripts (tokens over the last 5 hours and 7 days), labelled as such.
Show `used <n>% · resets <HH:MM>` per limit on the status or measures line, `unknown` when no source answers.

# Specs
* `specs/steering-ui.md`: a new requirement; the spec of the status line script if 1 is chosen.

# Milestone
Cardinal UI

# Resources
* `crates/cardinal-ui/src/status.rs`, `measures.rs`; `specs/steering-ui.md` (STU-16 to STU-19).

# Hints
* Never read nor copy the OAuth credentials of Claude Code.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
Show the used share and the reset time of the 5-hour and weekly limits in `cardinal-ui`. Measured on Claude Code
2.1.332: the status line input carries `rate_limits.five_hour` and `rate_limits.seven_day` (`used_percentage`,
`resets_at`), so source 1 of the Approach is taken; `/usage` and the transcript estimate are out of scope.

## Plan
1. [x] `cardinal run status-line`: a tool of `crates/cardinal/src/tools/status_line.rs` that records the limits of its
   input in `<cache>/cardinal-ui/usage.json` and prints `5h <n>% · 7d <n>%`; README says how to name it as the
   `statusLine` of the user settings; verified by tests through `cardinal run status-line` with a fake `HOME`; makes
   true STL-1 to STL-4 (~400 lines). (autonomous)
2. [x] The limits on the worktrees border: a module `crates/cardinal-ui/src/limits.rs` (reads `usage.json`, words the
   line), drawn at the right end of the top border of the worktrees panel on the screen and in `--print`
   (`panels.rs` is at 798 lines: it only calls the module); verified by `--print` tests with a fake cache; makes true
   STU-37 (~400 lines). (autonomous)
3. [x] Docs and fixes of the `high` review of the issue: the status line runs the installed `cardinal tool
   status-line` (no project needed), a session never moves the usage file back (STL-5), resets as `<weekday> HH:MM`,
   the `read` time dropped on a narrow border and `--print` widened for the limits, a decimal `resets_at`, no empty
   corner; verified by tests of STL-1, STL-5 and STU-37; makes true STL-5. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 229fbcaec1eb63013c3b09405d83294960492456
- Install: the user names `cardinal run status-line` as the `statusLine` of `~/.claude/settings.json` (README); `sync`
  never manages it.
- Place: the right end of the top border of the worktrees panel, as the mark of SUS-6.
- No chaining of another status line command (non-goal of STL).

## Deliverable 1
- done: `cardinal run status-line` records the 5-hour and 7-day limits of the status line input in
  `<cache>/cardinal-ui/usage.json` and prints `5h <n>% · 7d <n>%`; `cardinal_state::cache` holds the cache directory
  rule both binaries use (the measures of the UI now read it there).

| check | result |
|---|---|
| tests | pass (`tests/status_line.rs`, 7; 1 internal) |
| lint | pass |
| specs | pass, `specs/status-line.md` accepted |
| docs | `docs/usage.md`, `README.md`, `docs/architecture.md`, `CLAUDE.md` |
| code review | low, no findings |

red: STL-H1, STL-1, STL-2, STL-3, STL-4: `cardinal: unknown tool `status-line``, exit 3

Code review: no findings

not verified: a real Claude Code session running `cardinal run status-line` (the input format was read from the
binary of Claude Code 2.1.332, not observed live).

decisions needed: none

## Deliverable 2
- done: the worktrees panel ends its top border with the usage limits read from `usage.json` at each read
  (`limits.rs`), drawn as the mark of SUS-6 through one shared `corner` drawing; `clock` moves to `clock.rs` so
  `panels.rs` stays under 800 lines.

| check | result |
|---|---|
| tests | pass (`tests/print_limits.rs`, 4; 2 internal in `limits.rs`) |
| lint | pass |
| specs | pass, `specs/steering-ui.md` accepted again |
| docs | `docs/usage.md`, `README.md`, `docs/architecture.md` (flow diagram) |
| code review | low, no findings |

red: STU-H10, STU-37: the top border of the worktrees panel did not end with the limits (`border(&out).ends_with`
failed in the 4 tests)

Code review: no findings

not verified: the colors of the limits on a real terminal (drawn in `muted`, checked only through `--print`).

decisions needed: none

## Deliverable 3
- done: the six findings of the `high` review fixed (spec STL-5 added, STL-1 and STU-37 amended, docs on
  `cardinal tool status-line`).

| check | result |
|---|---|
| tests | pass (`tests/status_line.rs` 11, `tests/print_limits.rs` 4, `limits.rs` 4 internal) |
| lint | pass |
| specs | pass, both accepted |
| docs | `docs/usage.md`, `README.md`, `docs/architecture.md` |
| code review | low, no findings |

red: STL-5: `assertion left == right failed: left: Number(40.0) right: 55.0` (the file moved back to the older
answer), and the test of the recorded `seven_day`; STU-37: none observed, its tests and code changed together

High review of the issue: 6 findings, all on the issue's own change, all fixed here:
- the documented `cardinal run status-line` needed a Cardinal project and its pinned revision: now `cardinal tool
  status-line`;
- the limits dropped off a border under ~106 columns when a date showed: weekday resets, `read` dropped first,
  `--print` widened;
- the happy path test failed between 23:00 and 24:00 UTC: fixed by the widened `--print`;
- an idle session could overwrite fresher limits: STL-5;
- a decimal `resets_at` read as `unknown` (nit): read as whole seconds;
- an empty limits string drew a gap in the border (nit): no corner then.

Code review: no findings

not verified: a live Claude Code session running the status line; the colors of the limits on a real terminal.

decisions needed: none
