---
priority: medium
depends_on: [draw_the_progress_of_a_milestone_as_a_bar]
why: 'how far an issue at work went is invisible from the UI; reuses the bar of the milestone'
---
# Draw the progress of an issue as a bar

# Goal
The queue shows the state of an issue (`ready`, a claim, `merge pending`) but not how far its work went. The
expected outcome is a progress bar on the row of an issue at work: the share of the deliverables of its plan done.

# Findings
* `skill:issue-workflow` writes the plan in `## Plan` of `# Tracking`, one line `<n>. [x]` or `<n>. [ ]` per
  deliverable; `delivered_quality.rs` already counts those lines (QUA-3).
* The plan of an issue at work lives in its worktree (its branch), not on the trunk; an issue not yet planned has
  no plan line.
* `next-issue --json` (NXT-19) carries no plan; STU-3 limits what the UI reads.

# Approach
1. Source: add the counts (`done`, `planned`) to the document of NXT-19, read from the branch of the claim, or let
   the UI read the issue file in the worktree of its claim (amending STU-3).
2. Draw the bar of `draw_the_progress_of_a_milestone_as_a_bar` after the state, none when there is no plan.
3. Consider the steps before the plan (understanding, spec, plan validated) as a first segment.

# Specs
* `specs/steering-ui-queue.md`: SUQ-3 (the row); `specs/next-issue.md`: NXT-19 or `specs/steering-ui.md`: STU-3.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal/src/tools/delivered_quality.rs` (plan line count), `crates/cardinal-ui/src/panels.rs`,
  `.claude/skills/issue-workflow/SKILL.md` (the `# Tracking` layout).
