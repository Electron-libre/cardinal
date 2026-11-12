---
priority: low
why: 'two rules stay breached in most of the sessions they apply to, but they cost less than the waste and the blind measures'
---
# Find why specs and replies escape their rules

# Goal
Two rules of the shared harness stay breached over every group of the last evaluation: a spec written without
`skill:write-spec`, and a reply not in the `user_language`. A rule ignored that often is either misplaced, worded
so the model skips it, or wrong. The outcome: the cause of each is known from the transcripts, and the rule, the
skill description or the routing is changed, or the rule dropped.

# Findings
* Measured 2026-11-12 (`cardinal-report --last 6`, `rule-breaches --summary`), consumer groups `c6fd376`,
  `7e952ec`, `28c8eaa`:
  * `skill-routing-write-spec` (BRE-18): `breached_share` 83% (5/6), 94% (16/17), 83% (10/12); 100% (3/3) in a
    group of this repository.
  * `user-language` (BRE-10): 5% (1/21), 19% (13/68), 17% (24/142), 29 then 53 breaches; this repository 9% and
    20%. The `language-hint` hook is in place in those groups.
  * `skill-routing-commit-message` (BRE-16), for comparison: 38%, 42%, 26%.
* `issue-workflow` step 1 routes a new spec to `skill:write-spec` "up to its format check"
  (`files/.claude/skills/issue-workflow/SKILL.md:38`), so most specs are written inside an issue.

# Approach
Measure first: read the breaching sessions (`cardinal run rule-breaches` without `--summary` gives their lines).
1. For `write-spec`: does the agent write the spec from the issue-workflow step without loading the skill, or load
   it after the `Write`? If the template is all it takes from the skill, inline the template in the step, or move
   the routing to a hook on a `Write` under `specs/`.
2. For `user-language`: tell the breaches the hint caught (`detail` `hinted`, BRE-13) from the others, then see
   which reports slip (after a long tool run, in the final report, after a sub-agent's English result).
3. Hand the counts to `skill:audit-instructions` for the wording, as `skill:evaluate-harness` prescribes.

# Specs
* `specs/rule-breaches.md` only if a rule's definition turns out wrong (e.g. BRE-18 counting a spec the skill
  routed); `specs/language-hint.md` if the hint changes.

# Milestone
Measures

# Resources
* `.claude/skills/evaluate-harness/SKILL.md`, `.claude/skills/audit-instructions/SKILL.md`,
  `files/.claude/skills/write-spec/SKILL.md`, `files/.claude/skills/issue-workflow/SKILL.md`,
  `files/.claude/rules/user-language.md`,
  [measure_the_effect_of_the_language_hint](done/measure_the_effect_of_the_language_hint.md).
