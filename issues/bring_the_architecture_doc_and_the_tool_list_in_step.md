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

# Milestone
Specs and docs true to the code

# Resources
* `skill:update-docs`, `docs/adr/0028-a-consumer-needs-only-cardinal.md`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
Make `docs/architecture.md` (key table, worktree command names, steering UI section), the `about` lines of
`commit-check` and `issue-claim`, the refusal of `worktree close` from inside the worktree, and `CLAUDE.md` name what
the code reads and accepts. No spec change: PAR-11 asks only that the message name the main worktree.

## Plan
1. [x] Docs and messages in step: the key table (`roadmap_archive`, the tools reading `roadmap`, `architecture`,
   `user_language`), `cardinal run worktree open|close` in the architecture doc, `charts.rs`, the cut edge and the
   audit and cut sessions in the steering UI section, the message of `worktree.rs` naming `cardinal run worktree
   close`, the `about` lines with `[<rev>]` and `--release <stem>`, `update` and `upgrade` in `CLAUDE.md`; verified by
   the PAR-11 test asserting the command named, `project-check`; makes true PAR-11 (message). (autonomous)
2. [x] Fixes of the high review: the refusal names the target (`cardinal run worktree close <arg>`) and the test
   asserts it; `release` and `review-gate` among the readers of `roadmap`; the diagram reads `roadmap --open --json`;
   verified by the PAR-11 test, `project-check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 96227c96eab4255f33089de4327f295d9d37d48b
- 1 a: `CLAUDE.md:24` keeps `just open-worktree`, the recipe still exists.

## Deliverable 1
done: the key table, the worktree commands, the steering UI section, the refusal of `worktree close`, the `about`
lines and the subcommands of `CLAUDE.md` name what the code reads and accepts.

| check | result |
|---|---|
| tests | pass (`project-check`) |
| lint | pass |
| specs | pass, no change |
| docs | `update-docs`: `docs/architecture.md`, `CLAUDE.md`; `docs/usage.md:465` keeps the recipe `close-worktree`, which exists |
| code review | low, see below |

red: PAR-11: `FAIL - close refuses from the worktree to remove and names the main worktree and the command`

Code review: no findings

not verified: none. decisions needed: none.

## Deliverable 2
done: the four findings of the `high` review of `96227c9..HEAD` fixed (all nits or pre-existing).

| check | result |
|---|---|
| tests | pass (`project-check`) |
| lint | pass |
| specs | pass, no change |
| docs | `docs/architecture.md` |
| code review | low, see below |

red: PAR-11: not observed, the assertion and the message changed in the same edit

Code review: no findings

not verified: the red of PAR-11 (see above). decisions needed: none.
