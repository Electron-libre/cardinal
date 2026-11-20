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
2. [x] Screen: the three panels drawn with Ratatui, `--print` rendering them through its test backend (100 columns),
   the interactive loop (alternate screen, refresh every 2 s, `r`, `q`, `Esc`); verified by `tests/print.rs`
   (panels) and a unit test of the key handling; makes true STU-1, STU-2, STU-4, STU-H1. ~600 lines. (autonomous)
3. [x] Install and docs: `just install-ui`, `README.md`, `docs/architecture.md` (a section on the UI),
   `docs/usage.md`, `commit_scopes` gains `ui`; verified by `just check`. ~100 lines. (autonomous)
4. [x] Fix of the `high` review (important 1): with `cardinal-ui` in the repository, a `cargo install --git` that
   names no package fails; the launcher (BIN-5), the install it prints (BIN-20) and the CI installer (BIN-22) name
   `cardinal`, else `harness`; verified by `run_cli.rs`, `launcher_cli.rs`, `install_script.rs`. (autonomous)
5. [ ] Fixes of the `high` review of the UI (important 2 and 3, nits 4 to 10); verified by `tests/print.rs` and unit
   tests.

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

## Deliverable 2
done: the three panels drawn with Ratatui (titled boxes, the queue taking the rest, a help line), `--print` rendering
them through its test backend, the interactive screen (alternate screen, refresh every 2 s and on `r`, `q` and `Esc`
quit); STU-4 amended: the text is wider than 100 columns when a line needs it.

| check | result |
|---|---|
| tests | `tests/print.rs` 10/10, unit test of the keys 1/1 |
| lint | clippy clean |
| specs | `check-specs` clean, every STU requirement traced |
| docs | deferred to deliverable 3 by the plan |
| smoke | interactive run in a pseudo-terminal (`script`, 120x40): screen drawn, `r` then `q`, exit 0, alternate screen left |
| code review | `low`, 1 finding |

red: none observed: the layout test was written after the screen; the earlier `--print` printed `== Milestones ==`
and no box, which it would have failed on.

Code review: 1 finding
1. fixed: any event restarted the wait of 2 s, so a stream of keys or resizes could keep the screen from reading
   the project; the wait now counts from the last read (smoke rerun with keys between, exit 0). No test: the loop
   needs a real terminal.

not verified: the screen in a terminal other than the pseudo-terminal of `script` (colours, resize by hand)

decisions needed: none

## Deliverable 3
done: `just install-ui`; the UI in `README.md`, `docs/usage.md` and `docs/architecture.md` (its section and flow);
a row of the docs map; ADR 0030 and `specs/steering-ui.md` accepted.

| check | result |
|---|---|
| tests | `just check` green |
| lint | clean |
| specs | `check-specs` clean, STU accepted |
| docs | the pages above |
| code review | `low`, clean |

red: none: docs and a recipe, no test.

Code review: no findings

not verified: `cargo install --git` of the README, which needs the push

decisions needed: none

## Deliverable 4
done: the launcher, the install `check` prints and the CI installer name the package `cardinal` (else `harness`)
in their `cargo install --git`; BIN-5, BIN-20 amended, BIN-22 added.

| check | result |
|---|---|
| tests | `run_cli`, `launcher_cli`, `format_cli`, `install_script` green; `just check` green |
| lint | clean |
| specs | `check-specs` clean, BIN-22 traced |
| docs | `docs/architecture.md`; the managed installer synced |
| real cargo | `cargo install --locked --git file://<repo> --rev HEAD --root <tmp> cardinal` builds `cardinal` |
| code review | `low`, clean |

red: BIN-5: `cardinal run: no cardinal binary ... cargo is missing or failed to build it`, the fake cargo building
only a named package; BIN-20: the printed install lacked `cardinal`. BIN-22: not observed, the script changed first.

Code review: no findings

not verified: the CI of a consumer on a pushed commit

decisions needed: none

## Decisions
- Mode: autonomous
- Refresh: every 2 s and on `r` (1 a); install: `cargo install`, `just install-ui`, no published binary (2 a).
- Base commit: 6899cedc6b0ed22c783fb7ffd4151ee4ec8feebd
