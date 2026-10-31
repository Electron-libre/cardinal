---
rank: 66
why: 'after the cache share, before the exploration verdict: gives it the cost of a finished issue rather than of a session'
---
# Report the tokens spent per closed issue

# Goal
The efficiency table gives medians per session (REP-5). An issue often takes several sessions, and a session can stop
before anything lands, so a change that makes sessions shorter but more numerous looks cheaper while each delivered
issue costs more. The metric that matters is the cost of a task done: the tokens spent on an issue from its plan to
its close. The expected outcome: the report gives the tokens per closed issue next to the delivered quality.

# Findings
* `delivered-quality` already ties each closed issue of `issues/done/` to its sessions and to the events of those
  sessions that count for it (QUA-7, QUA-8), and gives per issue `sessions`, `failed_checks`, `validations`,
  `other_messages` (QUA-1); it reads no token.
* The quality table sums these per group and divides some by `issues` (REP-6).

# Approach
1. Add token columns to `delivered-quality` (input, cache read, output), summed over the messages of the issue's
   sessions that count for the issue (QUA-8), and a ratio per issue in the quality table.
2. Or join `session-metrics` rows to the issues by branch (EVA-6) in `cardinal-report`: simpler, but loses the
   sessions on the default branch and counts a whole session even when part of it worked on another issue.
Then the metric for `Expected-effect:` (VRD-1).

# Specs
* `specs/delivered-quality.md`: new columns of QUA-1 and their rule.
* `specs/cardinal-report.md`: REP-6 ratios; `specs/cardinal-verdicts.md`: VRD-1.

# Milestone
Lean agent loop

# Resources
* `specs/delivered-quality.md` (QUA-1, QUA-7, QUA-8), `specs/cardinal-report.md` (REP-6).
* `crates/cardinal/src/tools/delivered_quality.rs`, `transcripts.rs`.
* `issues/done/measure_the_main_context_peak_of_an_advised_message.md`: the usage of an advised message counts its
  iterations; the same trap applies to a token sum.

# Hints
* An abandoned issue never reaches `issues/done/`: its tokens are lost to this metric; say so in a non-goal.

# Tracking

Plan: validated

## Understanding
Sum the tokens of the messages that count for each closed issue (approach 1: the sessions and window of QUA-7 and
QUA-8, sub-agents included, top-level usage once per `message.id`), give their mean per issue in the quality table,
and make them `Expected-effect:` metrics. Out of scope: abandoned issues, sessions with no commit of the issue, advisor tokens.

## Plan
1. [x] Token columns of `delivered-quality`: `input_tokens`, `cache_read_tokens`, `output_tokens` per issue and in
   `--summary`, and the `QUALITY` header of `cardinal-report` that reads it; verified by a fixture session with a
   split message, a sub-agent and a message past the last commit; makes true QUA-1, QUA-7, QUA-15 (~250 lines).
2. [ ] Ratios and verdicts: `input_tokens_per_issue`, `output_tokens_per_issue` in the quality table and accepted
   by `commit-check` and the `## Verdicts` table; verified by the report and commit-check tests; makes true REP-6,
   VRD-1 (~120 lines).

## Decisions
- Mode: autonomous
- Base commit: 37098daa408e7593fe0b8864f51ea812138e86c8
- Quality table sums keep the names of the output (`input_tokens`...), no `issue_` prefix.
- The per-issue ratios are means (sum over `issues`), as the other REP-6 ratios.

## Deliverable 1
- done: `delivered-quality` sums the input, cache read and output tokens of the messages that count for each issue,
  sub-agents included; `cardinal-report` reads the new summary header.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass, `delivered-quality.md` accepted |
| docs | `docs/usage.md`, `docs/architecture.md` |
| code review | low |

- red: QUA-15: the token columns were missing (`the tokens of its span are summed once per message, sub-agents included` failed).
- Code review: 1 finding (the transcript cells bypassed `cell`): fixed, `cell` takes any count.
- not verified: a real run on the transcripts of this machine.
- decisions needed: none.
