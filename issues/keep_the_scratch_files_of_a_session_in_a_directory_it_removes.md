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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-19

## Understanding
Each session gets `$TMPDIR=<tmp>/claude-scratch-<session id>/` (a `SessionStart` hook through `CLAUDE_ENV_FILE`,
verified in the Claude Code binary), removed at `SessionEnd`, stale ones swept at start, kept on a `cardinal-keep`
file; a rule line asks for `$TMPDIR`; `rule-breaches` measures the literal `/tmp/<name>` writes. Out of scope: the
hint hook (approach 3, second half), until the measure shows the rule is not enough.

## Plan
1. [x] Measure: rule `scratch-outside-tmpdir` of `rule-breaches`, in a module of its own (`rule_breaches.rs` is over
   the file size limit), its tests in a new test file; verified by tests on fake transcripts; makes true BRE-24. (autonomous)
2. [x] Scratch directory: tool `cardinal run session-scratch start|end` (module `session_scratch.rs`), its two hook
   entries, the rule `files/.claude/rules/scratch-files.md`, the feature `session-scratch` of `features.toml`, docs;
   verified by tests through the binary (fake `CLAUDE_ENV_FILE`, `TMPDIR` in a tempdir) and the sync of this repo;
   makes true SCR-1 to SCR-10. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: f5cdc654c0e18760a88a3cf452f65f4e3251541c
- Keep: a file `cardinal-keep` in `$TMPDIR` keeps the directory (1 a).
- Sweep: a dead process and nothing changed for 24 hours (2 a).

## Deliverable 1
done: rule `scratch-outside-tmpdir` of `rule-breaches` (`tools/scratch_writes.rs`, `shell::commands_with_redirections`).
On the transcripts of the machine: 521 of the 713 sessions with a `Bash` call breach it, 2,973 findings.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (BRE-24 traced, spec still `draft` until SCR) |
| docs | `docs/evaluation.md` |
| code review | no findings |

red: BRE-24: `left: []` against the eleven expected findings (no rule).
Code review: no findings
not verified: none
decisions needed: none
file size: `rule_breaches.rs` (+25) and its test file (+3) stay over the limit: the rule lives in a module of its own,
only its dispatch and the expected summary rows grew.

## Deliverable 2
done: tool `cardinal run session-scratch start|end`, its two hook entries, the rule `.claude/rules/scratch-files.md`,
the feature `session-scratch`; specs `session-scratch` and `rule-breaches` accepted; `cardinal` installed here.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (SCR-1 to SCR-10 traced) |
| docs | `README.md`, `docs/architecture.md`, `docs/usage.md` |
| code review | 1 finding, fixed |

red: SCR-H1, SCR-H2, SCR-1 to SCR-10: `unknown tool session-scratch` (exit 3), and no entry, rule nor feature.
Code review: 1 finding: with no `CLAUDE_PID`, `start` recorded no process, so a later sweep could remove the
directory of a live session left unchanged for a day; fixed: `start` then creates nothing (SCR-4 amended, tested).
not verified: a live session: the next one started here gets its `TMPDIR` (the hooks run the installed `cardinal`).
decisions needed: none
