---
priority: low
why: 'each case needs a dangling link or a race, rare, but the install then loses or half-moves a hook of the user'
---
# Keep the gate install unchanged on failure

# Goal
SYN-19 promises the install over a user's `pre-commit` never overwrites it and fails changing nothing. Three paths
of `gate::install` break it: a failure there leaves the staged hook behind, or loses the user's hook. Expected: any
failure of the install leaves the hooks directory as it found it, and the move never replaces a `pre-commit.user`.

# Findings
* Found by the `high` review of `guard_a_dangling_chained_hook_at_the_gate_install`, in
  `crates/cardinal/src/gate.rs`, `install`:
* A dangling user `pre-commit` link is `Foreign`: `install` writes `pre-commit.cardinal-new`, then
  `fs::canonicalize(&hook)` fails with `No such file or directory`, the staged file left in the hooks directory.
* In the symlink branch, `fs::remove_file(&hook)` runs before `symlink(&target, &chained)`: when the link fails, the
  user's link is gone and no gate is installed.
* `fs::rename(&hook, &chained)` replaces a `pre-commit.user` created between `plan` and `apply` without an error.

# Approach
1. Write the failing test of SYN-19 with a dangling user `pre-commit` (refuse it in `plan`, or move the link as a
   link).
2. Create the new link before removing the old one.
3. Refuse at the move rather than only in `plan`: `hard_link` then `remove_file`, or `renameat2(RENAME_NOREPLACE)`;
   remove the staged file on any error.

# Specs
* `specs/sync.md` SYN-19 (scenarios).

# Resources
* `crates/cardinal/src/gate.rs`, `crates/cardinal/tests/gate_cli.rs`.
