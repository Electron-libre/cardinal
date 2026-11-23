---
name: audit-code
description: Audit the code of the whole repository (architecture, size and duplication, dead code, test coverage of the specs, drift from the specs and ADRs) with evidence, then turn the validated findings into issues. Use when the user asks to audit the code, review the repository as a whole or find technical debt.
---

# Audit the code

`code-review` reads a diff; this skill reads the repository as it stands. Propose first: write only the issues the
user validates. Change no code here.

## Procedure

1. **Scope**: the whole repository, or the path or axis the user names. Read the `architecture`, `roadmap`,
   `skipped_dirs` and `confidential` values of `.claude/cardinal.toml`, the ADRs (`docs/adr/` or what the
   `architecture` document cites), the open issues (`ls issues`) and their titles: what an open issue already tracks
   is no finding.
2. **Measure** what a tool already counts, before any reading: `cardinal run file-size --all` (the files over the
   limit), and with specs, `cardinal run check-specs` (untraced requirements) and `cardinal run spec-coverage` (test
   files tracing no requirement). Skip a tool the project turned off; its absence is no finding.
3. **Read through one subagent per axis**, five at most, in parallel, each read-only, given the scope, the measures
   of step 2 that concern it, the skipped directories and the axis:
   - **architecture**: the modules, layers and dependencies against the `architecture` document (a flow it draws
     that the code no longer follows, a module it omits, a dependency that crosses a boundary it states);
   - **size and duplication**: the files and functions over the limit or mixing responsibilities, the logic
     copied in several places;
   - **dead code**: what nothing calls or reads (functions, flags, config keys, recipes, docs of a removed command);
   - **tests**: the requirements no test proves, the behaviour no test covers, the tests that assert nothing;
   - **drift**: the code that contradicts a spec requirement, an ADR decision or a doc of the project.
   Each returns at most ten findings, each with `file:line`, what is wrong, why it matters and a fix in one line;
   nothing found is a valid return. A guess with no place in the code is no finding.
4. **Verify and rank**: reread the cited lines of each finding yourself; drop one the code does not show, a
   duplicate across axes, a matter of taste, and what the project's lint or an existing issue already covers. Rank
   by impact (a bug or a broken contract, then what slows every change, then cleanliness) and cost; group the
   findings of one cause into one proposal.
5. **Report, then STOP**: a table `# | axis | where | finding | proposed issue | priority`, `where` as `file:line`,
   most important first; proposing nothing is a valid outcome. The user answers with the numbers to keep.
6. **Create** one issue per validated proposal through `skill:create-issue` (its duplicate check, template, milestone
   and checker), the findings and their `file:line` in its `# Findings`, in generic wording per the `confidential`
   note. Reordering the queue is `skill:prioritize-issues`'s.
