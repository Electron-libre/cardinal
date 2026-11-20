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
