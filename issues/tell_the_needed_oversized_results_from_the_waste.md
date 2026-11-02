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
