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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
A key of `cardinal-ui` (and `cardinal-ui --release`) opens a zellij tab `release` in the main worktree running
`claude /release`, refused with its reason when it cannot (STU-26 to STU-28, STU-3 and STU-15 amended). The release
itself stays in `skill:release`; no milestone selection: one release covers every milestone awaiting it.

## Plan
1. [x] Start the release session: a module `release.rs` (the checks of STU-27, sharing those of the main worktree
   with `merge.rs`, then the tab of STU-28), `--release` in `main.rs`, the key `v` and its help in `screen.rs` (one
   task more, kept under 800 lines); verified by `tests/release.rs` through the binary with fake `cardinal` and
   `zellij`, and an internal test of the key; makes true STU-26, STU-27, STU-28, STU-3, STU-15. ~450 lines. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: f73616d022735f245480bc86c5f1c76b17039daf
- Key: `v` (version), `r` being the refresh.
- A project that turned the skill `release` off is refused (STU-27), pointed to the steps of `cardinal run release`.
- The split of `specs/steering-ui.md` (over `spec_max_bytes`) is a follow-up issue, not part of this one.

## Deliverable 1
done: `v` and `cardinal-ui --release` open a tab `release` in the main worktree running `claude /release`, refused
with its reason (STU-26 to STU-28); the checks of the main worktree are shared with the merge.

| check | result |
|---|---|
| tests | `cargo test -p cardinal-ui`: pass (9 new in `tests/release.rs`, 1 internal in `screen.rs`) |
| lint | clippy -D warnings: pass |
| specs | `check-specs`: STU-26 to STU-28 traced, spec `accepted` |
| docs | README, `docs/usage.md`, `docs/architecture.md` |
| code review | low: no findings |

red: STU-H8, STU-26, STU-27, STU-28: `cardinal-ui --release` exited 2 (usage: unknown argument)
Code review: no findings
not verified: the tab opened in a real zellij session (fakes only).
decisions needed: none
