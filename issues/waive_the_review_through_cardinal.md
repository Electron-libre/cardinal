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
* [ADR 0028](../docs/adr/0028-a-consumer-needs-only-cardinal.md), ADR 0005 (the gates catch mistakes, not bypasses).
* [add_a_just_recipe_to_waive_the_review](done/add_a_just_recipe_to_waive_the_review.md): why the waiver is a command.
* `cardinal-review.just`, `crates/cardinal/src/settings.rs`, `crates/cardinal/src/tools/rule_breaches.rs`.

# Hints
* A deny rule matches the command text: without its own rule, the agent can run the new command.
