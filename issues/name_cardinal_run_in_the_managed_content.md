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
