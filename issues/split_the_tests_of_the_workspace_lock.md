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
