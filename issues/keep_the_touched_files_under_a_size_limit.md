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
* `specs/file-size.md` (FSZ-1 to FSZ-9, new).

# Resources
* `crates/cardinal/src/tools/deliverable_size.rs`, `specs/deliverable-size.md`, ADR 0009: the tool to model on.
* `files/.claude/skills/issue-workflow/SKILL.md` (steps 2 and 5), `files/.claude/skills/prioritize-issues/SKILL.md`.
* `specs/rule-breaches.md` BRE-19, `specs/read-hint.md`, `specs/cardinal-report.md`.

# Hints
* A file over the limit may hold one cohesive responsibility (a table of data, a long test of one capability): the
  tool reports, the plan decides; never a refusal.
* Count the lines as `wc -l` does, on the files of the diff only, so the check stays cheap.
* Verdict: `Expected-effect: bytes_per_session:oversized-result down`, and the new metric down.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-13

## Understanding
Report the files a change pushes over `file_size_limit` (or grows while over it) and the files of the repository over
it, then have the plan, the review step and `skill:prioritize-issues` act on that report. Out of scope: any refusal,
the split itself, the new metric of `cardinal-report` (see Decisions).

## Plan
1. [x] The tool `cardinal run file-size [<rev> | <a>..<b> | --all]` and the key `file_size_limit` (seeded
   `cardinal.toml` comment, `docs/architecture.md`, `docs/usage.md`); verified by tests over temporary repositories;
   makes true FSZ-1 to FSZ-9. (autonomous)
2. [x] The skills: `issue-workflow` step 1 (where the code goes for a touched file over the limit, or a split deliverable
   first), step 3.5 (run `file-size`, cut or say why in the summary of 3.6), `prioritize-issues` (`file-size --all`,
   a restructuring issue proposed for the worst), `docs/workflow.md`; verified by `just check` and `just sync`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 435978cf46b5f58d4e6c052926e6be80f54e86a6
- The metric of the largest touched file in `cardinal-report`: a separate issue later; `oversized-result` measures the effect meanwhile.
- `file-size` exiting 1 at step 3.5 does not block: cut the file or write why not in the summary of 3.6.
- Commit 1823f53 (outside the plan): clippy 1.100 of the stable toolchain failed `-D warnings` on 32 sites of
  `map(_).unwrap_or_default()`; fixed apart, before deliverable 1, so the base stays green.
- The tests run with `TMPDIR` outside `/tmp` (its quota was full of other sessions' directories) and outside the
  repository (`audit_sources` reads the `CLAUDE.md` of the parents of its temporary directory).

## Deliverable 1
done: `cardinal run file-size [<rev> | <a>..<b> | --all]`, the key `file_size_limit` (800, 0 off), its spec accepted, docs.

| check | result |
|---|---|
| tests | pass (`file_size` 13, `deliverable_size` 14) |
| lint | pass |
| specs | pass, FSZ accepted |
| docs | `docs/usage.md`, `docs/architecture.md`, seeded `cardinal.toml` |
| code review | see below |

red: FSZ-1, FSZ-7, FSZ-H1: `left: (Some(3), "")` (the tool did not exist), 12 of 13 tests red.

Code review: 1 findings
- `a...b` and an open range `a..` were split by hand into bad revisions (exit 3): fixed, an empty side reads
  `HEAD` and `...` compares with the merge base (FSZ-2 amended, tested).

not verified: none. decisions needed: none.

## Deliverable 2
done: `issue-workflow` step 1 (where the code goes for a touched file over the limit) and step 3.5 (`file-size`, split
or say why), `prioritize-issues` (`file-size --all`, restructuring issues proposed), `docs/workflow.md`.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | `docs/workflow.md` |
| sync | copies updated, `check-cardinal` pass |
| code review | see below |

red: none (skill text).

Code review: no findings

not verified: none. decisions needed: none.
