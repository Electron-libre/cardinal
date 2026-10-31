---
rank: 165
why: 'after the size tool: the second mechanical rule the audit found; measure first, it may not be broken'
---
# Refuse staging every file before a commit

# Goal
`skill:commit-message` tells the agent to add only the files of the deliverable, never `git add -A`, so personal
or unrelated files (`settings.local.json`, another deliverable's work) stay out of the commit. The rule is prose;
a deny rule or a hook could refuse the command, if the agent breaks it often enough to pay for one.

# Findings
* The audit of the shared instructions (`issues/done/audit_the_shared_instructions_for_conflicts_and_stale_guidance.md`,
  row M2) found the ban in four skills, now kept in `commit-message` alone.
* `rule-breaches` has no rule for it: how often the agent runs `git add -A` or `git add .` is not measured.

# Approach
1. Measure first: a rule `stage-everything` in `rule-breaches` (a `Bash` call of `git add` with `-A`, `--all` or
   `.`), read over the kept transcripts.
2. If it is broken: a deny rule in `entries/.claude/settings.json` (`Bash(git add -A)`, `Bash(git add -A *)`, …),
   or a `PreToolUse` refusal when a command chains it (`git add -A && git commit`), which a prefix rule misses.
3. Breached rarely: keep the prose line and close the issue with the measure.

# Specs
* `specs/rule-breaches.md`: a new requirement for the rule of step 1.

# Milestone
Lean agent loop

# Resources
* `files/.claude/skills/commit-message/SKILL.md`, `entries/.claude/settings.json`, `specs/rule-breaches.md`.
* ADR 0005 (the gates catch mistakes, not bypasses), ADR 0013 (sync manages the allow rules too).

# Hints
* `issues/done/hook_code_review_before_commit.md`: the review gate runs before the `git add` of
  `git add -A && git commit`, so a check on the staged files sees nothing yet.
