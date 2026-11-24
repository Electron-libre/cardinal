---
priority: medium
why: 'refusals a spec states have no test of their own, and three test files are invisible to the spec tracing'
---
# Prove the refusals of release and rebase-issue

# Goal
`cardinal run spec-coverage` lists test files of `release` and `rebase-issue` that trace no requirement although
they prove one, and several refusals of `specs/release.md` and `specs/rebase-issue.md` have no test of their own. The
outcome: those tests traced, and one test per untested refusal.

# Findings
* Code audit of 2026-11-24; `cardinal run spec-coverage` lists `crates/cardinal/src/tools/release.rs`,
  `crates/cardinal/src/tools/rebase_issue.rs`, `crates/cardinal/tests/settings_hygiene.rs`.
* `crates/cardinal/src/tools/release.rs:579` (`the_seeded_changelog_is_the_header`) proves the last sentence of
  REL-4 untraced.
* `crates/cardinal/src/tools/rebase_issue.rs:262,268,274` (tests of `settle`) prove RBI-2 and RBI-3 untraced.
* `crates/cardinal/tests/release.rs:147`: the only existing-tag case of REL-3 (`v0.2.0`, `0.2.0`) is also below the
  proposed version, so the too-low check refuses it even without the tag check
  (`crates/cardinal/src/tools/release.rs:115-127`).
* `crates/cardinal/tests/release.rs:259-271` (REL-6) asserts only that stdout holds `--tag`, not the next steps.
* `crates/cardinal/tests/release.rs:233-257` (REL-5) runs from the root only, so a version command run from the
  current directory would pass.
* `crates/cardinal/src/tools/rebase_issue.rs:63-75,215-227`: the RBI-4 refusals of a bare repository, a detached main
  worktree, a branch that is the trunk and a modified tracked file have no test in
  `crates/cardinal/tests/rebase_issue.rs`; neither has the RBI-3 refusal of a binary file changed on both sides.

# Approach
1. Add the `// spec:` tags to the three unit tests.
2. REL-3: tag `v0.3.0` on a side branch and assert `release 0.3.0` is refused, the changelog unchanged.
3. REL-5: run from a subdirectory; REL-6: assert the full `release <version> --tag` line and the reread/commit
   steps.
4. One RBI-4 test per refusal listed above, one RBI-3 test of a binary conflict.

# Specs
* `specs/release.md`: REL-3, REL-4, REL-5, REL-6. `specs/rebase-issue.md`: RBI-2, RBI-3, RBI-4.

# Resources
* `crates/cardinal/tests/release.rs`, `crates/cardinal/tests/rebase_issue.rs`.

# Hints
* `settings_hygiene.rs` checks the repository itself, not a requirement: leave it untraced.
