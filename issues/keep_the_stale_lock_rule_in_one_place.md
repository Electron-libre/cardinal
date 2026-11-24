---
priority: medium
depends_on: [share_one_git_runner_across_the_tools]
why: 'the CLI and the UI each copy the stale-lock rule and the main-worktree reader: a fix in one leaves the other wrong'
---
# Keep the stale-lock rule in one place

# Goal
`cardinal-ui` does not depend on the `cardinal` crate, so it copies the rule that tells a stale lock or claim
(WSL-1, STU-7), the parsing of the lock files and the reader of the main worktree. The outcome: one implementation of
each, read by both the CLI and the UI, so the two can no longer disagree on which session holds a worktree.

# Findings
* Code audit of 2026-11-24.
* `alive` and `started_after` (`kill -0`, `ps -o etimes=`, a 2-second threshold):
  `crates/cardinal/src/tools/issue_claim.rs:85-121` and `crates/cardinal-ui/src/source.rs:331-360`.
* `main_worktree`: `crates/cardinal/src/tools/worktree.rs:37-53` and `crates/cardinal-ui/src/merge.rs:24-38`,
  line for line but their error texts.
* The `key value` lines of the lock and claim files are parsed at `crates/cardinal/src/tools/issue_claim.rs:29-34`,
  `crates/cardinal/src/tools/workspace_lock.rs:35-39` and `crates/cardinal-ui/src/source.rs:395-399`.

# Approach
A decision of architecture (ADR 0030 makes the UI a crate of its own), to take at step 1:
1. A small library crate shared by both binaries (the lock files, the stale rule, the worktree list), with an ADR.
2. Or the UI asks `cardinal run` for the state of the claims and the worktrees (a `--json` output), as it already
   does for `next-issue` and `roadmap`, and drops its copies.

# Specs
* `specs/workspace-lock.md` (WSL-1), `specs/steering-ui.md` (STU-7): unchanged behaviour.

# Milestone
One implementation per rule

# Resources
* `docs/adr/0030-the-steering-ui-is-a-crate-of-its-own.md`, `docs/architecture.md` (steering UI section),
  `crates/cardinal-ui/Cargo.toml`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
One implementation, read by `cardinal` and `cardinal-ui`, of the `key value` lines of the lock and claim files, of the
stale rule (WSL-1, STU-7) and of the main worktree of `git worktree list`; no change of behaviour.
Means: approach 1, a std-only library crate `crates/cardinal-state` (ADR 0033 draft), not a `--json` output.

## Plan
1. [x] Stale rule and lock lines in `cardinal-state`: the crate with `field`, `alive`, `started_after` and
   `lock_taken`, its unit tests (a pid that started after the lock was written is stale; an absent pid is dead; the
   `key value` parser), used by `issue_claim`, `workspace_lock`, `rebase_issue`, `session_scratch` and the UI's
   `source.rs` in place of their copies; verified by those unit tests and the existing tests of WSL-1, CLM-11,
   RBI-4 and STU-7; makes true nothing new (unchanged behaviour). ~350 lines. (autonomous)
2. [ ] Worktree list in `cardinal-state`: a parser of `git worktree list --porcelain` and `main_worktree` over it
   (one error text: `no main worktree: the repository is bare`, that of RBI-4 and SUA-11), used by
   `worktree.rs`, `rebase_issue.rs` and the UI's `merge.rs`; verified by its unit tests and the existing tests of
   RBI-4, SUA-11 and the worktree tool. ~250 lines.

## Decisions
- Mode: autonomous
- Base commit: 8225a99c2adcc83c80c14be8bbcfa36c55ddc415
- Crate name: `cardinal-state` (1 a).
- ADR 0033 partly supersedes ADR 0030 (2 a).

## Deliverable 1 summary
- done: crate `cardinal-state` (`field`, `Lock`, `alive`, `lock_taken`, `started_after`), read by `issue_claim`,
  `workspace_lock`, `rebase_issue`, `session_scratch` and the UI's `source.rs`; their copies removed; ADR 0033, docs.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass (clippy -D warnings) |
| specs | pass (no requirement changed) |
| docs | CLAUDE.md, docs/architecture.md, ADR index, update-docs map, `state` commit scope |
| code review | see below |

- red: none (refactor: the crate's tests failed to compile before it existed)
- Code review: no findings
- not verified: none
- decisions needed: none
