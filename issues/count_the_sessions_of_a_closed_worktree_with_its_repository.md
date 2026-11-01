---
priority: low
depends_on: [reuse_a_worktree_across_issues]
why: 'a measure that drifts only for the sessions of a removed worktree named apart from its branches'
---
# Count the sessions of a closed worktree with its repository

# Goal
`session-metrics` gives each session the project of its first `cwd` (EVA-7). When that `cwd` is gone, a directory
`<repo>-<x>` next to an existing `<repo>` counts with `<repo>` only when `<x>` is a `gitBranch` of the session, the
naming of the former `start-issue`. Since `open-worktree` (ADR 0023) a worktree is named apart from its branch
(`--branch`) and serves several issues on other branches, so once it is closed its sessions count as a project of
their own (`repo-<x>`), and `evaluate-harness` splits one project in two. The outcome: the sessions of a removed
worktree `<repo>-<x>` count with `<repo>`, whatever the branches.

# Findings
* High review of `reuse_a_worktree_across_issues`, 2026-11-01: `repository()` of
  `crates/cardinal/src/tools/transcripts.rs` matches the suffix `-<branch>` against the session's branches only.
* Scenario: `just open-worktree feat --branch issue-a`, then issue `issue-b` there, then `close-worktree feat`: the
  sessions of `../repo-feat` count as project `repo-feat`.

# Approach
1. Link a missing `<repo>-<x>` to `<repo>` when `<repo>` is an existing git repository, whatever `<x>`; or
2. read the git directory of the main repository for a record of the removed worktree (none is left after
   `git worktree remove`: measure first), or
3. keep the branch match and add the worktree name given to `open-worktree` (no record of it either).

# Specs
* `specs/evaluation.md`: EVA-7 amended.

# Milestone
Longer autonomy

# Resources
* `crates/cardinal/src/tools/transcripts.rs` (`repository`), `specs/evaluation.md` (EVA-7).
* `docs/adr/0023-worktrees-open-next-to-the-main-one-with-no-issue.md`.

# Hints
* A directory `<repo>-<x>` next to `<repo>` may be an unrelated clone: approach 1 then misattributes it.
