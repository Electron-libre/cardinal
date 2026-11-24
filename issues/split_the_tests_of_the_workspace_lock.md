---
priority: low
why: 'a comfort: the file passed the file size limit by 47 lines, no behaviour depends on it'
---
# Split the tests of the workspace lock

# Goal
`crates/cardinal/tests/workspace_lock.rs` holds 847 lines, over the `file_size_limit` of 800 that `cardinal run
file-size` reads, since the two tests of `apply_the_claim_time_to_the_claims_a_lock_holder_owns_by_pid`. Split it by
responsibility so each part stays under the limit, with no change of behaviour and every `// spec:` citation kept.

# Findings
* `cardinal run file-size` (2026-11-23): `847 crates/cardinal/tests/workspace_lock.rs (was 801)`.
* The file holds the lock tests (WSL-1 to WSL-10) then, from `hook_with_transcript`, the tests of a session continued
  in another (WSL-11, WSL-12, WSL-13), with helpers shared by both (`repo`, `hook`, `lock`, `claim`, `Other`, `live`,
  `dead`).

# Approach
1. Move the transcript hand-over tests (WSL-11 to WSL-13) and their helpers (`hook_with_transcript`, `transcript`,
   `append`, `claim_of`, `issue_claim`) to `tests/workspace_lock_handover.rs`, the shared helpers to a module of
   `tests/common/` or a `tests/workspace_lock/` helper module both files include.
2. A move only, in a deliverable of its own: `deliverable-size` counts a moved line twice.

# Resources
* `crates/cardinal/tests/workspace_lock.rs`, `crates/cardinal/tests/common/mod.rs`
* `specs/workspace-lock.md`, `specs/README.md` (placement of `// spec:`)

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
Split `tests/workspace_lock.rs` (847 lines) so every part stays under 800, by moving code only: no test changes its
assertions, every `// spec:` citation stays on its test. Out of scope: any change of the hooks or of the specs.

## Plan
1. [x] Shared helpers: move `git`, `Repo`, `repo`, `Out`, `spawn`, `finish`, `hook`, `lock_path`, `lock`, `written_at`,
   `dirty`, `claim`, `claim_as`, `claim_since`, `live`, `Other`, `dead` to `tests/workspace_lock_common/mod.rs`
   (`pub`, `#![allow(dead_code)]`), included by `mod workspace_lock_common;`; verified by `just check` (same tests
   pass, spec traceability unchanged); ~390 lines counted. (autonomous)
2. [x] Hand-over tests: move WSL-11 to WSL-13 and their helpers (`hook_with_transcript`, `transcript`, `append`,
   `claim_of`, `issue_claim`) to `tests/workspace_lock_handover.rs`; verified by `just check` and `cardinal run
   file-size` no longer listing the file; ~730 lines counted. (autonomous)
3. [x] Docs and high review of the issue: `update-docs` on `base..HEAD`; verified by its grep and the `high` review.
   (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 813cb803a42a850fc1dd9d98bdacdd8c3f609981
- Helpers module: `tests/workspace_lock_common/mod.rs` (answer 1 a).
- Two deliverables rather than one: the whole move counts ~1,100 lines, over the ~1,000 of a deliverable.
- `tests/workspace_lock_common/mod.rs` rather than `tests/common/`: `common` is compiled into 16 other test crates
  that need none of these helpers.

## Deliverable 1
- done: the shared helpers moved verbatim to `tests/workspace_lock_common/mod.rs` (`pub`, `#![allow(dead_code)]`);
  `workspace_lock.rs` goes from 847 to 664 lines, its 26 tests unchanged.
- red: none (a move, no new test)
- Code review: no findings
- checks: `just check` green (26 tests of `workspace_lock` pass, as before); not verified: nothing.

## Deliverable 2
- done: the WSL-11 to WSL-13 tests and their helpers moved verbatim to `tests/workspace_lock_handover.rs` (369
  lines, 10 tests); `workspace_lock.rs` keeps 16 tests in 304 lines; the `// spec:` citations are the same.
- red: none (a move, no new test)
- Code review: no findings
- checks: `just check` green (16 + 10 tests pass, 26 as before); `file-size` no longer lists the file; not verified: nothing.

## Deliverable 3
- done: `update-docs` on `813cb80..HEAD`: no document names the moved files; `.claude/update-docs-map.md` gets the row
  a move of test code lacked.
- red: none (docs only)
- Code review (`high`, whole issue): 2 findings, both pre-existing in the moved helpers, not fixed here (a move only):
  `claim_as` writes its blob under `<dir>/.git/`, which panics for a linked worktree (every call passes `r.main`
  today); `dead()` returns a reaped pid the kernel may reuse (unlikely: Linux allocates pids sequentially).
  Left to the user whether to file them as issues.
- Code review: no findings (`low`, this deliverable)
- checks: `just check` green; not verified: nothing.
