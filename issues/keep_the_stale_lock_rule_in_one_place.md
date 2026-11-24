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

# Resources
* `docs/adr/0030-the-steering-ui-is-a-crate-of-its-own.md`, `docs/architecture.md` (steering UI section),
  `crates/cardinal-ui/Cargo.toml`.
