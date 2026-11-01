---
priority: medium
why: 'the commit, issue and spec features each own recipes of the one `cardinal.just`, which a feature cannot leave out'
---
# Split the managed recipes by feature

# Goal
`cardinal.just` is one managed file holding the recipes of every feature (`check-commit`, `next-issue`,
`check-specs`…). A feature can leave out whole files only (ADR 0025 rejected marked blocks), so the recipes of a
feature turned off stay. The outcome: the recipes of each feature in a file of their own, imported so that a missing
one is no error, and `cardinal.just` keeping the recipes of the core.

# Findings
* `cardinal.just` holds `waive-review`, `check-commit`, `check-commits`, `close-issue`, `next-issue`,
  `open-worktree`, `close-worktree`, `release`, `check-issue`, `check-specs`, `spec-coverage`, `check-cardinal`.
* The seeded `justfile` imports `cardinal.just`; the installed `just` is 1.58.

# Approach
1. Check that `just` has an optional import (`import?`) and how a seeded `justfile` of an older project picks the
   new files up (a migration of its seeded text, ADR 0017, or `cardinal.just` importing the others itself).
2. Move the recipes into one file per feature, `cardinal.just` importing each optionally.
3. A raised format if an older `cardinal` would misread the layout (ADR 0014).

# Milestone
Optional features

# Resources
* `files/cardinal.just`, `seed/justfile`, ADR 0014, ADR 0017, ADR 0025.

# Hints
* `cardinal.just` importing the others keeps the seeded `justfile` untouched: no migration of a project-owned file.
