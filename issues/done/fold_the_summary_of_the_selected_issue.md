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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
Show the `# Goal` of the selected issue in a panel under the queue, unfolded and folded with `i`, folded at start.
The goal reaches the UI through a new `goal` field of `next-issue --json` (NXT-19), so STU-3 stays as it is; the
summary shows the issue file of the work tree `cardinal-ui` reads (the trunk's from the main worktree).

## Plan
1. [x] Goal in the JSON: `next-issue --json` gives each issue its `goal`; verified by a test of the binary in a new
   `crates/cardinal/tests/next_issue_goal.rs` (`tests/next_issue.rs` is over the file size limit); makes true NXT-19. (autonomous)
2. [x] Summary panel: `i` folds and unfolds a panel under the queue with the title and the wrapped goal of the selected
   issue; the new code in a module `crates/cardinal-ui/src/summary.rs` (`screen.rs` is over the file size limit, it
   only gets the call), `source.rs` reads `title` and `goal`, `keys.rs` maps `i`; verified by Ratatui tests of the
   drawn screen; makes true SUQ-6, SUQ-7, SUQ-H3. (autonomous)
3. [x] Help line: `i summary` after `tab panel` when the selection is on an issue of the queue; verified by the tests
   of `help.rs`; makes true SUA-15, SUA-H7. (autonomous)
4. [x] Docs and high review: the docs of the whole issue (STU-1, the architecture diagram) and the fixes of the
   `high` review; verified by the tests of `goal` and of the summary; makes true NXT-19, SUQ-6, SUQ-7, STU-1. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 499a13f6bb532654b7c5c8caff07ca96d85d7b2f
- Source: the `goal` field of `next-issue --json`, the whole `# Goal` section; key `i`, folded at start, hidden while the selection is in the worktrees (all recommendations taken).

## Summaries

### 1. Goal in the JSON
done: `next-issue --json` gives each issue its `goal` (NXT-19); the spec drafts of the issue committed with it.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (the three specs `draft` until their last requirement is traced) |
| docs | `docs/usage.md` names the title and the goal |
| code review | see below |

red: NXT-19: `left: Null, right: String("First line second line.\nThird. ``` # not a heading ```")`

Code review: 1 findings
- A `# Goal` line inside a code block before the section was taken as its start: fixed, the code blocks are tracked
  from the top of the body, and the test holds such a block.

not verified: none. decisions needed: none.

### 2. Summary panel
done: `i` folds and unfolds, between the queue and the worktrees, a panel titled with the title of the selected issue
holding its goal wrapped (`summary.rs`); the title of a panel became a `Cow` to hold it.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass; `next-issue.md` and `steering-ui-queue.md` set `accepted` |
| docs | README, `docs/usage.md`, `docs/architecture.md` (module table) |
| code review | see below |

red: SUQ-H3, SUQ-6, SUQ-7: `a summary titled Do a: ┌ Queue ───…` (no panel drawn), `shown again`, `left: [] right: ["one two", "three"]`
- `screen.rs` stays over the file size limit (840 lines before, 852 now: the call of the summary and the fold
  state): its split by responsibility is a refactor of its own, proposed as a follow-up.

Code review: 1 findings
- The last row cut to a panel of one column kept a grapheme before its `…` (two columns): fixed, it holds `…` alone,
  with a test.

not verified: the screen in a real terminal (the tests draw it in Ratatui's test backend). decisions needed: none.

### 3. Help line
done: the help line offers `i summary` after `tab panel` with the selection on an issue of the queue, also while a
task runs (SUA-15); `steering-ui-actions.md` set `accepted`.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | `docs/usage.md` |
| code review | see below |

red: SUA-15, SUA-H7: `left: "q quit  r refresh  ↑↓ select  tab panel  f merge  E end merged  g charts"`, `right: "… tab panel  i summary  f merge …"`

Code review: no findings

not verified: none. decisions needed: none.

### 4. Docs and high review
done: STU-1 names the summary between the two panels, the architecture diagram the goals; the fixes of the `high`
review of the issue (below), NXT-19, SUQ-6 and SUQ-7 reworded to match.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | `docs/usage.md`, `docs/architecture.md`, `specs/steering-ui.md` |
| code review | see below |

red: SUQ-7, NXT-19: the short screen showed its last rows without `…` (checked by disabling the fix); `left: "First line second line.\nThird. ``` # not a heading ``` ~~~"`
- `screen.rs` stays over the file size limit (857 lines; the layout taken out of `render` for the summary): its split
  is the follow-up proposed in deliverable 2.

High review (`499a13f..`): 7 findings
- important: on a short screen the layout cut the summary without its `…`: fixed, the summary takes no more rows than
  the layout gives it (`screen::layout` shared with `render`), with a test of the case.
- `goal` ended at any line opening with `#`: fixed, only at `# ` as `check-issue` reads a section.
- list items were joined into one line: fixed, each opens a paragraph.
- code blocks were copied as words and `~~~` not read: fixed, both fences read, the blocks left out.
- the panels were cloned at each tick: fixed, borrowed while the summary is folded.
- the help line and `i` used two predicates: fixed, both use `summary::selected`.
- SUQ-6 said the summary hides on any error of STU-6: reworded, it hides when `next-issue` cannot be read (the queue
  then holds no issue); a failed `roadmap` still lists the issues.

Code review: no findings

not verified: the screen in a real terminal (the tests draw it in Ratatui's test backend). decisions needed: none.
