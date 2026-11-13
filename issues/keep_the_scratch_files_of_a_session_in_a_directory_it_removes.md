---
priority: medium
why: 'the scratch files the sessions leave in /tmp filled the quota of the machine and made the tests of Cardinal fail; only one written path in ten is removed'
---
# Keep the scratch files of a session in a directory it removes

# Goal
The agent writes its scratch files (measures, smoke runs, logs, browser profiles, copies of a repository) to
literal `/tmp/<name>` paths and almost never removes them. They pile up across sessions and projects until the
temporary directory is full, and then unrelated commands and tests fail. The outcome: each session writes its
scratch files under a directory of its own that is removed when the session ends, and what a session still leaves
in `/tmp` is measured.

# Findings
* Measured 2026-11-13 on the machine: `/tmp` a 32 GB tmpfs with a quota per user, 25 GB used and the quota full;
  the tests of `run_cli` failed with `Disk quota exceeded` (git clones, the upgrade of the launcher). The largest
  entries: 159 browser profiles of one consumer's smoke runs (11 GB), then measure and spike directories of 0.5 to
  2 GB each (`measure-board`, `fogmeasure`, `fw-measure`, `secbench`...). Removed by hand.
* Over the transcripts of every project of the machine (892 sessions, 73,532 Bash commands, 6 projects): 592
  sessions wrote to a literal `/tmp/<name>` path (through `mktemp`, `mkdir -p`, a redirection, `tee`, `cp`, `-o`,
  `--user-data-dir=`), 940 distinct paths, of which 87 were removed by a later `rm`.
* The shared hooks already run at `SessionStart` and `SessionEnd` (`workspace-lock`, `entries/.claude/settings.json`).

# Approach
1. Measure first: a rule of `rule-breaches` (or a count of `session-metrics`) for the paths a session writes under
   `/tmp` outside its own directory, so the effect shows in `cardinal-report`.
2. A `SessionStart` hook that creates `/tmp/claude-scratch-<session id>/` and exports it as `TMPDIR` to the Bash
   commands of the session (Claude Code's `CLAUDE_ENV_FILE`, to verify), and a `SessionEnd` hook that removes it;
   `mktemp`, `tempfile` and Node's `tmpdir()` then land in it with no change of habit.
3. A line in a rule loaded in every session: scratch files go under `$TMPDIR`, never a literal `/tmp/<name>`; or a
   hint hook on a Bash command that writes to a literal `/tmp/` path, if (1) shows the line is not enough.
4. A feature in `features.toml` to turn it off, and a way to keep the directory of a session on purpose (an
   artefact the user wants to read after the session).

# Specs
* A new spec of the scratch directory of a session (or an amendment of `specs/workspace-lock.md` if it shares the
  hooks of the session).

# Resources
* `entries/.claude/settings.json` (`SessionStart`, `SessionEnd`), `crates/cardinal/src/tools/workspace_lock.rs`.
* `specs/rule-breaches.md`, `specs/read-hint.md` (a hint hook), `features.toml`.
* The Claude Code documentation of the hooks (`CLAUDE_ENV_FILE`, `SessionEnd`).

# Hints
* `SessionEnd` does not run when a session is killed: the `SessionStart` hook may also remove the directories of
  sessions that ended long ago (no lock, older than a day), never one of a session still running.
* A test of Cardinal must not depend on `TMPDIR` pointing outside the repository: `audit_sources` reads the
  `CLAUDE.md` of the parents of its temporary directory.
* Verdict: `Expected-effect:` the new metric down.
