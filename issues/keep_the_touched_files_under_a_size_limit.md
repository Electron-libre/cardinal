---
priority: medium
why: 'the files of a consumer grow past a thousand lines in days and no step of the workflow looks at them, while every later read and edit of such a file costs the agent'
---
# Keep the files a change touches under a size limit

# Goal
Every step of `skill:issue-workflow` that could step back looks at the diff, never at the file it lands in: the plan
counts the changed lines, the refactor of step 3.3 is local to the new code, the reviews read the diff,
`deliverable-size` counts the changed lines, `skill:prioritize-issues` reads the roadmap and the issues. So a module
grows one deliverable at a time and is only split when the user notices. A large file costs the agent at every later
touch (whole reads, ambiguous `Edit` matches, context spent on unrelated code) as it costs a person. The outcome: a
change that would grow a file past a limit says where its code goes before it is written, and the files already
over the limit become restructuring issues.

# Findings
* Measured 2026-11-13 on a consumer project (a Rust workspace with a client and a server crate, 761 commits, 199
  `.rs` files, 80,402 lines): 21 files over 800 lines. The four largest grew in a week: 1,872 lines in 47 commits
  since 2026-11-06, 1,594 lines in 38 commits since 2026-11-08, 1,442 lines in 41 commits since 2026-11-09, 1,113
  lines in 24 commits since 2026-11-09. Test files reach 1,355 lines.
* 89 `refactor` commits there, nearly all local (a shared helper, a narrower borrow); the few splits into modules
  ("move the save of the documents out of registry.rs") came after the fact.
* Whole reads of code files already show among the `oversized-result` breaches (BRE-19,
  `issues/done/hint_before_a_whole_read_of_a_large_file.md`); `read-hint` bounds the read, not the growth.

# Approach
1. A key `file_size_limit` in `[project]` of `.claude/cardinal.toml` (default 800 lines, test files included, `0`
   turns it off), read like `deliverable_size_limit`.
2. Plan (step 2 of `skill:issue-workflow`): for each file the plan touches that is over the limit, or that the
   estimate pushes over it, the plan says where the new code goes: a new module, or a first deliverable that splits
   the file by responsibility with no change of behaviour.
3. A tool `cardinal run file-size [<base>]`, beside `deliverable-size`: lists the files the diff pushes over the
   limit, or grows while already over it, with their lines before and after; exits 1 when one is listed. Run at
   step 5; a listed file is cut in the deliverable, or the reason it is not is written in the summary of step 6.
4. `skill:prioritize-issues` lists the files over the limit (largest first) and proposes a restructuring issue for
   the worst, with a cut by responsibility, for the user to validate.
5. Measure: a metric of `cardinal-report` for the lines of the largest file a deliverable touched, next to
   `oversized-result`.

# Specs
* A new spec of `file-size` (or an amendment of `specs/deliverable-size.md`, SIZ), the requirements of the key and
  of the tool.

# Resources
* `crates/cardinal/src/tools/deliverable_size.rs`, `specs/deliverable-size.md`, ADR 0009: the tool to model on.
* `files/.claude/skills/issue-workflow/SKILL.md` (steps 2 and 5), `files/.claude/skills/prioritize-issues/SKILL.md`.
* `specs/rule-breaches.md` BRE-19, `specs/read-hint.md`, `specs/cardinal-report.md`.

# Hints
* A file over the limit may hold one cohesive responsibility (a table of data, a long test of one capability): the
  tool reports, the plan decides; never a refusal.
* Count the lines as `wc -l` does, on the files of the diff only, so the check stays cheap.
* Verdict: `Expected-effect: bytes_per_session:oversized-result down`, and the new metric down.
