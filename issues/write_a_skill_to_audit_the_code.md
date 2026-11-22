---
priority: medium
why: 'no skill reviews the repository as a whole; the UI command of the audit needs it first'
---
# Write a skill to audit the code

# Goal
No skill reviews a repository at a high level: `code-review` reads a diff, `audit-instructions` the instructions of
the agent. The expected outcome is a shared skill that audits the code of a project (architecture, maintainability,
debt, tests, consistency with its specs and ADRs) and turns its findings into issues the user validates.

# Findings
* Skills in `files/.claude/skills/`: `audit-instructions`, `commit-message`, `create-issue`, `evaluate-harness`,
  `issue-workflow`, `prioritize-issues`, `release`, `update-docs`, `write-spec`; none audits the code.
* `skill:create-issue` writes and validates an issue; `skill:prioritize-issues` orders them.

# Approach
1. Define the axes (architecture against `architecture` of `.claude/cardinal.toml`, size and duplication, dead code,
   test coverage of the specs, drift from the ADRs) and what counts as a finding (evidence: file and line).
2. Delegate the reading to subagents per axis to keep the context small, then rank the findings.
3. Propose the findings as a table for validation; write the validated ones with `skill:create-issue`.
4. Keep it generic: project values through `.claude/cardinal.toml`, nothing specific to one project.

# Milestone
A steering UI to command

# Resources
* `files/.claude/skills/audit-instructions/SKILL.md` (an audit skill that proposes for validation),
  `files/.claude/skills/create-issue/SKILL.md`, `docs/architecture.md`, `docs/adr/`, `features.toml`.

# Hints
* A new skill grows the fixed context of every project (`fixed_context_budget`): keep its description short.
