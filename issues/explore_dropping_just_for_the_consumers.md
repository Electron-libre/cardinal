---
priority: low
why: 'an exploration: no consumer is blocked, `just` installs in one command, and the outcome may be to keep it'
---
# Explore dropping just for the consumers

# Goal
A project that adopts Cardinal must install `just` next to `cardinal` (`docs/tutorial.md` step 1): the agent reaches
the project's checks and Cardinal's recipes through it. Find out what it would take for a consumer to need only
`cardinal`, what it would cost, and decide whether to do it: the outcome is a measured proposal (keep `just`, make it
optional, or drop it), and the issues that carry it out if any.

# Findings
* 2026-11-02, `grep` of the managed content: the skills of `files/.claude/skills/` name `just <recipe>` about 26 times
  (`just check`, `check-issue`, `next-issue`, `close-issue`, `check-specs`, `spec-coverage`, `check-cardinal`,
  `open-worktree`, `release`, `check-commit`), plus `files/specs/README.md`.
* Every recipe of `cardinal*.just` calls `cardinal` (`cardinal run <tool>`, `cardinal run worktree open`,
  `cardinal check`), except `waive-review`, a shell script that creates the waiver file.
* `check` is the project's own recipe of the seeded `justfile` (format, lint, tests, then `check-specs`): the one
  entry point the agent runs before each review, and the only recipe with no `cardinal` equivalent.
* The review waiver is kept from the agent by the deny rules `Bash(just waive-review)` and
  `Bash(just waive-review *)` of `entries/.claude/settings.json`; `rule-breaches` detects `just` + `waive-review`
  (`tools/rule_breaches.rs`).
* `delivered-quality` counts a failed check as a `just check*` or `just test*` command (`tools/delivered_quality.rs`).
* `files/.github/workflows/cardinal.yml` installs `just` to run `just check-specs` and `just check-commits`.
* The seeded `.claude/settings.json` allows `Bash(just)` and `Bash(just *)`, and `docs/tutorial.md` tells the project
  to run the commands of its stack through `just`.
* 2026-11-03, inventory of what needs `just` and what it would become (step 1 of the approach):

  | Place | Uses of `just` | Without it |
  |---|---|---|
  | Skills (`files/.claude/skills/`) | 33 mentions, 13 recipes | `cardinal run <tool>`; `check` → the `check_command` of the project |
  | `files/specs/README.md` | 4 | same |
  | `cardinal*.just` (5 files, 12 recipes) | every recipe is `cardinal run …` / `cardinal check`, but `waive-review` | kept as an option, or dropped |
  | `seed/justfile` | `check` (project-owned), `default` | stays a seed the project may delete |
  | `entries/.claude/settings.json`, `features.toml`, `settings.rs` `DENY_RULES` | deny `Bash(just waive-review[ *])`, ask `Bash(just release *.*)` | the same rules on the `cardinal run` form, both kept during the migration |
  | `rule_breaches.rs` | `just` + `waive-review` (`review-waiver`), `just` as launcher of the checker (BRE-16) | read both forms |
  | `delivered_quality.rs` `is_check` (QUA-8) | `just check*` / `just test*` | also the `check_command`; both forms read, no series cut |
  | Messages of the tools | `close_issue.rs`, `next_issue.rs` name `just close-issue`, `just close-worktree` | `cardinal run …` |
  | `files/.github/workflows/cardinal.yml` | installs `just` for `check-specs`, `check-commits` | `cardinal run` (the install step goes) |
  | Docs | `tutorial` 22, `usage` 16, `workflow` 13, `architecture` 7, `README` 5, `gates` 3; specs 36 lines | rewritten; ADR 0001 and 0010 state that a consumer needs `just` |

* Consumers today: this repository and one other project, both with a `justfile`; nobody is blocked.
* Fixed context: `cardinal run <tool>` is 9 bytes longer than `just <tool>`; ~40 mentions add ~0.4 KB to bodies loaded on
  call only (the skill descriptions, the fixed part, name no recipe).
* Estimated cost of making `just` optional: ~600–900 changed lines in 3 issues (skills + messages + CI; `check_command`
  + the readers; the waiver tool + deny rules + migration), a `cardinal-format`/`FORMAT` raise (ADR 0014) with a
  `migrations/<n>.toml` naming the replaced commands (ADR 0017).

# Approach
Measure first, then propose; nothing is decided here.
1. List every place that needs `just` (skills, specs README, settings entries, CI, the readers of the transcripts,
   docs) and what each would become without it.
2. Candidates, which can be combined:
   1. Keep `just` and only document it (done in `docs/tutorial.md`): no cost.
   2. The skills name `cardinal run <tool>` instead of the recipes, and a `[project]` key (`check_command`, as
      `lint_command`) names the project's check; `just` stays an option of the project.
   3. A `cardinal waive-review` (or a tool) with its own deny rule, so the waiver no longer needs a recipe.
3. For each, weigh: the fixed context of the skills (`cardinal run` is longer than `just`), the deny rules and the
   readers of the transcripts that match `just` (a `rule-breaches` or `delivered-quality` series cut in two), the
   migration of the consumers (ADR 0017, `migrations/<n>.toml` for a renamed command), and what a project loses
   (`just --list`, its own recipes).
4. Write the proposal (an ADR draft when it changes a decision) and, on the user's agreement, the issues.

# Resources
* `cardinal.just`, `cardinal-*.just`, `seed/justfile`, `seed/.claude/settings.json`, `entries/.claude/settings.json`.
* `files/.claude/skills/*/SKILL.md`, `files/specs/README.md`, `files/.github/workflows/cardinal.yml`.
* `crates/cardinal/src/tools/rule_breaches.rs`, `crates/cardinal/src/tools/delivered_quality.rs`.
* `docs/tutorial.md` step 1, `docs/usage.md` ("Features a project turns off"), `docs/gates.md` (the review gate).
* ADR 0001 (one harness synced by a Rust tool), ADR 0005 (the gates catch mistakes), ADR 0017 (migrations).
* [add_a_just_recipe_to_waive_the_review](done/add_a_just_recipe_to_waive_the_review.md): why the waiver is a recipe.

# Hints
* A deny rule matches the command text: a new way to waive needs its own rule, or the agent can waive its review.
* The readers of the transcripts compare revisions: changing the command they match cuts a series unless both forms
  are read.

# Proposal

| Candidate | Cost | Fixed context | Deny rules, transcript series | Migration | A project loses |
|---|---|---|---|---|---|
| a. Make `just` optional (**chosen**) | ~600–900 lines, 3 issues | +0.4 KB in bodies loaded on call, descriptions unchanged | rules on both forms; `rule-breaches` and `delivered-quality` read both, no series cut | `FORMAT` raised, `migrations/<n>.toml` names the replaced commands | nothing: `cardinal*.just` and the `justfile` stay for who wants them |
| b. Keep `just`, document it | none | none | none | none | nothing; every consumer installs `just` |
| c. Drop `just` everywhere | a + deleting the recipes | as a | as a | as a, plus the removed recipe files | `just --list`, the recipes it already uses |

Option a, in [ADR 0028](../docs/adr/0028-a-consumer-needs-only-cardinal.md):
- `cardinal run project-check` runs the `check_command` of `[project]` (the project's format, lint and tests), then
  `check-specs` when the feature `specs` is on, and fails when `check_command` is unset: the one entry point, with no
  recipe, named apart from `cardinal check` (the drift check).
- `cardinal run waive-review` creates the waiver; deny rules on `cardinal run waive-review[ *]` next to the `just`
  ones, and `rule-breaches` reads both forms.
- The skills, `specs/README.md`, the tool messages and the CI name `cardinal run <tool>`; `cardinal*.just` stay as
  thin aliases (`commit-check` reads a revision itself for `check-commit`); `delivered-quality` counts
  `cardinal run project-check` as well as `just check*` / `just test*`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-03

## Understanding
Measure what ties a consumer to `just` (done in Findings), weigh the three candidates and write a proposal the user
decides; out of scope: carrying it out here. When the proposal changes ADR 0001/0010, its ADR draft goes with it.

## Plan
1. [x] Proposal: the candidates weighed (cost, context, deny rules, transcript series, migration, what a project loses)
   and the recommendation, in a `## Proposal` of this issue, plus an ADR draft in `docs/adr/` when the choice changes
   a decision; verified by `just check` (the ADR index, the links). (autonomous)
2. [x] The follow-up issues of the chosen option (`skill:create-issue`), with their `priority` and `depends_on`;
   verified by `just check-issue` on each and `just next-issue`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 707e029466af500afec6eb9e901c1633c0c9c8d0
- Option: a, make `just` optional (validated 2026-11-03)

## Summary of deliverable 1
done: the candidates weighed in `# Proposal`, ADR 0028 (Proposed) and its index line.

| check | result |
|---|---|
| `just check` | pass |
| deliverable size | 27 lines, within |
| docs | ADR index updated; no behaviour changed, other docs n/a |
| code review (low) | pass |

red: none (a proposal, no test)
Code review: no findings
not verified: the line estimates of option a (measured by grep, not by a prototype)

## Summary of deliverable 2
done: three issues, `run_the_project_check_through_cardinal`, `waive_the_review_through_cardinal` and
`name_cardinal_run_in_the_managed_content` (the last depends on the first two), all `low`, no milestone; the
private project named in the Findings replaced by a generic mention.

| check | result |
|---|---|
| `just check-issue` | pass on each |
| `just next-issue` | lists the first two, the third blocked by them |
| code review | n/a: a diff under `issues/` only |

red: none (issues only)
Code review: no findings
not verified: nothing
3. [x] Fixes of the `high` review of the issue: `cardinal run project-check` (named apart from `cardinal check`,
   fails with no `check_command`, specs only with the feature on), the waiver deny rules against the allow rule
   `Bash(cardinal run *)`, `check-commit` not a bare alias, a narrower consequence on `delivered-quality`, 0001 and
   0010 partly superseded once accepted; in ADR 0028, the proposal and the follow-up issues. (autonomous)

## Summary of deliverable 3
done: the 7 findings of the `high` review fixed in ADR 0028, `# Proposal` and the three follow-up issues.

| check | result |
|---|---|
| `just check` | pass |
| `just check-issue` | pass on the three issues |
| code review (low) | pass |

red: none (docs only)
Code review: no findings
not verified: nothing
