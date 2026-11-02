---
priority: medium
why: 'a fix before the comfort of its milestone: three copies of the roadmap parser, and a `## ` line in a comment can set a milestone done'
---
# Share one roadmap grammar between the tools

# Goal
Three tools read the `roadmap` documents, each with its own parser: `release` (`mark_released`, REL-9),
`close-issue` (`milestones_done`, CLS-8, a copy of the former) and `next-issue` (`milestones`, NXT-10). None skips a
`## ` line inside a fenced code block or an HTML comment, so such a line cuts the body of a milestone early: its
`State:` or its `Issues:` list lands in a fake section, and a commented template milestone listing real issues could
be set `done`. The outcome: one reading of the roadmap (milestone name, body lines, state, listed issues) that the
three tools share, fences and comments skipped.

# Findings
* 2026-11-02, the `high` review of
  [set_the_milestone_done_when_its_last_issue_closes](done/set_the_milestone_done_when_its_last_issue_closes.md)
  (commit acf19bd): two findings left unfixed there, the duplicated section parser and the `## ` line in a fence or
  an HTML comment.
* `seed/ROADMAP.md` describes the shape of a milestone in prose and a fenced block; a project that keeps a template
  milestone in a comment would hit the misread.

# Approach
1. A module of the roadmap grammar under `crates/cardinal/src/tools/` (milestones with their name, body range, state
   line, issue links), skipping fenced blocks and HTML comments like `close_issue::link_targets` skips fences.
2. `release`, `close-issue` and `next-issue` read through it; their specs gain the fence and comment case as a
   scenario where they draw that boundary.

# Specs
* `specs/release.md` (REL-9), `specs/close-issue.md` (CLS-8), `specs/next-issue.md` (NXT-10): a scenario each, or
  one requirement of the roadmap format if the three are amended together.

# Milestone
Longer autonomy

# Resources
* `crates/cardinal/src/tools/release.rs` (`mark_released`), `close_issue.rs` (`milestones_done`, `link_targets`),
  `next_issue.rs` (`milestones`).
* `seed/ROADMAP.md`: the shape of a milestone.
