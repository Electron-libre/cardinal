---
priority: low
why: 'only a session that crashed holding a working claim and whose pid a later process took, rare and settled by editing the claim by hand'
---
# Release a claim whose pid another process took

# Goal
A `working` claim names the `pid` of its session, and its liveness is `kill -0` on it (`Claim::alive`). When the
session crashed and a later process took the number, `cardinal run issue-claim --release <stem>` refuses with
`<stem>: session <id> still runs in <path>: end it first` forever, and `next-issue` shows the issue worked by a live
session. The expected outcome: such a claim counts as ended, as a worktree lock does (SUE-4, RBI-4).

# Findings
* `high` review of `let_rebase_issue_pass_a_lock_whose_pid_another_process_took` (2026-11-23), pre-existing:
  `crates/cardinal/src/tools/issue_claim.rs` (`Claim::alive`, the refusal of `--release`) and
  `crates/cardinal/src/tools/next_issue.rs` (its reads of `alive`). A claim has no mtime of its own, so
  `issue_claim::started_after` does not apply as it is.

# Approach
1. Compare the start of the process against the commit time of the claim ref (no format change).
2. Or store the time of the claim in it (a format change: `cardinal-format`, ADR 0014).

# Specs
* `specs/issue-claim.md`: the requirement of the liveness of a claim.

# Resources
* `crates/cardinal/src/tools/issue_claim.rs` (`Claim`, `started_after`), `crates/cardinal/src/tools/next_issue.rs`
* `specs/issue-claim.md`
