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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
A shared skill `audit-code` that reviews a whole repository on five axes (architecture against the `architecture`
document and the ADRs, size and duplication, dead code, test coverage of the specs, drift from the specs and ADRs),
reads through one subagent per axis, verifies and ranks the findings, stops on a table, then writes the validated
ones through `skill:create-issue`. Out of scope: a feature to turn it off (FEA non-goal: each feature has its own
issue), a `cardinal run` tool, the UI command (`audit_the_code_from_the_ui`).

## Plan
1. [x] Skill `audit-code`: `files/.claude/skills/audit-code/SKILL.md` (short description, axes, evidence `file:line`,
   one subagent per axis, verify and rank, report then STOP, issues through `skill:create-issue`, project values
   through `.claude/cardinal.toml`), synced copy, README and docs; verified by required passages in
   `crates/cardinal/tests/skills.rs` and the structural skill checks (cited skills, docs) and `fixed-context` under
   `fixed_context_budget`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 18ad8c68562b4501b6802e9a2e5f4bb0cbcaafa0
- Name: `audit-code`; no feature to turn it off; one subagent per axis, five at most.

## Deliverable 1 summary
- done: skill `audit-code` (`files/.claude/skills/audit-code/SKILL.md`, synced), its required passages in
  `crates/cardinal/tests/skills.rs`, the `README.md` skill list and the issue lifecycle of `docs/workflow.md`.
- checks: tests ok, lint ok, specs ok (no spec: harness), docs updated, fixed context of the skill descriptions
  +330 bytes, under `fixed_context_budget`.
- red: none (no spec id); the test failed first with `audit-code: skill missing, cannot hold 'one subagent per axis'`.
- Code review: no findings
- not verified: the skill run on a real repository (a billed session).
- file-size: `crates/cardinal/tests/skills.rs` was already over the limit (1212 lines) before this deliverable; its
  split is a refactor of its own, not part of this skill.
