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
  [measure_the_effect_of_the_language_hint](measure_the_effect_of_the_language_hint.md).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
Find from the transcripts why the two rules are breached, then change the routing or the wording so the next
evaluation can tell (`Expected-effect`). Out of scope: changing what BRE-10 or BRE-18 count, which the measure
found right.

## Measure (2026-11-12)
`cardinal run rule-breaches` (detail), every transcript kept on the machine.

* `skill-routing-write-spec`: 58 breaches, 58 sessions, 5 projects. In all 58 the spec was written inside
  `issue-workflow` (loaded before the `Write`), and `write-spec` was loaded neither before nor after; none read
  `specs/README.md` or another spec before the `Write`. BRE-18 counts right: the skill was never in context.
  The cause: step 1 restates what the agent needs of the skill (ids, `deterministic`, `draft`, the marker) and says
  "run" where step 3.5 says "**invoke ... with the Skill tool**", the one routing followed in every session.
* `user-language`: 138 breaches; 119 `intermediate`, 17 `hinted`, 2 `final`. The final reports hold; the notes
  between two tool calls slip (96 after a `Bash` result, 19 right after a `Skill` load). The last skill loaded
  before the breach: `code-review` 77, `issue-workflow` 24, `commit-message` 8, others 10, so the notes of the
  review loop and of the fixes that follow it, in an English context, with the rule read once at the start. The
  hook cannot judge them (LNG-7 to LNG-10 withdrawn).

## Plan
1. [x] Route the spec: step 1 of `issue-workflow` says to invoke `skill:write-spec` with the Skill tool before the
   first `Write` under `specs/`, and stops restating its format; `Expected-effect: breached_share:skill-routing-write-spec down`;
   verified by `just check` (managed skill tests) and `cardinal check`. (autonomous)
2. [x] Word the language rule for the notes: `files/.claude/rules/user-language.md` names every text the user sees,
   the short notes between tool calls included, and step 3.5 of `issue-workflow` recalls it for the review loop
   (wording checked with `skill:audit-instructions`); `Expected-effect: breached_share:user-language down`; verified by `just check`. (autonomous)
3. [x] Settle the `high` review of the issue; verified by `just check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: c0310a79d4b99e2e01a3ed942304929e72e63754
- Routing of the spec: wording of step 1 (1 a); a hook only if the next evaluation shows no effect.
- Language: reword the rule and recall it at step 3.5 (2 a); BRE-10 unchanged.
- No recall of the language at step 3.5: FEA-29 keeps the language said once, in the rule; deliverable 2 rewords
  the rule only.

## Deliverable 1
- done: step 1 of `issue-workflow` invokes `skill:write-spec` with the Skill tool before the first write under
  `specs/`, and no longer restates its format.

| check | result |
|---|---|
| `just check` | pass |
| docs (`update-docs`) | n/a: `docs/workflow.md` still true |
| code review | clean |

red: none (a required passage of `tests/skills.rs`: `missing '**invoke `skill:write-spec` with the Skill tool**'`)
Code review: no findings

## Deliverable 2
- done: the rule names every text the user sees, the notes between two tool calls included, whatever the language
  of the skill, review or tool output around them. `skill:audit-instructions` classes a breached rule a hook cannot
  see as a rewording.

| check | result |
|---|---|
| `just check` | pass |
| docs (`update-docs`) | n/a: `docs/` cites the rule, not its wording |
| code review | clean |

red: none (`tests/skills.rs`: `the rule does not name the notes between two tool calls`)
Code review: no findings

## Deliverable 3
- done: the findings of the `high` review settled: `write-spec` ends after its step 4 when `issue-workflow` calls
  it; step 1 names that step; the rule keeps repository text in the project's language when shown; the language
  test merged into FEA-29's; the required passage no longer pins the bold.
- not fixed: BRE-18 counts a new spec only, so the effect on amendments goes unmeasured; by design of the rule,
  the outcome named in the issue is the new specs.

| check | result |
|---|---|
| `just check` | pass |
| docs (`update-docs`, whole issue) | n/a: `docs/workflow.md`, `README.md`, `AGENTS.md` still true |
| code review | clean |

red: none (review fixes)
Code review: no findings
