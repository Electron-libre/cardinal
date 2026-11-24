---
priority: low
why: 'docs only: they mislead who changes a key or a tool, but no behaviour is wrong'
---
# Bring the architecture doc and the tool list in step

# Goal
Several documents no longer match the code: the table of the configuration keys, the steering UI section, the
names of the worktree commands, the one-line descriptions `cardinal run` prints and the subcommands `CLAUDE.md`
lists. The outcome: each names what the code reads and accepts.

# Findings
* Code audit of 2026-11-24.
* `docs/architecture.md:57-61`: the table of keys by reader omits `roadmap_archive` (read by
  `crates/cardinal/src/tools/release.rs:278`), and lists `roadmap`, `architecture` and `user_language` under the
  skills only, while `next_issue.rs:314`, `roadmap_state.rs:119`, `close_issue.rs:419`, `delivered_quality.rs:439,443`
  and `language.rs:94` (under `crates/cardinal/src/tools/`) read them.
* `docs/architecture.md:208` names the recipe `close-worktree` and `:197,218,220` the recipe `open-worktree`, where
  the tool is `cardinal run worktree close|open`; `crates/cardinal/src/tools/worktree.rs:303` prints "run
  close-worktree from the main worktree" (ADR 0028: a consumer needs only `cardinal`), while `next_issue.rs:777`
  prints `cardinal run worktree close`.
* `docs/architecture.md:257-304`: no row for `crates/cardinal-ui/src/charts.rs`, no edge for the cut of a milestone
  (`crates/cardinal-ui/src/cut.rs:26`), and the prose at `:253-254` names neither the audit nor the cut sessions.
* `crates/cardinal/src/tools/mod.rs:96-99,126-129`: the `about` lines of `commit-check` and `issue-claim` omit the
  `<rev>` argument (CMC-2) and `--release <stem>` (CLM-10).
* `CLAUDE.md:17` lists `init`, `sync`, `check`, `run`, not `update` and `upgrade` (`crates/cardinal/src/main.rs`).

# Approach
1. Fix the key table, the worktree command names and the steering UI section of `docs/architecture.md`.
2. Change the message of `worktree.rs:303` to name `cardinal run worktree close`, and the two `about` lines.
3. Add `update` and `upgrade` to `CLAUDE.md`.

# Resources
* `skill:update-docs`, `docs/adr/0028-a-consumer-needs-only-cardinal.md`.
