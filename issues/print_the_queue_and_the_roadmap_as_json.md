---
priority: high
why: 'the steering UI reads the state through these tools, and their text output is no contract to parse'
---
# Print the queue and the roadmap as JSON

# Goal
The steering UI (milestone "Steering UI") shows the milestones, the queue of issues and the claims. Today the tools
that know them print text for the agent: `cardinal run next-issue` a few lines (`Next:`, `  why:`, `On hold:`),
`cardinal run roadmap` a Markdown table. Parsing that text would tie the UI to a wording that changes with each
revision. The outcome: `next-issue` and `roadmap` take `--json` and print one stable document a program reads, the
text output staying as it is.

# Findings
* `crates/cardinal/src/tools/next_issue.rs` (603 lines) already computes everything the UI needs: the order
  (`order`), the claims (`claims`, `working` or `merge pending`, `session ended`), the branches and worktrees that
  take an issue (`Branch`), the blocked issues, those on hold and the dependency cycles; `offer` turns it into text.
* `crates/cardinal/src/tools/roadmap.rs` prints `| milestone | state | released | issues | document |` from
  `roadmap_state.rs`.
* No tool of `crates/cardinal/src/tools/` prints JSON today.

# Approach
1. Split the computation of `next-issue` from its rendering: one structure (each open issue with its title,
   priority, milestone, dependencies, state `ready`, `taken`, `blocked`, `on hold`, and what takes it), rendered as
   text or JSON.
2. Same for `roadmap`: each milestone with its state, release, open and total issues, document and issue stems.
3. Choose the JSON library (`serde_json` is the obvious candidate; check what the crate already depends on) and
   whether the document carries a `version` field the UI checks.

# Specs
* `specs/next-issue.md` and `specs/roadmap.md`: a requirement each for `--json`, its fields and its stability.

# Milestone
Steering UI

# Resources
* `specs/next-issue.md`, `specs/roadmap.md`, `specs/issue-claim.md`
* `docs/architecture.md`, "Parallel issues"
* ADR 0021 (priority and depends_on), ADR 0022 (claims)

# Hints
* The text output is read by the agent every time it runs: leave it byte for byte unchanged.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
`next-issue --json` and `roadmap --json` print one JSON object on one line for the steering UI, with a `version`
field; the text outputs, the stderr reports and the exit codes stay as they are. Out of scope: the UI itself, a JSON
form of the stderr reports, `worktree` and `cardinal-report`.

## Plan
1. [x] `next-issue --json`: `Entry` keeps the priority, milestone and dependencies it reads, one renderer builds the
   document from the `Queue`; verified by `crates/cardinal/tests/next_issue_json.rs` (`tests/next_issue.rs` is over
   the file size limit); makes true NXT-18, NXT-19. ~350 lines. (autonomous)
2. [x] `roadmap --json`: `rows` returns a milestone structure rendered as the table or as JSON; verified by
   `crates/cardinal/tests/roadmap.rs`; makes true RMP-7, RMP-8. ~200 lines. (autonomous)
3. [x] Fixes of the `high` review of the issue: `path` of each listed issue in `roadmap --json`, `title` and
   unknown `milestone` `null` in `next-issue --json`, unknown options of `next-issue` refused, `--json` in the tool
   help and the recipe comments; verified by `next_issue_json.rs` and `roadmap.rs`; makes true NXT-18, NXT-19, RMP-8. (autonomous)

## Deliverable 1
done: `next-issue --json` prints the queue as one JSON object (version, next, issues with state, hold, taken, stale).

| check | result |
|---|---|
| tests | `next_issue_json` 7/7, `next_issue` and `issue_claim` unchanged and green |
| lint | clean |
| specs | `check-specs` clean for NXT |
| docs | `docs/usage.md`, module doc |
| code review | `low`, clean |

red: NXT-18, NXT-19, NXT-H2: stdout empty, `next-issue` read `--json` as the `issues` directory (`no --json directory`).

Code review: no findings

not verified: none

decisions needed: none

## Deliverable 2
done: `roadmap --json` (alone or with `--open`) prints the milestones as one JSON object (version, milestones with
state, hold, release, document and listed issues).

| check | result |
|---|---|
| tests | `roadmap` 17/17 (4 new) |
| lint | clean |
| specs | `check-specs` clean for RMP, spec `accepted` |
| docs | `docs/usage.md`, module doc |
| code review | `low`, clean |

red: RMP-7, RMP-8, RMP-H2: `roadmap` exited 2 on the unknown argument `--json`.

Code review: no findings

not verified: none

decisions needed: none

## Deliverable 3
done: fixes of the `high` review (7 findings, 5 fixed).

| check | result |
|---|---|
| tests | `next_issue_json` 9/9, `roadmap` 18/18, `next_issue`, `issue_claim`, `skills` green |
| lint | clean |
| specs | `check-specs` clean, NXT-18, NXT-19, RMP-8 amended |
| docs | `docs/usage.md`, tool help, `files/cardinal/issues.just` (synced) |
| code review | `low`, clean |

red: none observed: the tests were written with the fixes, not before them.

Code review: no findings

not verified: none

decisions needed: none

High review findings:
1. fixed: `roadmap --json` gave issues of the same stem in two directories, or not under `issues/` directly, with
   nothing to tell them apart: each issue now carries its `path` (RMP-8).
2. not fixed: `next-issue --json` deletes the `merge pending` claims the trunk closed, as the text output does on
   every run; the pruning is idempotent and keeps the claims true, so the JSON form keeps it.
3. fixed: `next-issue` refuses an unknown option or a second directory (NXT-18).
4. fixed: `title` is `null` for an issue with no heading instead of `(no file)` (NXT-19).
5. fixed: `milestone` is `null` for a milestone NXT-13 reports unknown (NXT-19).
6. fixed: `--json` in the help of both tools and the comments of the `next-issue` and `roadmap` recipes.
7. not fixed: the new test file builds its repositories as `tests/next_issue.rs` and `tests/issue_claim.rs` do,
   without the isolation of `tests/common`; aligning them all is outside this issue.

## Decisions
- Mode: autonomous
- Base commit: 2925de2ffdcda6496a51a9391c526b621740bad3
