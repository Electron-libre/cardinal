---
priority: low
why: 'a step of ADR 0028 (proposed): the waiver is the only recipe that is not a call of `cardinal`'
---
# Waive the review through cardinal

# Goal
The review waiver is the shell recipe `waive-review` of `cardinal-review.just`, the only recipe that does not call
`cardinal`. Give the user `cardinal run waive-review`, kept from the agent like the recipe, so that a consumer
without `just` can still waive a review (ADR 0028).

# Findings
* 2026-11-03, `cardinal-review.just`: `waive-review` creates `claude-review-waived` in the git directory and prints
  its path.
* The agent is kept from it by the deny rules `Bash(just waive-review)` and `Bash(just waive-review *)`
  (REV-19, `settings.rs` `DENY_RULES`, the `review-gate` entries of `features.toml`).
* `rule-breaches` finds `review-waiver` on the words `just` and `waive-review` (BRE-6).

# Approach
1. A tool `waive-review` doing what the recipe does; the recipe becomes `cardinal run waive-review`.
2. Deny rules `Bash(cardinal run waive-review)` and `Bash(cardinal run waive-review *)` next to the `just` ones, in
   `entries/.claude/settings.json`, `features.toml` and `DENY_RULES`.
3. `rule-breaches` finds both forms.

# Specs
* `specs/review-gate.md` REV-19 and `specs/rule-breaches.md` BRE-6 amended; the tool's requirement in
  `specs/review-gate.md`.

# Resources
* [ADR 0028](../../docs/adr/0028-a-consumer-needs-only-cardinal.md), ADR 0005 (the gates catch mistakes, not bypasses).
* [add_a_just_recipe_to_waive_the_review](add_a_just_recipe_to_waive_the_review.md): why the waiver is a command.
* `cardinal-review.just`, `crates/cardinal/src/settings.rs`, `crates/cardinal/src/tools/rule_breaches.rs`.

# Hints
* A deny rule matches the command text: without its own rule, the agent can run the new command, which the managed
  allow rule `Bash(cardinal run *)` approves with no prompt; declare both the bare and the `*` form.
* With the feature `review-gate` off, the tool must refuse or be absent, as its deny rules are not synced.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-03

## Understanding
Give the user `cardinal run waive-review` (the recipe becomes its alias), denied to the agent like the recipe, refused
when `review-gate` is off, and found by `rule-breaches`. Out of scope: renaming `just waive-review` to the new form in
the skills and docs (`name_cardinal_run_in_the_managed_content`), dropping the `just` deny rules.

## Plan
1. [x] Tool `waive-review`, recipe alias, deny rules, rule: the tool creates and prints the waiver of the worktree, refuses
   with `review-gate` off; `cardinal-review.just` runs it; the two `cardinal run` deny rules in
   `entries/.claude/settings.json`, `features.toml`, `DENY_RULES`; `rule-breaches` finds both forms; verified by the
   tests of `tests/review_gate.rs` (tool through the binary), `settings_warnings_cli.rs`, `skills.rs` (FEA-23) and
   `rule_breaches.rs`; makes true REV-17, REV-19, REV-22, BRE-6 (~200 lines). (autonomous)
2. [x] Fixes of the `high` review: an empty waiver (a spent stamp kept by `touch` made `native` drop the new one), the
   `cardinal run|tool waive-review` form read in a row in one command, the waiver path of `review_gate.rs` reused, its
   module doc; verified by `tests/review_gate.rs` and `tests/rule_breaches.rs`; makes true REV-17, BRE-6. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 9385c1d9ecb0e0df4ef303d483d51d827b94a00e
- No `FORMAT` raise: the deny rules are added, nothing is replaced, and `cardinal run` runs the tool of the pinned revision.

## Deliverable 1

done: `cardinal run waive-review` creates and prints the waiver, refuses with `review-gate` off; the recipe is its
alias; two `cardinal run` deny rules; `rule-breaches` finds both forms; gates, usage and architecture docs updated.

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | pass |
| specs | pass |
| docs | updated (`docs/gates.md`, `docs/usage.md`, `docs/architecture.md`) |
| code review | done (`low`) |

red: REV-17, REV-22, BRE-6: `FAIL - the tool runs` (unknown tool `waive-review`), `the_waiver_is_refused_with_the_review_gate_off` exit 3 not 1, `FAIL - finds a proj r1 review-waiver blocked a.jsonl:16`

Code review: no findings

not verified: the deny rules in a live Claude Code session.

## Deliverable 2

done: the fixes of the `high` review of the issue (findings 1, 2, 5, 9).

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | pass |
| specs | pass |
| docs | REV-17, BRE-6 and a non-goal of `specs/review-gate.md` amended |
| code review | done (`low`) |

red: REV-17, BRE-6: `FAIL - it empties a spent waiver already there`, `FAIL - no finding beyond the expected ones` (the line `cardinal run check-specs && grep -rn waive-review docs` counted)

Code review: no findings

High review: 9 findings
- 1 (a spent stamp kept), 2 (false `review-waiver` findings), 5 (waiver path duplicated), 9 (module doc): fixed.
- 3 (`cardinal tool waive-review` not denied): not fixed, ADR 0005 (the agent writes `cardinal run`); a non-goal of the spec, and `rule-breaches` finds it.
- 4 (refuse on the deny rules present, the root of the config): not fixed, ADR 0005; the root is the one `project-check` reads.
- 6 (deny rules in four places): not fixed, the existing design; each copy is checked by a test (REV-19, FEA-23).
- 7 (the recipe not run): not fixed, running it needs a `cardinal` that pins the revision under test; the recipe is one line.
- 8 (two `git` processes): not fixed, negligible for a command the user runs by hand.

not verified: none.
