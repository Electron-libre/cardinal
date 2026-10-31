---
rank: 100
why: 'first: a small cleanup that nothing blocks; the two others wait on the user''s choice or on sessions'
---
# Drop the format 4 issue queue

# Goal
The format 5 orders the issues by the `rank` of each issue file (ADR 0019), and `cardinal` still reads the format 4
`issues/queue.md` so that a consumer not yet converted keeps working. Every consumer is now converted (the user, on
2026-10-31): the code, specs, tests and docs of the queue are dead weight that every reader of `next-issue` and
`close-issue` still pays. The outcome: no tool reads or writes `issues/queue.md`, and the docs no longer describe it.

# Findings
* `crates/cardinal/src/tools/next_issue.rs`: reads `queue.md` while it exists (NXT-4, `from_queue`), `--convert`
  turns it into ranks (NXT-5, `convert`), hints at `--convert` when a file has a rank the queue hides (NXT-6), and
  leaves `queue` out of the issue stems (NXT-1).
* `crates/cardinal/src/tools/close_issue.rs`: `drop_from_queue` removes the closed issue's line while the file exists.
* `crates/cardinal/src/tools/issue_worktree.rs`: skips the stem `queue`.
* Tests: `tests/next_issue.rs` (28 hits), `tests/close_issue.rs` (18), `tests/skills.rs` (2 allowed citations of
  `issues/queue.md` for NXT-6).
* Specs: `specs/next-issue.md` (NXT-1 exclusion, NXT-4 to NXT-6 and their scenarios), `specs/close-issue.md` (a CLS-4
  scenario cites `queue.md` as an example link), `specs/parallel-issues.md` (PAR-8 scenarios speak of "the queue").
* Docs: `docs/usage.md` (the paragraph on a project seeded before the format 5), the comment of `next-issue` in
  `files/cardinal.just` (`--convert`).
* `migrations/5.toml` renames `issues/queue.md` and names `just next-issue --convert` in its warning.
* `session_metrics.rs` and `memory.rs` hit `queue` for other meanings (a transcript record type, a BFS queue): out of
  scope.

# Approach
1. Withdraw NXT-4 to NXT-6 (marked withdrawn, ids kept), drop `queue.md excluded` from NXT-1, reword the scenarios of
   CLS-4 and PAR-8 that cite the queue.
2. Remove the queue paths of `next-issue`, `close-issue` and `issue-worktree`, and their tests.
3. Docs: `docs/usage.md`, `files/cardinal.just` (then `just sync`).
4. Open questions for step 1 of `issue-workflow`:
   - Does dropping the reading of a file need a raised format (ADR 0014)? A pinned revision runs its own binary, so an
     older consumer keeps its `next-issue`; a lock of the format 4 synced to the new source would lose its queue.
   - `migrations/5.toml` names `just next-issue --convert` in its warning text: keep it as history (a migration is
     applied once, from a lock of the format 4), or reword it.

# Specs
* `specs/next-issue.md`: NXT-1 amended, NXT-4, NXT-5, NXT-6 withdrawn.
* `specs/close-issue.md`, `specs/parallel-issues.md`: scenarios reworded, no requirement changed.

# Resources
* `docs/adr/0019-each-issue-file-carries-its-rank.md`, `docs/adr/0014-the-source-declares-the-format-cardinal-must-read.md`,
  `docs/adr/0017-the-source-declares-its-migrations.md`.
* `issues/done/rank_the_issues_without_a_shared_queue_file.md`: how the queue was replaced.

# Hints
* `issues/done/` and `CHANGELOG.md` cite the queue as history: leave them.
