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

# Specs
* `specs/steering-ui.md` (new, STU-1 to STU-8).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
A new binary `cardinal-ui` (crate `crates/cardinal-ui`, Ratatui) shows the milestones, the queue and the worktrees
with their sessions, read through `cardinal run next-issue --json`, `roadmap --json` and git, refreshed every 2 s.
Out of scope: starting sessions (next issue), measures, editing, publishing a binary. `--print` renders one screen
as text for scripts and the tests, which run a fake `cardinal` named by `CARDINAL_UI_CARDINAL`.

## Plan
1. [x] Crate and reading: workspace member, ADR 0030, `source` module (run the tools, check `version`, the errors of
   STU-6; worktrees and their lock, STU-7) and a plain-text `--print` of the read state; verified by
   `crates/cardinal-ui/tests/print.rs` with a fake `cardinal`; makes true STU-3, STU-5 to STU-8. ~700 lines. (autonomous)
2. [ ] Screen: the three panels drawn with Ratatui, `--print` rendering them through its test backend (100 columns),
   the interactive loop (alternate screen, refresh every 2 s, `r`, `q`, `Esc`); verified by `tests/print.rs`
   (panels) and a unit test of the key handling; makes true STU-1, STU-2, STU-4, STU-H1. ~600 lines.
3. [ ] Install and docs: `just install-ui`, `README.md`, `docs/architecture.md` (a section on the UI),
   `docs/usage.md`, `commit_scopes` gains `ui`; verified by `just check`. ~100 lines.

## Deliverable 1
done: crate `cardinal-ui` (workspace member), its reading of the tools, the worktrees and their locks, and a
plain-text `--print`; ADR 0030 drafted; the scope `ui` added to `commit_scopes`.

| check | result |
|---|---|
| tests | `cardinal-ui` `tests/print.rs` 9/9 |
| lint | clippy clean on the workspace |
| specs | `check-specs` clean (STU draft, STU-2 left to deliverable 2) |
| docs | deferred to deliverable 3 by the plan; module docs written |
| code review | `low`, 2 findings |

red: STU-H1, STU-1, STU-3 to STU-8: the empty binary printed nothing (`no line with One` and the usage assertion).

Code review: 2 findings
1. not fixed: `on_hold` read as on hold whenever it is not `null`, for fear of a boolean: RMP-8 makes it a string
   (`""` for an empty reason, still on hold) or `null`, so the reading is right.
2. fixed: `next-issue` exiting 1 with no document showed `not JSON`; it now shows its exit code and stderr (test
   `a_queue_that_fails_with_exit_1_and_no_document_shows_its_stderr`).

not verified: the interactive screen (deliverable 2)

decisions needed: none

## Decisions
- Mode: autonomous
- Refresh: every 2 s and on `r` (1 a); install: `cargo install`, `just install-ui`, no published binary (2 a).
- Base commit: 6899cedc6b0ed22c783fb7ffd4151ee4ec8feebd
