---
priority: low
why: 'only an issue checked in `issues/done/` is affected, and `check-issue` runs mostly on open issues'
---
# Find the project root of an issue checked in issues/done

# Goal
`check-issue` takes the project root as the grandparent of the issue file. For `issues/done/foo.md` that is
`issues/`, so `features::is_off` finds no configuration and an attribution line is refused even when the project
turned off `no-ai-attribution` (FEA-9). The outcome: the root found wherever the issue file lies.

# Findings
* `crates/cardinal/src/tools/check_issue.rs`, the `root` computed with `path.parent().and_then(Path::parent)`.
* Found by the `low` review of `turn_off_conventional_commits`.

# Approach
1. Walk up from the issue file to the directory holding `.claude/cardinal.toml`.
2. Or use `repo_root()`, as `commit-check` does.

# Specs
* `specs/features.md` FEA-9.

# Milestone
Optional features

# Resources
* `crates/cardinal/src/tools/check_issue.rs`, `crates/cardinal/src/features.rs` (`is_off`).
