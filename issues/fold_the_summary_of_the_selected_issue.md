---
priority: medium
why: 'the stem alone does not tell what an issue is about; reading it means leaving the screen'
---
# Fold the summary of the selected issue

# Goal
The queue of `cardinal-ui` names each issue by its stem only (SUQ-3); to know what it is about, the user opens the
file in another terminal. The expected outcome is the summary of the selected issue shown on the screen, folded and
unfolded with one key.

# Findings
* `cardinal run next-issue --json` (NXT-19) gives the `title`, the `why` of the priority and no section of the body.
* STU-3 limits what the UI reads to the tools it lists: reading `issues/<stem>.md` itself is outside it.
* Every issue has a `# Goal` section (template of `skill:create-issue`), a few sentences long.

# Approach
1. Source: add the `goal` (the first paragraph of `# Goal`) to the document of NXT-19, or amend STU-3 so the UI
   reads the issue file of the selection.
2. Where: a panel under the queue or a few rows under the selected row, its title the `title` of the issue.
3. Fold: one key toggles it (folded by default or not, to decide), the state kept across reads and moves of the
   selection; the help line (SUA-15) offers it when an issue is selected.

# Specs
* `specs/steering-ui-queue.md`: a new requirement (summary, fold key).
* `specs/next-issue.md`: NXT-19 if the goal joins the JSON document; or `specs/steering-ui.md`: STU-3.
* `specs/steering-ui-actions.md`: SUA-15 (the key in the help line).

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/panels.rs`, `crates/cardinal-ui/src/keys.rs`, `crates/cardinal-ui/src/help.rs`,
  `crates/cardinal/src/tools/next_issue.rs`.

# Hints
* An issue of a linked worktree may have a newer `# Goal` on its branch than on the trunk: say which one is shown.
