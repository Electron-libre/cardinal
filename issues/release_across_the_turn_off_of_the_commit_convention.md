---
priority: low
depends_on: [release_a_project_without_the_commit_convention]
why: 'only the one release spanning the commit that turns `conventional-commits` off is affected, and the user can still pass a higher version'
---
# Release across the turn off of the commit convention

# Goal
`release` reads whether `conventional-commits` is off from the current `.claude/cardinal.toml` and applies it to
every commit since the last tag (REL-11). The release that spans the commit turning the feature off reads a
`feat!: drop the old API` written under the convention as a plain subject: it proposes a patch and lists it under
`### Changed`. The outcome: each commit is read with the convention its project followed when it was written.

# Findings
* Found by the `high` review of `release_a_project_without_the_commit_convention`.
* `rule-breaches` already reads the configuration as of each commit for rule `commit-format` (FEA-22).
* With the feature off and no `release_breaking_files`, every release is proposed as a patch; the warning of
  REL-11 names the key without saying the project sets none.

# Approach
1. Read `.claude/cardinal.toml` as of each commit (`git show <commit>:.claude/cardinal.toml`), as FEA-22 does, and
   classify it with the convention of that revision.
2. Add to the warning of REL-11, when `release_breaking_files` is empty, that every release is proposed as a patch.

# Specs
* `specs/release.md`: amends REL-11.

# Milestone
Optional features

# Resources
* `crates/cardinal/src/tools/release.rs` (`commits`, `classify`), `crates/cardinal/src/tools/rule_breaches.rs`
  (FEA-22), `specs/release.md`, `specs/features.md`.
