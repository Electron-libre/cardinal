---
priority: medium
why: 'the rule breached most often on the last groups, with false positives that blur it'
---
# Find why commits skip the commit message skill

# Goal
`skill-routing-commit-message` (BRE-16) counts a `git commit` made before `skill:commit-message` was loaded in the
session. Its breached share swings from 4% to 43% between groups and stayed at 40% and 42% on the last two complete
groups, so the skill is often skipped, or the rule counts commits it should not. The outcome: the commits the rule
should not count are left out, and the remaining skips have a known cause and a fix.

# Findings
* Measured 2026-11-03 over 525 sessions: 145 breaches and 40 blocked commits in 44 sessions, spread over this
  repository and three consumer projects.
* Breached share by group (last five complete): 10%, 43%, 4%, 40%, 42%; a rule audit was never run on it.
* Commands of the breaches, by kind:
  * harness update commits (`chore(harness): update the shared harness to …`, `chore(cardinal): update Cardinal to …`);
  * the close of an issue or a review line appended to the issue then committed in the same command;
  * commits in a throwaway repository (`mktemp -d`, `git init`, `chore: init`, `chore: adopt cardinal`): a smoke
    test, not a commit of the project;
  * ordinary deliverable commits with a message written inline (`msg='…'`, `-F -`).

# Approach
1. Classify all the breaches of the last groups by the kinds above, with counts.
2. Leave out of the rule the commits outside the project's repository (a `git init` or a `cd` to another directory in
   the same command), and amend BRE-16.
3. For the kinds that remain, run `skill:audit-instructions` with the counts: candidates are a line in
   `skill:issue-workflow` at the close of an issue, a line in the update recipe or `cardinal update` output, or the
   commit gate hook refusing a commit with no checked message.

# Specs
* `specs/rule-breaches.md` BRE-15, BRE-16.

# Milestone
Lean agent loop

# Resources
* `specs/rule-breaches.md`, `files/.claude/skills/commit-message/SKILL.md`, `files/.claude/skills/issue-workflow/SKILL.md`.
* `files/.claude/skills/audit-instructions/SKILL.md`.

# Hints
* A `blocked` commit (its result an error) was refused by a hook: it is a breach the gate already caught.
