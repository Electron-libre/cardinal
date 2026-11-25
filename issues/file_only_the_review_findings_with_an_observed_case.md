---
priority: high
why: 'each unobserved finding filed costs a whole issue and its own `high` review, which files the next one'
---
# File only the review findings with an observed case

# Goal
`skill:issue-workflow` turns every review finding outside the deliverable into an issue, observed or not. A `high`
review reads the code and reports the corners of a format nobody writes; each one filed becomes an issue whose own
`high` review reports the next corner. The prompts should file a finding only when a real case shows it (a file of a
consumer, a run, a report of the user) or its cost is measured, and list the others in the summary, not filed.

# Findings
* 2026-11-25, this repository: five issues of the milestone "One implementation per rule" in a row
  (`read_the_sections_of_an_issue_with_the_shared_fence_grammar` to
  `end_the_sections_of_delivered_quality_at_every_heading`) were each born from the `high` review of the previous one;
  the review of the last one filed two more (a heading indented by 1-3 spaces, a `# Tracking` comment in the front
  matter), which the user dropped: "J'ai l'impression que tu tournes en rond."
* `issues/done/`: 34 of 285 issues cite the review of another issue in their Findings, 33 of them hold "not run" or
  "read from the code".
* `files/.claude/skills/issue-workflow/SKILL.md` step 3.5: "a finding outside the requirements of the deliverable
  becomes an issue (`skill:create-issue`)", with no condition; `files/.claude/skills/create-issue/SKILL.md` step 2
  counts "code" among what was "actually observed".

# Approach
1. `issue-workflow` step 3.5 and the `high` review: a finding outside the deliverable becomes an issue only with an
   observed case or a measured cost; otherwise it is listed `not filed: <why>` in the summary.
2. `create-issue` step 2: an issue born from a review finding names its observed case in `# Findings`; reading the
   code shows where a defect is, not that it happens.
3. Optionally, a finding whose source is the review of an issue itself born from a review finding goes to the user
   instead of being filed (the chain this issue met).

# Resources
* `files/.claude/skills/issue-workflow/SKILL.md` (step 3.5, the `high` review), `files/.claude/skills/create-issue/SKILL.md`
  (step 2), `files/.claude/skills/issue-workflow/details.md`.
* `issues/done/end_the_sections_of_delivered_quality_at_every_heading.md` (its `High review` section).

# Hints
* `files/` is the source: run `just sync` after editing it; a commit may carry `Expected-effect:` (CLAUDE.md).
