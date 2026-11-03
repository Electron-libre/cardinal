---
priority: low
depends_on: [run_the_project_check_through_cardinal, waive_the_review_through_cardinal]
why: 'the last step of ADR 0028 (proposed): once the check and the waiver have a `cardinal` command, `just` becomes optional'
---
# Name cardinal run in the managed content

# Goal
Once the check and the waiver run through `cardinal`, make `just` an option of the consumer: the skills, the specs
README, the messages of the tools, the CI and the docs name `cardinal run <tool>`, and a consumer installs only
`cardinal` (ADR 0028, then accepted).

# Findings
* 2026-11-03, the inventory of
  [explore_dropping_just_for_the_consumers](done/explore_dropping_just_for_the_consumers.md): 33 mentions of `just` in
  the skills, 4 in `files/specs/README.md`, the messages of `close_issue.rs` and `next_issue.rs`, the two jobs of
  `files/.github/workflows/cardinal.yml` that install `just`, about 66 lines of docs, and ADR 0001 and 0010.

# Approach
1. The managed content and the messages name `cardinal run <tool>` (`cardinal check` for `check-cardinal`); the
   recipes of `cardinal*.just` stay as aliases. `check-commit` pipes `git log -1 --format=%B <rev>` into
   `commit-check`: let `commit-check` read a revision so the skills can name it alone.
2. The CI runs `cardinal run check-specs` and `cardinal run check-commits` and stops installing `just`.
3. `rule-breaches` BRE-16 already reads `run <checker>`: check the other readers match both forms.
4. Raise `cardinal-format` and `FORMAT` with a `migrations/<n>.toml` naming the replaced commands (ADR 0014,
   ADR 0017); `docs/tutorial.md` step 1 makes `just` optional; ADR 0028 accepted, ADR 0001 and 0010 partly
   superseded.
5. Likely two deliverables: the managed content and the tools, then the docs.

# Resources
* [ADR 0028](../docs/adr/0028-a-consumer-needs-only-cardinal.md), ADR 0014, ADR 0017.
* `files/.claude/skills/*/SKILL.md`, `files/specs/README.md`, `files/.github/workflows/cardinal.yml`,
  `docs/tutorial.md`, `docs/usage.md`, `docs/workflow.md`, `docs/architecture.md`.

# Hints
* The skills are read by every session: `cardinal run` is 9 bytes longer than `just`, keep the rest of each line short.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-03

## Understanding
The managed content, the seeds, the tool messages and the CI name `cardinal run <tool>`, so a consumer installs only
`cardinal`; the recipes of `cardinal*.just` stay as aliases. Out of scope: dropping the recipes or the `just` deny
rules. The readers already match both forms (BRE-16 `run`, the waiver, QUA-8 `project-check`): nothing to change there.

## Plan
1. [x] `commit-check [<rev>]` checks the message of a revision, `check-commit` calls it; verified by tests through the
   binary; makes true CMC-1, CMC-2, CMC-3 (spec `accepted`). (autonomous)
2. [ ] The managed content names `cardinal run`: the skills, `files/specs/README.md`, the CI (no `just` install),
   the messages of `close-issue`, `next-issue` and the `CHANGELOG.md` header of `release`, the seeded `ROADMAP.md`,
   `CHANGELOG.md` and `cardinal.toml` comment; `check_command` of this repository; `cardinal-format`/`FORMAT` 9 and
   `migrations/9.toml`; verified by a test that no file of `files/` but the `.just` ones names a `just` recipe, and
   the tests of the messages; makes true BIN-8 (amended). (autonomous)
3. [x] Docs: `docs/tutorial.md` step 1 (`just` optional), `usage`, `workflow`, `architecture`, `gates`, `README.md`,
   the specs that say a consumer needs `just` (BIN intent), ADR 0001 and 0010 partly superseded by 0028; verified by
   `just check` (links, ADR index). (autonomous)

4. [x] Fixes of the `high` review of the issue; verified by `just check` and the tests of CMC-3 (a path, a range),
   the allow rule of `cardinal check` and BIN-8 (a wrapped line, a quiet recipe). (autonomous)
Estimated size: ~150, ~400 (copies included), ~200 lines.

## Decisions
- Mode: autonomous
- Base commit: 8306fd82d108932233e7aa6c167874cf8fc1b9d6
- Format: raised to 9 with `migrations/9.toml` (names `` `just check` ``, rewrites the seeded `ROADMAP.md` and `CHANGELOG.md` lines)
- This repository: `check_command = "just check"` (keeps its `justfile` and `--strict`)

## Summary of deliverable 1
done: `commit-check [<rev>]` checks the message of a revision (else stdin), `check-commit` calls it; spec
`commit-check.md` accepted.

| check | result |
|---|---|
| `just check` | pass |
| deliverable size | 119 lines, within |
| docs | no page names the pipe of `check-commit`; spec added |
| code review (low) | pass |

red: CMC-1, CMC-2, CMC-3: `the message of HEAD, not stdin: left: 0 right: 1`; `left: (1, "format must be …") right: (2, "")`
Code review: no findings
not verified: nothing

## Summary of deliverable 2
done: the skills, `specs/README.md`, the CI (no `just` install), the tool messages and doc comments, the seeds and
the `CHANGELOG.md` header name `cardinal run <tool>`; format 9 with `migrations/9.toml`; this repository and the replay
fixture set `check_command = "just check"`; FEA-32 names `cardinal run check-specs`.

| check | result |
|---|---|
| `just check` | pass |
| `cardinal run project-check` here | pass |
| deliverable size | 267 lines, within |
| docs | specs FEA-32, BIN-8 amended; the docs pages in deliverable 3 |
| code review (low) | see below |

red: BIN-8: `files/.claude/skills/commit-message/SKILL.md names `just check-commit`: name `cardinal run <tool>``

## Summary of deliverable 3
done: the tutorial (step 1: `cardinal` alone, `just` optional), `usage`, `workflow`, `architecture`, `gates`,
`README.md`, `ROADMAP.md` and the specs (BIN intent, the non-goals of `format-hook` and `lint-gate`) name
`cardinal run <tool>`; ADR 0001, 0010 and 0028 were already marked at the acceptance of 0028.

| check | result |
|---|---|
| `just check` | pass |
| docs | this deliverable |
| code review (low) | pass |

red: none (docs only)
Code review: no findings
not verified: nothing

## Summary of deliverable 4
done: the fixes of the `high` review of `8306fd8..af87f40`.

| # | finding | done |
|---|---|---|
| 1 | `commit-check <path>` read as a pathspec | fixed: the revision resolved with `rev-parse --verify` first; a path and a range exit 2 |
| 2 | `cardinal check` has no allow rule | fixed: `Bash(cardinal check)` in the managed allow rules |
| 3 | the migration names `` `just check` `` only | not fixed: the other recipes stay as working aliases, a warning on each would be noise |
| 4 | the BIN-8 test misses a wrapped line, a `@` recipe | fixed |
| 5 | `details.md` names `close-worktree`, "the recipe" of the waiver | fixed |
| 6 | the message of a revision read twice | fixed: `check-commits` calls `message_of` |
| 7 | the CI keys `check-specs` on `cardinal-specs.just` | not fixed: the file belongs to the feature `specs` (FEA-31), present exactly when it is on |
| 8 | the specs checked twice by `project-check` | fixed for the replay (`just fmt clippy test`); kept here, the strict check needs the recipe (decision 2) |
| 9 | the replay `sed` silent or duplicating | fixed: skipped when the key exists, fails when it is still missing |
| 10 | a broken pipe in the test helper | fixed: the write to stdin ignores it |

| check | result |
|---|---|
| `just check` | pass |
| deliverable size | 106 lines, within |
| docs | `details.md` |
| code review (low) | pass |

red: CMC-3: `cardinal run commit-check README.md` exited 0 on the message of HEAD (the `high` review)
Code review: no findings
not verified: the replay recipe (billed)
