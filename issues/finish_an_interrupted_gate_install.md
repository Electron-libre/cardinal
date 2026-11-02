---
priority: low
why: 'only a sync killed mid-install reaches it, but every later sync then fails until the user merges two identical files'
---
# Finish an interrupted gate install

# Goal
A `sync` killed during the gate install can leave `pre-commit` and `pre-commit.user` as two hard links of the same
hook of the user. Every later `sync` then refuses with `both exist`, and `check` asks for a manual merge of two
identical files. Expected: `sync` sees that both names are the same file and finishes the install.

# Findings
* `crates/cardinal/src/gate.rs`, `swap`: for `State::Foreign` it first runs `link_new(hook, chained)` (a hard link),
  then `rename(staged, hook)`. A kill between the two leaves both names on the same inode. `conflicts` and `plan` take
  that for a conflict (found by the `high` review of `name_the_chained_hook_conflict_in_check`).

# Approach
1. Write a failing test with `pre-commit` and `pre-commit.user` hard-linked, then let `conflicts` treat the same
   inode (`std::os::unix::fs::MetadataExt::ino` and `dev`) as an interrupted move: install the gate over `pre-commit`
   and keep `pre-commit.user`.
2. Optionally, extend the same treatment to identical content.

# Specs
* `specs/sync.md` SYN-19 (amend: a `pre-commit.user` that is the same file as `pre-commit` is not a conflict), SYN-23.

# Resources
* `crates/cardinal/src/gate.rs`, `crates/cardinal/tests/gate_cli.rs`.
