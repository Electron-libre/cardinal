---
rank: 175
why: 'last of the new ones: the costliest; the observational verdicts come first and show whether it is needed'
---
# Replay a reference task set to compare two harness variants

# Goal
Every verdict of Cardinal is observational: it compares revisions over the real sessions that happened to run on
them, with different issues, projects, models and versions. A change of a skill or of the model can only be judged
after the fact, and a regression that shows once in two runs is easily missed. The outcome: a small set of fixed tasks
on a test repository, run headless several times against two variants of the harness (or two setups), reported with
the same metrics as `cardinal-report`.

# Findings
* `cardinal-report` groups sessions by revision and names the confounders (models, versions, project mix): REP-4,
  REP-5; it has no controlled comparison.
* The article on folding subagents recommends about ten reference tasks to start, graded on the final answer, each run
  several times: in its support-agent test a regression appeared in only one run of two.
* Not verified: the cost of a run of a full issue in headless mode (`claude -p`) on the user's plan.

# Approach
1. Measure first: one task (a small issue of a fixture repository) run headless twice, its duration, tokens and the
   effort to grade it.
2. Candidate: a fixture repository with about ten small issues, a recipe that syncs a given Cardinal revision into a
   copy, runs each issue N times and feeds the transcripts to `session-metrics` and `delivered-quality`.
3. Grading: the tests of the fixture pass, the review gate held, the rules of `rule-breaches` kept.
Likely over the size limit: split at `issue-workflow` step 2 (fixture, runner, report).

# Milestone
Lean agent loop

# Resources
* `specs/evaluation.md`, `specs/delivered-quality.md`, `specs/cardinal-report.md`.
* https://claude.com/fr/resources/articles/building-agents-when-to-fold-subagents-into-your-main-agent-loop

# Hints
* No network in the unit tests: the runner is a recipe, not a test of `cargo test`.
* The tasks must need no answer from the user, or the run stops at the stop of step 2: hand the plan pre-validated.

# Tracking

Plan: validated

## Understanding
This issue is cut to its first step: measure what one headless run of a small fixture issue costs (duration, tokens,
cost, grading effort) before building the ten-task set. Out of scope here: the ten tasks, the comparison of two
variants, the report; they become new issues only if the measure says the replay is affordable.

## Plan
1. [x] Fixture: `fixtures/replay/`, a tiny Rust crate (outside the workspace) with one small issue whose `# Tracking`
   holds a validated plan in autonomous mode, so the run never stops for the user; verified by `cargo test` failing on
   the fixture as handed over, then passing with the expected change applied by hand (not committed); no requirement.
2. [ ] Runner: a `replay <rev> <runs>` recipe of the `justfile` (this repository only, not `files/`) that copies the
   fixture to a fresh directory per run, commits it, runs `cardinal init` at `<rev>` of this repository, runs
   `claude -p` on the issue with `--output-format json`, then grades it (fixture tests, the review gate held: a commit
   exists and `issues/` records `Code review`) and prints one tab-separated line per run (duration, cost, tokens,
   turns, graded); verified by a dry run with `runs=1`; no requirement.
3. [ ] Measure: two runs at `HEAD`, their lines and the grading effort written in `# Findings`, with the go or no-go
   for the other cuts (ten tasks and grading; the comparison of two variants with the metrics of `cardinal-report`).

## Decisions
- Mode: autonomous
- No spec: a measure and a recipe of this repository, not a capability of `cardinal` (step 1 skips the spec for a measure).
- Base commit: 1d828de0912389a281eff9c5defef4e8f09e7002
- Headless runs use `--permission-mode auto`, `--max-budget-usd 5` (a capped run counts as failed) and the model
  of the user's configuration.
- The other cuts (ten tasks and grading; the comparison) are written as issues only after the measure, on a go.

## Deliverable 1: fixture
done: `fixtures/replay/` (crate, spec TXT, issue with a validated plan, held-out `grade/grade.rs`); `fixtures` added to
`skipped_dirs` so `check-specs --strict` does not read its citations.

| check | result |
|---|---|
| tests | `just check` passes |
| lint | clean |
| specs | `check-specs --strict` passes |
| docs | `CLAUDE.md` index, `update-docs-map.md` row |
| code review | see below |

red: none (a fixture; checked by hand: the grading test fails to compile on the fixture as handed over, `unresolved
import replay_fixture::word_frequencies`, and its 4 tests pass with a reference `word_frequencies` not committed)
Code review: no findings

not verified: none

decisions needed: none
