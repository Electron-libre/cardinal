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

# Milestone
One implementation per rule

# Resources
* The files above; the specs whose error messages name a git failure (`specs/rebase-issue.md` RBI-4).

# Hints
* The review gate hooks are hot paths: keep their runner free of extra processes.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
One git runner (`crates/cardinal/src/git.rs`, crate level so `gate.rs` and `lib.rs` use it too), one top-level lookup
and one reader of `git worktree list` for every caller, with no change of behaviour beyond the error texts, which all
become RBI-4's `git <args>: exit <code>: <first stderr line>`. Out of scope: the git calls of the tests.

## Plan
1. [x] `git` module and the `Result` callers: `run` (stdout, end trimmed), `bytes`, `run_stdin` (input written from a
   thread), `run_ok`, `toplevel`, `command` (for callers that set an environment or a deadline), one failure text;
   `rebase_issue`, `release`, `review_target`, `worktree`, `git_diff` and `transcripts::git`/`git_bytes` route through
   it; verified by unit tests of the module (tempdir repository) and the existing tests of those tools (RBI-4 text
   kept); ~350 lines. (autonomous)
2. [x] The hooks: `review_gate`, `lint_gate`, `workspace_lock`, `issue_claim`, `gate.rs`, `tools/mod.rs` (`repo_root`,
   `toplevel`), `update_notice` (its deadline kept through `command`); no process added on the hot paths; verified by
   their existing tests; ~450 lines. (autonomous)
3. [ ] The other tools and `lib.rs`: `close_issue`, `next_issue`, `rule_breaches`, `language_hint`, `session_wait`,
   `repo_files`, `commit_check`, `check_commits`, `delivered_quality`, `transcripts`, `lib.rs` (`git`, `git_output`,
   `resolves`, `ls-files`); verified by their existing tests; ~450 lines.
4. [ ] One reader of the worktrees: `worktree.rs`'s `worktrees`/`main_worktree` shared (`pub(crate)`), used by
   `rebase_issue::locate`, `issue_claim::trunk`, `next_issue` and the branch lookup of `close_issue`; verified by the
   tests of `rebase-issue`, `issue-claim`, `next-issue`, `close-issue` and `worktree`; ~200 lines.

The files over the size limit it touches (`lib.rs`, `close_issue.rs`, `rule_breaches.rs`, `delivered_quality.rs`) only
lose lines: the new code goes to `src/git.rs`.

## Decisions
- Mode: autonomous
- Base commit: 2e4eac504b1ea5ec88076b847b829fa078199a32
- `lib.rs`: the stderr of `clone`, `fetch`, `checkout` is captured, its first line in the error (1 a).

## Summaries

### Deliverable 1
- done: `crates/cardinal/src/git.rs` (`command`, `exec`, `bytes`, `run`, `run_ok`, `toplevel`, `Failure`); `release`,
  `review_target`, `worktree`, `git_diff`, `rebase_issue`, `transcripts`, `fixed_context`, `delivered_quality` route
  through it; the failures of `release`, `worktree`, `review_target` and `git_diff` now read
  `git <args>: exit <code>: <first stderr line>`; `transcripts::git` trims the end only.
- checks: project-check pass; deliverable-size 419 lines, within.
- red: none (a refactor; the module's own tests were written with it, the tools' tests stay green).
- Code review: 1 finding: `name` dropped a `-c` of the subcommand (`git grep -c`) from the failure text; fixed, only
  the leading `-C`/`-c` pairs are dropped, tested.
- not verified: none.
- decisions needed: none.

### Deliverable 2
- done: `review_gate`, `lint_gate`, `workspace_lock`, `issue_claim`, `gate.rs`, `tools/mod.rs` and `update_notice` run
  git through `crate::git` (`run_stdin` added; `gate.rs` keeps its cleared environment, `update_notice` its deadline,
  through `command`); `command` runs git in `dir` (`current_dir`) instead of `-C <dir>`, so a wrapper of git on the
  `PATH` still reads the subcommand first (the fake git of `tests/review_gate.rs`); no process added to the hooks.
- checks: project-check pass; deliverable-size 209 lines, within.
- red: none (a refactor: the tests of the hooks stay green; the fake-git test went red on `-C` before the fix above).
- Code review: 1 finding: `lint_gate::git` and `review_gate::git` no longer return the stdout of a failed git;
  not fixed, no caller reads it: `rev-parse`, `status`, `diff`, `ls-files` print nothing worth reading on failure, and
  the partial output of `hash-object --stdin-paths` was already replaced by one hash per file when short.
- not verified: none.
- decisions needed: none.
