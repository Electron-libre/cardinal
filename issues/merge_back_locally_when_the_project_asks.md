---
priority: high
why: 'a session started from the UI must leave its branch for a local merge instead of pushing it and opening a pull request'
---
# Merge back locally when the project asks

# Goal
"Merge back" of `issue-workflow` closes the issue on its branch, then pushes the branch and opens a pull request.
A project whose issue branches are merged locally into the trunk (`git merge --ff-only <branch>` in the main
worktree) has no use for the push nor the pull request, and the user has to stop the agent by hand each time. The
outcome: a key of `.claude/cardinal.toml` makes "Merge back" stop once the close is committed and `cardinal check`
passed, and print the local merge command, so the steering UI (or the user) merges it.

# Findings
* `files/.claude/skills/issue-workflow/details.md`, "Merge back", step 3: push to the remote the trunk tracks, then
  `gh pr create`; with no remote or no CLI of the host it already stops and prints `git merge --ff-only <branch>`.
* 2026-11-20, after the close of `show_the_measures_in_the_steering_ui`, the agent pushed and opened a pull request
  and the user asked for a local merge instead; today only a memory of the user's machine holds that preference.
* `seed/.claude/cardinal.toml` and `docs/usage.md` list the project keys (`roadmap_archive` is an example).

# Approach
1. Name the key (e.g. `merge_back = "local"` or `"pull-request"`, the latter the default) and where it lives:
   `.claude/cardinal.toml` (the team's choice) rather than `cardinal.local.toml`.
2. Make step 3 of "Merge back" read it: `local` stops after step 2 and prints the merge command for the main
   worktree, without pushing; the agent never runs the merge from its linked worktree.
3. Document it in the seed and `docs/usage.md`; set it to `local` in this repository.

# Milestone
Merge back from the steering UI

# Resources
* `files/.claude/skills/issue-workflow/SKILL.md`, "Merge back"; `files/.claude/skills/issue-workflow/details.md`, "Merge back"
* `seed/.claude/cardinal.toml`, `docs/usage.md`, `docs/workflow.md` ("Parallel issues")

# Hints
* The skills are managed content: edit `files/`, then `just sync`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
A `[project]` key `merge_back` (`"pull-request"`, the default, or `"local"`) of `.claude/cardinal.toml` makes step 3
of "Merge back" stop after the close commit and `cardinal check`, printing `git merge --ff-only <branch>` for the main
worktree, with no push nor pull request. Out of scope: running the merge (next issues of the milestone). No spec: the
key is read by the skill only, as `test_conventions` or `confidential`.

## Plan
1. [x] Merge back locally: `merge_back` read at step 3 of "Merge back" (`files/.claude/skills/issue-workflow/details.md`,
   pointer in `SKILL.md`), the commented key in `seed/.claude/cardinal.toml`, `docs/usage.md` and `docs/workflow.md`,
   `merge_back = "local"` in this repository's `.claude/cardinal.toml`, `just sync`; verified by `just check` (managed
   copies in sync) and a reread of the step. (autonomous)
2. [x] Review fixes of the whole issue (`high` review): rebase proposal when the trunk moved, unknown value is a
   stop, `docs/architecture.md` key table, the diagram of `docs/workflow.md`; verified by `just check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 07583e9d2be4bfe35112eeb4e5ddf7fde8db5a8e
- Key: `merge_back = "pull-request"` (default) or `"local"` (1a).
- The memory of the local merge is deleted once the key is committed (2a).

## Deliverable 1
done: `merge_back = "local"` makes step 3 of "Merge back" stop on the local merge command; seed, docs and this repository's config.

| check | result |
|---|---|
| tests | `cardinal run project-check` green |
| lint | green |
| specs | green (no spec: a key read by the skill only) |
| docs | `docs/usage.md`, `docs/workflow.md` |
| code review | see below |

red: none
Code review: no findings

not verified: a real close on a branch with `merge_back = "local"` (the next issue session exercises it).

decisions needed: none

## Deliverable 2
done: the 5 findings of the `high` review of the issue fixed.

| check | result |
|---|---|
| tests | `cardinal run project-check` green |
| specs | green |
| docs | `docs/architecture.md`, `docs/workflow.md` |
| code review | see below |

red: none
High review: 5 findings: fast-forward fails once the trunk moved (fixed: check the ancestry, propose a rebase on the
user's go); `merge_back` missing from `docs/architecture.md` (fixed); a mistyped value pushes (fixed: a stop); the
diagram drew the local merge on the host (fixed: `alt` block); the UI named as able to merge (fixed: removed).
Code review: no findings

not verified: none

decisions needed: none
