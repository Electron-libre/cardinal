---
priority: medium
why: 'nine copies of the git runner and ten of the top-level lookup handle errors each their own way and slow every tool change'
---
# Share one git runner across the tools

# Goal
Each tool of `crates/cardinal` defines its own `fn git` and its own lookup of the repository root, with differing
trimming, stderr handling and error texts, and `rebase-issue` parses the worktree list again instead of using
`worktree.rs`. The outcome: one git runner, one top-level lookup and one main-worktree reader in the crate, used by
every tool, with no change of behaviour beyond uniform error texts.

# Findings
* Code audit of 2026-11-24.
* `fn git` defined 9 times: `crates/cardinal/src/lib.rs:397`, `crates/cardinal/src/tools/issue_claim.rs:123`,
  `lint_gate.rs:35`, `rebase_issue.rs:52`, `release.rs:558`, `review_gate.rs:53`, `review_target.rs:80`,
  `update_notice.rs:224`, `worktree.rs:18` (all under `crates/cardinal/src/tools/` but the first).
* `rev-parse --show-toplevel` issued in `crates/cardinal/src/tools/mod.rs:248,261`, `close_issue.rs:625`,
  `review_gate.rs:362`, `workspace_lock.rs:153`, `lint_gate.rs:50`, `rule_breaches.rs:409`, `issue_claim.rs:249`,
  `transcripts.rs:368`, `language_hint.rs:15`, `crates/cardinal/src/gate.rs:118`.
* `crates/cardinal/src/tools/rebase_issue.rs:60` (`locate`) parses `git worktree list --porcelain` again instead of
  `main_worktree` (`crates/cardinal/src/tools/worktree.rs:37`); a lookup by branch is repeated at
  `worktree.rs:84` and `close_issue.rs:635`.

# Approach
1. Add a `git` module to the tools (`run`, `run_ok`, `run_stdin`, `toplevel`) with one error text naming the
   command, its exit code and the first stderr line.
2. Route the tools through it, one deliverable per few tools; keep the error texts a spec or a test fixes.
3. Make `rebase_issue` and `close_issue` read the worktrees through `worktree.rs`.

# Resources
* The files above; the specs whose error messages name a git failure (`specs/rebase-issue.md` RBI-4).

# Hints
* The review gate hooks are hot paths: keep their runner free of extra processes.
