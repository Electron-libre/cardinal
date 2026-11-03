---
priority: medium
why: 'the only token waste the report measures, and today it mostly counts the diffs a review must read'
---
# Tell the needed oversized results from the waste

# Goal
`rule-breaches` counts every tool result over 25,000 bytes as `oversized-result` (BRE-19), the token waste
`cardinal-report` weighs per session. Over the last five complete groups it reads 8 to 42 KB per session with no
trend, but most of these results are diffs a reviewer must read, so the measure cannot tell a regression of the agent
from the size of the issues. The outcome: the report separates the results a step needs from those the agent could
have bounded, and the avoidable ones shrink, starting with the report's own output.

# Findings
* Measured 2026-11-03 over 525 sessions of 7 projects: 174 `oversized-result` breaches, 138 in sub-agent transcripts
  and 36 in the sessions' own.
* 121 of the 138 come from the reviewer sub-agents of the built-in `/code-review`: 95 at `high` (the review of a whole
  issue, `skill:issue-workflow` step 6) and 26 at `low`; 2 from `Explore`.
* By tool, all transcripts: `Read` 72 results (3.0 MB), `git` 60 (1.6 MB), `cd …;` commands 17, `cat` 11, `sed` 5.
* The 36 of the sessions' own: `cd …;` 14, `cat` 9, `Read` 7, `git` 2, `WebFetch` 2, `cardinal` 1.
* `cardinal run cardinal-report` printed 30 KB for 19 groups (its rules table alone 197 lines), so the audit that read
  it breached the rule; it has no option to limit the groups.

# Approach
1. Locate each breach by its source in the summary or the report: the sub-agents of a review, other sub-agents, the
   session's own results; candidates: a column per source, or a `--source` filter of `rule-breaches`.
2. Give `cardinal-report` a `--last <n>` (the last `n` groups) and use it in `skill:evaluate-harness`.
3. For the review diffs, measure first how much of a whole-issue diff is the issue file, specs and docs, then decide
   whether the `high` review should leave them out.
4. For the session's own `cat` and `Read` of large files, check whether a rule or a hint would bound them (a range, a
   `head`), and whether it pays its fixed context.

# Specs
* `specs/rule-breaches.md` BRE-19, BRE-20; `specs/cardinal-report.md` (a new requirement for `--last`).

# Milestone
Lean agent loop

# Resources
* `specs/rule-breaches.md`, `specs/cardinal-report.md`, `files/.claude/skills/evaluate-harness/SKILL.md`.
* `files/.claude/skills/issue-workflow/SKILL.md`, steps 3 and 6 (the `/code-review` calls).
* `issues/judge_the_delegated_exploration_before_keeping_it.md` (the sub-agents it judges are not the source here).

# Hints
* A sub-agent's oversized result does not enter the main context; it costs the sub-agent's tokens, not the peak.
* A sub-agent transcript has a `.meta.json` next to it with its `agentType`; the review reviewers are
  `general-purpose` with no description and a first prompt that starts with the review level.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-03

## Understanding
Split the `oversized-result` waste by its source (the session's own results, the `/code-review` reviewers, other
sub-agents), give `cardinal-report` a `--last <n>` used by `skill:evaluate-harness`, then measure the two avoidable
sources (the docs share of a whole-issue diff, the session's own `cat` and `Read`) before deciding any rule.
Means: two new waste rules of `rule-breaches` (BRE-22) rather than a column or a `--source` filter, so the summary,
the rules table and the verdicts need no change.

## Plan
1. [x] Waste by source: `oversized-result` keeps the session's own results, `oversized-result-review` and
   `oversized-result-subagent` take the sub-agents' by their `.meta.json`; verified by tests of `rule-breaches`
   (own, reviewer, `Explore`, no `.meta.json`) and of its summary; makes true BRE-19, BRE-20, BRE-22. (autonomous)
2. [x] `cardinal-report --last <n>`: only the rows of the last `n` groups, verdicts over every group; refused with
   `--by setup` or a bad value; `skill:evaluate-harness` passes `--last 6`; verified by tests of `cardinal-report`;
   makes true REP-17. (autonomous)
3. [ ] Measure the avoidable sources: share of `issues/`, `specs/`, `docs/` in the whole-issue diffs of the closed
   issues, and the session's own `cat`/`Read` breaches by file and need; findings and the decision proposed in this
   issue, a follow-up issue for any change of a skill or rule; verified by the numbers recorded here.

## Decisions
- Mode: autonomous
- Base commit: f116984c0003b4b49ad4e4207326f221e426c5b0
- Source of the waste: three waste rules (`oversized-result`, `oversized-result-review`, `oversized-result-subagent`), not a `--source` filter.
- `skill:evaluate-harness` passes `--last 6` (the five complete groups and the open one).

## Deliverable 1
done: `rule-breaches` tells the oversized results by source: `oversized-result` (the session's own),
`oversized-result-review` (a `general-purpose` sub-agent with no description), `oversized-result-subagent` (any other).

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (BRE accepted, REP draft until deliverable 2) |
| docs | `docs/evaluation.md`, `docs/usage.md` |
| code review | see below |

red: BRE-19, BRE-20, BRE-22: `FAIL - a tool result over 25,000 bytes is a breach at its line, of the rule of its transcript's source`

Code review: no findings (it noted the draft REP-17 in the diff: the spec drafts go with the first deliverable).

not verified: the split on the real transcripts (deliverable 3 reads it).

## Deliverable 2
done: `cardinal-report --last <n>` prints only the last `n` groups (verdicts over all of them, refused with
`--by setup`); `skill:evaluate-harness` passes `--last 6` and tells the three waste rules apart.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (REP accepted) |
| docs | `docs/usage.md`, `files/.claude/skills/evaluate-harness/SKILL.md` (synced) |
| code review | see below |

red: REP-17: `FAIL - --last succeeds` (the option was refused as unknown)

Code review: no findings

not verified: the size of the real report with `--last 6` (deliverable 3 runs it).
