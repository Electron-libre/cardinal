---
priority: high
depends_on: [print_the_queue_and_the_roadmap_as_json]
why: 'the base every other view of the milestone is built on'
---
# Show the project state in a steering UI

# Goal
The user reads the state of a project through several commands (`next-issue`, `roadmap`, `git worktree list`) and
opens each worktree to see which session works in it. The outcome: a terminal UI, `cardinal-ui`, run in a project,
shows in one screen the milestones, the queue of issues, the claims and each worktree with its branch and the
session that holds it, refreshed as they change. Read only: it starts nothing and edits nothing.

# Findings
* Decided with the user (2026-11-20): a separate crate of this workspace (`crates/cardinal-ui`), installed apart,
  not a subcommand of `cardinal`, which is a launcher pinned per project; Ratatui; sessions in zellij (next issue).
* The workspace holds one member (`Cargo.toml`: `members = ["crates/cardinal"]`).
* The lock of a worktree is the file `cardinal-session` of its git dir (`workspace_lock.rs`, `lock_path`); the
  claims are the refs `refs/cardinal/claims/<stem>` (`issue-claim`).

# Approach
1. Write the ADR: the UI lives in a crate of its own and reads the project only through `cardinal run <tool> --json`
   and git, never the internals of `cardinal`.
2. Create the crate, its layout (milestones, queue, worktrees and sessions) and a refresh on an interval or on a
   change of `.git` and `issues/`.
3. Decide how it is published: the release workflow builds `cardinal` only; adding a binary there or leaving the UI
   to `cargo install` is a decision of step 2.

# Milestone
Steering UI

# Resources
* `docs/architecture.md`, "Tools and their binary" and "Parallel issues"
* `specs/workspace-lock.md`, `specs/issue-claim.md`, `specs/parallel-issues.md`
* `.github/workflows/release.yml`
* ADR 0011 (the user opens the worktree), ADR 0028 (a consumer needs only `cardinal`)

# Hints
* `cardinal`'s tests must not depend on the UI crate; `just check` runs clippy on the whole workspace.
