---
rank: 118
why: 'right after the token waste rule, whose verdict it completes; before the advisor comparison, which needs no waste figure'
---
# Weigh the token waste per revision

# Goal
Rule `oversized-result` of `rule-breaches` (BRE-19) flags each tool result over 25,000 bytes with its size as
`detail`, but only its `breached_share` reaches a verdict: a change that halves the oversized bytes while each
session keeps one large result gets `no change`. And the rule is no instruction the agent breaks: the skills that
read the rules summary take a rule broken in a noticeable share of the sessions for a hook or rewording candidate,
which this one will always be. The expected outcome: a waste figure per revision, in bytes, that a change can expect
to move, and the waste rule kept out of the hook candidates.

# Findings
* Over the 200 local sessions with a Cardinal revision, `oversized-result` applied in 200, breached in 82, 119
  breaches; over all 1611 transcripts, the results over 25 KB weigh 6.4 MB of 67.3 MB of results.
* `audit-instructions` step 2 and `evaluate-harness` send a rule whose `breached_share` stays high to the audit as a
  hook candidate; the `--summary` of `rule-breaches` (BRE-3) has no bytes column.
* The `high` review of the waste rule raised both points.

# Approach
1. A `bytes` column in the `--summary` of `rule-breaches` (the sum of a numeric `detail`), read by the rules table of
   `cardinal-report`, and an `Expected-effect:` metric on it (`oversized_bytes_per_session`, say).
2. Or a column of `session-metrics` (the bytes of the results over the threshold per session), so the efficiency
   table of `cardinal-report` carries it, and the rule leaves `rule-breaches`.
3. In either case, name the waste rules apart in `audit-instructions` and `evaluate-harness`: a measure, not a
   hook candidate.

# Specs
* `specs/rule-breaches.md` (BRE-3, BRE-19), `specs/cardinal-report.md`, `specs/cardinal-verdicts.md` (VRD-1).

# Milestone
Lean agent loop

# Resources
* `issues/done/detect_token_waste_in_the_transcripts.md`: the measure and the rule.
* `crates/cardinal/src/tools/rule_breaches.rs`, `session_metrics.rs`, `cardinal_report.rs`.
* `files/.claude/skills/audit-instructions/SKILL.md`, `files/.claude/skills/evaluate-harness/SKILL.md`.

# Hints
* Approach 2 changes the columns of `session-metrics`, which `cardinal-report` reads by header: check REP-3.
