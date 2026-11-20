---
priority: high
depends_on: [show_the_milestones_awaiting_their_release, write_the_release_steps_in_a_skill]
why: 'starting the release from the UI is what the milestone saves the user'
---
# Start a release session on the trunk

# Goal
Once a milestone awaits its release, the user opens a session in the main worktree and asks for the release by
hand. The outcome: a key of `cardinal-ui` on a milestone awaiting its release opens a zellij tab in the main
worktree running an interactive `claude` whose first message is `/release`, so the version, the reread, the review
and the tag stay in Claude Code.

# Findings
* STU-9, STU-11 and STU-12 start an issue session the same way (`zellij action new-tab --cwd ... --name ... --
  claude <first message>`), refused outside zellij.
* WSL-1: one session per worktree; a session already in the main worktree holds its lock.

# Approach
1. Refuse, saying why: outside zellij, no milestone awaiting its release, the main worktree not on the trunk, with
   uncommitted changes, or held by a running session, or a tab of the release already open.
2. Open the tab (named e.g. `release`) in the main worktree; offer `cardinal-ui --release` for tests.
3. Choose the key.

# Specs
* `specs/steering-ui.md`: amends STU-3; new requirements for the start of a release and its refusals.

# Milestone
Releases from the steering UI

# Resources
* `specs/steering-ui.md` (STU-9, STU-11, STU-12), `specs/workspace-lock.md`, `crates/cardinal-ui/src/start.rs`
