---
rank: 125
why: 'after the advisor count and the workflow guidance, and once enough sessions of each setup exist'
---
# Compare an advised executor with a solo model

# Goal
The advisor strategy claims that a cheaper executor with a stronger advisor reaches close to the quality of the strong
model alone at a fraction of its cost. For Cardinal's own work this is unproven. The outcome: `skill:evaluate-harness`
shows, for the same harness, the efficiency and the delivered quality of each setup (executor alone, executor with an
advisor, strong model alone), so the user can pick the default model of a project from numbers.

# Findings
* Anthropic's published guidance: run the same eval on Sonnet solo, Sonnet with an Opus advisor and Opus solo.
* The user's sessions mix Opus 5.5 (about 49,600 assistant messages) and Sonnet 5.5 (about 21,100), mostly with an
  Opus advisor configured (counted 2026-10-30).
* `cardinal-report` groups by Cardinal revision (REP-4) and only lists the models of a group (REP-5): it cannot split a
  group by setup.

# Approach
1. A setup per session: the main model plus the advisor model (from `count_the_advisor_calls_of_each_session.md`), or
   `solo`.
2. Candidate: a `--by setup` option of `cardinal-report` grouping by setup instead of revision, the other columns
   unchanged; or a setup column in the efficiency and quality tables.
3. `skill:evaluate-harness` presents it, with the sample size of each setup and its confounders (project mix, version).

# Specs
* `specs/cardinal-report.md`: new requirements for the setup grouping.

# Milestone
Advisor

# Resources
* `specs/cardinal-report.md`, `specs/delivered-quality.md`, `files/.claude/skills/evaluate-harness/SKILL.md`.
* https://claude.com/blog/the-advisor-strategy

# Hints
* A setup is a choice of the user, often per task difficulty: say that the comparison is observational, not an A/B.
* The issues of one setup may be harder than another's; the reference task set
  (`replay_a_reference_task_set_to_compare_two_harness_variants.md`) is the controlled version.

# Tracking

Plan: validated

## Understanding
`cardinal-report --by setup` groups the sessions by setup (main model, ` solo` or ` advised by <advisor>`) instead of
revision; `delivered-quality` gives each issue the setup of its sessions so the quality table splits too;
`skill:evaluate-harness` presents it as an observational comparison. Out of scope: rules, fixed context and verdicts by
setup (kept per revision), a controlled A/B (the reference task set issue).

## Plan
1. [x] Group by setup: `--by setup` in `cardinal-report`, the setup of a session, the `revisions` column, the
   revision-keyed tables left out; verified by report tests on fixture rows; makes true REP-14, REP-15, REP-16 (but
   its quality part).
2. [ ] Quality by setup: the `setup` column of `delivered-quality` per issue and per summary row, read by the quality
   table of both groupings; verified by delivered-quality tests on fixture transcripts and a report test; makes true
   QUA-16, QUA-17, the quality part of REP-16.
3. [ ] Present it: `skill:evaluate-harness` runs the report a second time with `--by setup` and reads it with its
   sample sizes, its confounders (revisions, projects, versions) and the observational caveat; verified by the skill
   checks of `just check`.

## Decisions
- Mode: autonomous
- Base commit: 70b6c7f267a403daf9e3c68943b9867a4d5fdb95
- Main model of a session: the first of `models` (sub-agent models would fragment the setups).
- Under `--by setup`, the rules, fixed context and verdicts are not printed; their files are still accepted.
- The revision mix of a setup group shows in a `revisions` column of the efficiency table.
- An issue made under several setups is `mixed`, a group of its own.

## Deliverable 1

- done: `cardinal-report --by setup` groups the sessions by setup with their revision mix; the quality, rules, fixed
  context and verdicts tables are left out by setup until deliverable 2 gives the quality summaries a setup.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (REP-14 to REP-16 traced, spec still `draft` until QUA-16, QUA-17) |
| docs | `docs/usage.md`, `docs/architecture.md` |

- red: REP-14, REP-15, REP-16: `unknown argument --by`
- Code review: 1 findings, none fixed: "`--by setup` returns before the tables print" is false, `table()` prints each table as it is built (the setup test reads them)
- not verified: none
- decisions needed: none
