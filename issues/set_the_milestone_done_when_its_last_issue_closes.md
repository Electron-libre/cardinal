---
priority: medium
depends_on: [let_the_close_of_an_issue_commit_without_a_review]
why: 'each milestone end costs a refused commit and a review of one word; it does not block the close'
---
# Set the milestone done when its last issue closes

# Goal
The Rules of `issue-workflow` tell the agent to set a milestone `done` in the commit that closes its last open issue,
but REV-20 lets the close through without a review only for the references `close-issue` rewrites: the edit of the
`State:` line is refused (its scenario says so), so every milestone end costs a refused commit and a `low` review of
one word. The outcome: closing the last open issue of a milestone commits without a review, its `State:` set `done`
by the tool, not by the agent.

# Findings
* 2026-11-02, a consumer project: the agent closed the last issue of a milestone and set it `done` as the Rules ask;
  the native hook refused the commit, the agent ran `/code-review low` on the diff (no findings) and committed.
* The decision "milestone set `done` in the close commit stays gated" (1 a) of
  [let_the_close_of_an_issue_commit_without_a_review](done/let_the_close_of_an_issue_commit_without_a_review.md)
  is what this issue revisits, with this cost measured.
* The roadmap format is already read by tools: `## <name>` headings (NXT-10), a `- State:` line and an `- Issues:`
  list of links, and `release` adds `- Released:` after `State:` (REL-9).

# Approach
1. `close-issue` sets `- State: done` on each milestone of the `roadmap` documents whose listed issues are all in
   `issues/done/` once the move is done (a milestone `in issues` only), and prints it; the agent no longer edits it.
2. REV-20 accepts, in a roadmap document, the `State:` line `close-issue` would set for the issues the index moves,
   computed the same way, next to the rewritten references; any other edit stays gated.
3. The Rules of `issue-workflow` drop the manual edit and keep the release proposal on the line `close-issue` prints.

# Specs
* `specs/close-issue.md`: a new CLS requirement for the milestone state.
* `specs/review-gate.md`: REV-20 and its scenario amended.

# Milestone
Longer autonomy

# Resources
* `crates/cardinal/src/tools/close_issue.rs`, `crates/cardinal/src/tools/review_gate.rs` (`records_close_only`).
* `files/.claude/skills/issue-workflow/SKILL.md`, Rules.
* `specs/release.md` (REL-9), `specs/next-issue.md` (NXT-10): how the roadmap is read today.

# Hints
* A milestone that lists an issue still open, or none, stays as it is; `idea` and `shaped` ones are the user's.
* The gate must recompute the state from the index, not trust the text of the roadmap.
