---
priority: low
why: 'no metric is wrong today; the unknown count only hides a future format change'
---
# Know the new line types of the transcripts

# Goal
`session-metrics` counts in `unknown_lines` a transcript line whose `type` is not in its list (EVA-9), to notice when
the format moves. Claude Code now writes four types it does not know, so the count is no longer zero and a real
change would go unnoticed among them. The outcome: the known list holds these types, and `unknown_lines` is back to 0
on the current transcripts.

# Findings
* Measured 2026-11-03 over 525 sessions: 1,511 unknown lines.
* The types missing from the list: `tools-used` (708 lines in 215 sessions, e.g. `{"type":"tools-used","tools":["Bash"]}`),
  `relocated` (402 lines, 6 sessions), `worktree-state` (402 lines, 6 sessions), `permission-mode` (1 line).

# Approach
1. Add the four types to the known list, with a test per type.
2. Check whether `tools-used` gives a cheaper source for a column already computed, and say so in the issue if it does.

# Specs
* `specs/evaluation.md` EVA-9.

# Milestone
Lean agent loop

# Resources
* `crates/cardinal/src/tools/session_metrics.rs`, `KNOWN_TYPES`; `specs/evaluation.md`.
