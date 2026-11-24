---
priority: high
why: 'close-issue takes a broken configuration for an empty one, against what the other tools report; every new key copies a reader'
---
# Read the project table through one reader

# Goal
The `[project]` table of `.claude/cardinal.toml` is parsed by hand in several tools, each with its own policy for a
missing or invalid file: some report the error, some take a broken file for a missing one in silence. The outcome is
one typed reader of the table (string, list, integer) with a single error policy, used by every tool, so a broken
configuration is reported the same way everywhere.

# Findings
* Code audit of 2026-11-24.
* `crates/cardinal/src/tools/close_issue.rs:26`: a private `config_list` shadows the shared one of
  `crates/cardinal/src/tools/mod.rs:353`; it returns an empty list on an unreadable or invalid file, where the shared
  one returns `Err`. `close-issue` then ignores a broken `skipped_dirs` (`close_issue.rs:51`) or `roadmap`
  (`close_issue.rs:419`).
* Other hand-written readers of the table: `crates/cardinal/src/tools/git_diff.rs:30` (`limit`),
  `crates/cardinal/src/tools/check_specs.rs:758` (`config_names`, with a fallback to a top-level key),
  `crates/cardinal/src/tools/delivered_quality.rs:433` (`DocFiles::read`),
  `crates/cardinal/src/tools/rule_breaches.rs:739` (`commit_scopes`), besides `project_value`/`project_str`
  (`crates/cardinal/src/tools/mod.rs:337`) and `Declared::of` (`mod.rs:280`).
* `crates/cardinal/src/tools/release.rs:138` comments: "The other readers of the configuration take a broken one
  for a missing one: a program would believe it (REL-16)".

# Approach
1. List each reader and the behaviour its spec requires for a missing file, an invalid file and a missing key.
2. Add one accessor over the `[project]` table (e.g. a `Config` with `str`, `list`, `int` getters) next to
   `crate::read_config` (`crates/cardinal/src/lib.rs:325`), with the error policy the specs agree on.
3. Route the readers above through it, deleting the private copies; keep the top-level fallback only where a spec
   requires it.

# Resources
* `crates/cardinal/src/tools/mod.rs`, `crates/cardinal/src/lib.rs:325`, `specs/release.md` (REL-16),
  `specs/file-size.md` (SIZ-8, SIZ-10), `specs/close-issue.md`.

# Hints
* A test of each tool on an invalid `cardinal.toml` proves the single policy through the binary.
