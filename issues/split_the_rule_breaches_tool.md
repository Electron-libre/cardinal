---
priority: low
depends_on: [read_the_project_table_through_one_reader]
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the rule breaches tool

# Goal
`crates/cardinal/src/tools/rule_breaches.rs` holds 1029 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
responsibilities, so each change grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of behaviour.

# Findings
* Code audit of 2026-11-24 (`cardinal run file-size --all`).
* `crates/cardinal/tests/rule_breaches.rs` holds 1467 lines (`cardinal run file-size --all`, 2026-11-25), over the
  limit too.
* The transcript model (86-312), the verdicts of each rule (414-692) and the checks of the commit convention
  (`managed_paths`, `conventional`, `commit_scopes`, `Record`, `check_commits`, `record` at 693-927).

# Approach
1. Move 693-927 to `rule_breaches/commits.rs`.
2. Split `crates/cardinal/tests/rule_breaches.rs`: the tests of the commit convention from those of the transcript
   rules, as the source, into a test file of its own beside it (as `delivered_quality_headings.rs`), its helpers
   shared through `tests/common`; a deliverable of its own, cut in two if the move counts over the limit.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/tests/rule_breaches.rs`.
* `crates/cardinal/src/tools/rule_breaches.rs`, `docs/architecture.md` (the module list it may name).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
Split `rule_breaches.rs` (1023 lines) and its test file (1467 lines) by responsibility, every part under 800 lines, no
change of behaviour: moves only. Differs from the issue: `Record`, `record` and `managed_paths` assemble a session, not
the commit convention, so they stay in the root; the transcript model moves too, else the root stays over 800; the
test file needs the language and routing checks moved besides the commit ones to fall under the limit.

## Plan
1. [x] Source modules: move `attributed`, `OLD_SCOPE`, `conventional`, `commit_scopes`, `check_commits` to
   `rule_breaches/commits.rs` and `EDIT_TOOLS`, `Call`, `Event`, `Transcript`, `typed_skills`, `skill_name` to
   `rule_breaches/transcript.rs` (~320 moved, ~660 counted, root left at ~710); verified by `project-check` unchanged
   green and `file-size`. (autonomous)
2. [x] Test helpers: move `COLUMNS`, `SUMMARY`, `sh`, `commit_at`, `new_repo`, `Transcript`, `bash`, `edit`, `run`,
   `row`, `sorted`, `tool_command` of `tests/rule_breaches.rs` to `tests/rule_breaches_common/mod.rs` (as
   `delivered_quality_common`; ~170 moved, ~360 counted); verified by the tests unchanged green. (autonomous)
3. [x] Commit tests: move `git_at`, `commit_repo`, `commit_checks`, `claim_checks`, `attribution_off_checks`,
   `convention_off_checks` (BRE-9, FEA-11, FEA-22) to `tests/rule_breaches_commits.rs` with a `#[test]` of their own
   (~270 moved, ~560 counted); verified by the same checks passing there. (autonomous)
4. [ ] Language tests: move `FRENCH`, `ENGLISH`, `AN_AS`, `HINT`, `language_checks`, `personal_language_checks`,
   `language_kind_checks` (BRE-10, BRE-13, BRE-14) to `tests/rule_breaches_language.rs` (~205 moved, ~430 counted).
5. [ ] Routing tests: move `ROUTED_LOCK`, `routing_other_sessions`, `skipped_commits`, `routing_checks` (BRE-15 to
   BRE-18) to `tests/rule_breaches_routing.rs` (~165 moved, ~350 counted); `tests/rule_breaches.rs` left at ~680
   lines; verified by the tests and `file-size --all` listing no `rule_breaches` file.
6. [ ] Docs and high review: the module docs list the modules and the test files; the fixes of the `high` review;
   verified by `project-check`.

## Decisions
- Mode: autonomous
- Base commit: f1735e12e22560857c4282fda31d9a6d8c271534

## Deliverable 1
- done: the commit message checks moved to `rule_breaches/commits.rs`, the transcript model (calls, events, typed
  skills) to `rule_breaches/transcript.rs`; the root keeps the rules, the session record and `main` (720 lines).

| check | result |
|---|---|
| tests, lint, specs (`project-check`) | pass |
| docs | no change: no document names a module of the tool |
| deliverable-size | 649 changed lines, within |
| file-size | no `rule_breaches` source file over 800 |
| code review | low |

- red: none (a move, no new test)
- Code review: no findings
- not verified: nothing
- decisions needed: none

## Deliverable 2
- done: the fixture repositories, the transcript under construction, the runs of the tool and its rows moved from
  `tests/rule_breaches.rs` to `tests/rule_breaches_common/mod.rs`, shared by the test files to come.

| check | result |
|---|---|
| tests, lint, specs (`project-check`) | pass |
| docs | no change |
| deliverable-size | 342 changed lines, within |
| code review | low |

- red: none (a move, no new test)
- Code review: no findings
- not verified: nothing
- decisions needed: none

## Deliverable 3
- done: the commit checks (BRE-9, FEA-11, FEA-22) moved to `tests/rule_breaches_commits.rs` with a `#[test]` of their
  own, in the same order; the doc comments of `session_a` and `tool_command` that deliverable 2 dropped restored.

| check | result |
|---|---|
| tests, lint, specs (`project-check`) | pass, the 6 commit checks green in their own binary |
| docs | the module docs of both test files |
| deliverable-size | 576 changed lines, within |
| code review | low |

- red: none (a move, no new test)
- Code review: no findings (the `low` level skips test hunks: the move checked line by line against the base instead)
- not verified: nothing
- decisions needed: none
