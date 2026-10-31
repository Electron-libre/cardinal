---
rank: 225
why: 'waits for ~10 issue sessions with an Explore call on the largest project before the new measure'
---
# Judge the delegated exploration before keeping it

# Goal
`skill:issue-workflow` hands a broad exploration to an `Explore` subagent to keep the main context small. A subagent is
worth its cost only when the single loop degrades measurably from a cluttered context: each one starts cold, re-reads
what the main agent may already know, and hands back a summary that loses detail. The outcome: a verdict on the
delegation from the sessions since it shipped, on all the metrics it may move, and a decision to keep, narrow or fold
it back into the main loop.

# Findings
* The delegation shipped with `Expected-effect: main_context_peak down` only; `input_tokens` was left out on purpose
  ("its direction is unknown: each subagent starts cold"), and no quality effect was declared
  (`issues/done/delegate_the_exploration_of_the_issue_workflow_to_a_subagent.md`).
* Its own tracking left "not verified: whether the agent delegates in practice".
* The article on folding subagents into the main loop reports a customer about 2x faster with a single loop, and keeps
  subagents for three cases only: measured context degradation, parallel work on fresh contexts, tool confusion.

# Approach
1. Run `skill:evaluate-harness` over the revisions around the delegation commit: `main_context_peak` and
   `peak_sessions`, `input_tokens`, `turns`, the duration, and the quality table.
2. Count how often the agent actually delegates during an issue (the `Agent` calls with `Explore` per issue session).
3. Decide with the user: keep, narrow (e.g. history searches only) or fold back (remove the paragraph, commit with the
   opposite `Expected-effect:`).

# Milestone
Lean agent loop

# Resources
* `issues/done/delegate_the_exploration_of_the_issue_workflow_to_a_subagent.md`, `specs/cardinal-verdicts.md`.
* `files/.claude/skills/issue-workflow/SKILL.md`, step 1 "Delegate a broad exploration".
* https://claude.com/fr/resources/articles/building-agents-when-to-fold-subagents-into-your-main-agent-loop

# Hints
* A verdict on few sessions is `confounded` (REP-10): say the sample before any decision.
* Look at which issues changed outcome, not only the medians.

# Tracking

Plan: validated

## Understanding
Measure the delegation of step 1 of `issue-workflow` (shipped in 657bf10, 2026-10-28) on its sessions: the metrics of
`skill:evaluate-harness` around that commit, and how often the agent actually calls an `Explore` subagent per issue
session; then apply the user's decision (keep, narrow or fold back). Out of scope: a permanent metric of the delegations.

## Plan
1. [x] Measure: `skill:evaluate-harness` over the groups around 657bf10 (`main_context_peak`, `peak_sessions`,
   `input_tokens`, `cache_read_share`, `turns`, duration, quality, the verdict of 657bf10), plus a one-off count of
   the `Agent` calls with `subagent_type: Explore` per issue session from the transcripts (no committed script); the
   sample size and the issues that changed outcome written in `## Measure` of this issue, anonymised; verified by the
   report tables quoted (~40 lines under `issues/` only).
2. [x] Decision: per the user's answer, keep (a line in `## Decisions`), narrow (rewrite the paragraph of step 1 to
   the kept case) or fold back (remove the paragraph and its mention in step 3.1); `just sync`; commit with the
   opposite `Expected-effect:` on a fold-back only; verified by `just check` and `just check-cardinal` (~0-20 lines).

## Measure
Sessions since 657bf10 (2026-10-28 to 2026-10-31), four projects, one model (claude-opus-5-5), Claude Code 2.1.312
mostly; anonymised.

| fact | value |
|---|---|
| issue-workflow sessions since 657bf10 (this one left out) | 121 |
| sessions with an `Explore` call | 6 (5%), 8 calls; 5 in one consumer project, 1 here |
| other subagents | none |
| verdict of 657bf10 on `main_context_peak` | `confounded` (before b410ad5 92164, after f19d912..9341832 120575; the before groups mix Sonnet and Opus, other projects) |
| `main_context_peak` of the groups after (9 groups, 10 to 28 sessions each; the first, f19d912..9341832, also holds revisions before 657bf10) | 120575, 106461, 92029, 72645, 99612, 70933, 122587, 103630, 80193: no trend down |
| median of the sessions with / without `Explore` | peak 180656 / 94487; turns 174 / 67; duration 2519 s / 1125 s |

- The sessions that delegated are the longest issues (2.6x the turns): the peak gap is their size, not the delegation;
  6 sessions support no conclusion on its effect.
- The quality table (`input_tokens_per_issue` down from ~12-14M to 4-9M, `fixes_after_per_issue` down) moved over
  the same span as ~170 other commits and a model change: not attributable to the delegation.
- Conclusion: the agent almost never delegates (5% of the sessions); the paragraph costs fixed context in every
  session for a behaviour that barely happens, and no measured gain backs it.
- The usage grows with the project size (~13% of the sessions of the largest project, ~85k lines, none in the
  smaller ones), and no larger project has sessions: hence a narrowing and a new measure rather than a fold-back.

## Decisions
- Mode: autonomous
- Base commit: f42d70cbb5c778b5d4768dfb08e45dbe775a0067
- Delegations counted once from the transcripts, no committed script or `session-metrics` column.
- Decision (the user's, answer `a`): narrow the delegation to a search of the git history or over many files of a large repository (step 1
  of `issue-workflow`); no `Expected-effect:` since no effect is shown. The issue stays open for a new measure;
  the verdict of 657bf10 (`main_context_peak down`) keeps running on purpose, over the narrowed text.
- A `confounded` sample still leads to a decision, on the delegation count and the issues that changed outcome, saying so.

## Left
- Measure again once the largest project has ~10 issue sessions with an `Explore` call since the narrowing: their
  `main_context_peak` against sessions of the same size without one; then keep or fold back.

## Deliverable 2: narrowed delegation

- done: step 1 of `issue-workflow` delegates only a search of the git history or over many files of a large
  repository; step 3.1 points to it; copies synced.

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | pass |
| specs | pass |
| docs | `docs/workflow.md` missed here, fixed in deliverable 3 |
| code review | low, see below |

- red: none (skill text, no test).
- Code review: no findings
- not verified: whether the narrowed text changes how often the agent delegates.
- decisions needed: none.

## Deliverable 3: fixes of the `high` review

- done: the step 1 paragraph shortened to three lines and told apart from the `git log` of a resource;
  `docs/workflow.md` follows the narrowed delegation; the measure and the decisions corrected (9 groups, the first
  straddling 657bf10; the user's answer; why narrow rather than fold back; no opposite `Expected-effect:`).
- not fixed: no size threshold for a "large repository" (a project value the measure cannot set yet).

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | pass |
| specs | pass |
| docs | `docs/workflow.md` |
| code review | high: 10 findings, 9 fixed, 1 not fixed (above); low on the fixes, see below |

- red: none (skill text and docs, no test).
- Code review: no findings (low on the fixes).
- not verified: whether the narrowed text changes how often the agent delegates.
- decisions needed: none.
