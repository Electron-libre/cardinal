---
rank: 160
why: 'after the audit that found it; small, and it takes a computation out of the prose of the workflow'
---
# Compute the size of a deliverable with a tool

# Goal
`skill:issue-workflow` asks the agent to estimate and then check the size of each deliverable by hand: added plus
removed lines from `git diff --shortstat`, new files included, `issues/` and lock files left out, a move that also
edits counted twice, against ~1,000 lines per deliverable and ~3,000 per issue. A `cardinal run` tool computes it,
so the skill says "run the tool" instead of the rule, and the review step stops on a number, not on an estimate.

# Findings
* The audit of the shared instructions (`issues/done/audit_the_shared_instructions_for_conflicts_and_stale_guidance.md`,
  row M1) found the rule in prose in steps 1 and 3.5 of `files/.claude/skills/issue-workflow/SKILL.md`, and its
  split case in `details.md`.
* `git diff --shortstat` leaves out the untracked files and counts a pure rename as its whole content unless
  `-M` is given.

# Approach
1. A spec of the size tool: its inputs (the working tree against `HEAD`, or a range `<base>..HEAD`), the files it
   leaves out, how it counts a rename and a moved-then-edited file, the limits it reports against.
2. `cardinal run deliverable-size [<range>]` printing the count and whether it exceeds the limit, the limits read
   from `.claude/cardinal.toml` with the current ones as defaults.
3. `issue-workflow` steps 1 and 3.5 call the tool instead of restating the counting rules.

# Specs
* `specs/deliverable-size.md`, to write at `issue-workflow` step 1.

# Milestone
Lean agent loop

# Resources
* `files/.claude/skills/issue-workflow/SKILL.md` (step 1 "Size limit", step 3.5), `details.md` ("Split an issue").
* `crates/cardinal/src/tools/repo_files.rs` (ADR 0009: scanners list the files through git).

# Hints
* The issue total counts a moved file edited later twice: the tool reads the range, not only the last diff.

# Tracking

Plan: validated

## Understanding
A tool `cardinal run deliverable-size [<rev> | <a>..<b>]` counts the changed lines (renames detected only when exact,
so a moved-then-edited file counts twice) and exits 1 over the limit; the skill calls it at step 3.5 and on a resume.
Out of scope: counting deliverables, estimating a plan before its code exists (step 1 keeps the estimate).

## Plan
1. [x] Tool: `deliverable-size` with its spec, the limits `deliverable_size_limit` / `issue_size_limit` in the seed;
   verified by `tests/deliverable_size.rs` driving the binary in temp repos; makes true SIZ-1 to SIZ-10.
2. [ ] Skill: `issue-workflow` steps 0, 1 and 3.5, `details.md` and `docs/workflow.md` name the tool instead of the
   counting rules; verified by `just check` (skills tests, specs) and a run of the tool on this issue's range.

## Decisions
- Mode: autonomous
- Base commit: 7121eb25d5d035e62c71540528209089bf1e6b86
- Limit by argument: none → `deliverable_size_limit`, a `<rev>` or a range → `issue_size_limit` (1 a).
- Left-out files: a fixed list, no configuration key (2 a).

## Deliverable 1

- done: `cardinal run deliverable-size` with `specs/deliverable-size.md` (accepted), the limits in the seed,
  `docs/usage.md` and `docs/architecture.md`; 450 changed lines by its own count.

| check | result |
|---|---|
| tests | `tests/deliverable_size.rs`: 10 passed |
| lint | clippy clean |
| specs | `just check-specs` ok, SIZ-1 to SIZ-10 traced |
| docs | `docs/usage.md` (new section), `docs/architecture.md` (config keys), seed |
| code review | `/code-review low`: clean |

- red: SIZ-1, SIZ-2, SIZ-3, SIZ-4, SIZ-5, SIZ-6, SIZ-7, SIZ-8, SIZ-9, SIZ-10: `cardinal: unknown tool
  \`deliverable-size\``, exit 3 instead of 0.
- Code review: no findings
- not verified: a path that is not UTF-8 (read lossily, then not found on disk for an untracked file).
- decisions needed: none.
