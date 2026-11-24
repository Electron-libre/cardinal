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

# Milestone
Specs and docs true to the code

# Resources
* `crates/cardinal/tests/release.rs`, `crates/cardinal/tests/rebase_issue.rs`.

# Hints
* `settings_hygiene.rs` checks the repository itself, not a requirement: leave it untraced.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
Tests only: trace the unit tests that already prove REL-4, RBI-2 and RBI-3, and add one behavior test per untested
refusal or step of REL-3, REL-5, REL-6, RBI-3 and RBI-4. No change of the specs nor of the code, unless a new test
finds a gap (then it stops for a decision). `settings_hygiene.rs` stays untraced.

## Plan
1. [x] Release: trace `the_seeded_changelog_is_the_header` (REL-4); REL-3 test of a tag `v0.3.0` on a side branch
   refused with the changelog unchanged; REL-5 run from a subdirectory; REL-6 assert the reread, commit and
   `release 0.3.0 --tag` steps; verified by `crates/cardinal/tests/release.rs`; makes true REL-3, REL-4, REL-5, REL-6
   (~60 lines, the file stays under 800). (autonomous)
2. [ ] Rebase-issue: trace the three `settle` unit tests (RBI-2, RBI-3); one RBI-4 test each for a bare repository,
   a detached main worktree, the branch being the trunk, a modified tracked file; one RBI-3 test of a binary file
   changed on both sides; verified by `crates/cardinal/tests/rebase_issue.rs`; makes true RBI-2, RBI-3, RBI-4
   (~120 lines).

## Decisions
- Mode: autonomous
- Base commit: b2f231e5e6900565ec01ffbf866c95f62919e5c6

## Summary of deliverable 1
done: REL-4 seed test traced; REL-3 existing tag above the proposal, REL-5 run from a subdirectory, REL-6 next steps.

| check | result |
|---|---|
| tests | `cargo test --test release`: 12 passed |
| lint, specs | `cardinal run project-check` |
| docs | none: tests only |
| code review | `/code-review low`: no findings |

red: REL-3, REL-5, REL-6: the tests pass on the existing code; checked by mutation instead: without the tag check
`release 0.3.0` is accepted (release.rs:198), with the command run from the current directory `version.txt` lands in
`sub/` (release.rs:275), with another wording of the next step the REL-6 assertion fails (release.rs:313).

Code review: no findings

not verified: nothing.
