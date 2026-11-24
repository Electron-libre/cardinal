---
priority: low
why: 'only an issue holding a fenced block with a `# ` line is misread'
---
# Read the sections of an issue with the shared fence grammar

# Goal
The title and the goal of `next-issue` now skip the fenced code blocks (`markdown::fenced`, QUA-12), but two other
readers of the `# ` lines of an issue do not: `check-issue` (its title and its required sections) and the milestone of
`next-issue`. Each should read the headings of an issue through the same grammar, so the tools agree on what an issue
holds.

# Findings
* `high` review of `issues/done/read_the_title_of_an_issue_with_the_shared_fence_grammar.md` (2026-11-24), read from
  the code, not run.
* `crates/cardinal/src/tools/check_issue.rs` (`headings`): every line opening with `# ` counts, inside a fenced block
  too. A body opening with a fenced block holding `# Goal` is reported `first heading must be the title, not # Goal`
  while `next-issue` reads the real title after it; a `# Approach` or `# Resources` quoted in a fenced block satisfies
  the missing-section check.
* `crates/cardinal/src/tools/next_issue.rs` (`milestone`): a `# Milestone` line inside a fenced block (a Hint quoting
  the template) is read as the section, and a fenced `# ` line inside the real section ends it.
* `next_issue.rs` (`title`): an existing issue with no title outside a fenced block (a fence left open before it) is
  printed `(no file)` in the text of `next-issue`.

# Approach
1. Build the `headings` of `check-issue` from `markdown::fenced`, after the front matter, the fenced lines left out.
2. Read the `# Milestone` section of `milestone` through `markdown::fenced`, as `heading` and `goal` do.
3. Print `(no title)` rather than `(no file)` when the file exists but has no title.
4. A test of each; the title and section checks of `check-issue` have no requirement in `specs/check-issue.md`
   yet: add one, and say in NXT-10 that the `# Milestone` section is read outside fenced blocks.

# Specs
* `specs/check-issue.md`, a new requirement for the title and the required sections.
* `specs/next-issue.md` NXT-10 and NXT-1 (the title printed), amended.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/check_issue.rs`, `crates/cardinal/src/tools/next_issue.rs`, `crates/cardinal/src/markdown.rs`.
* `issues/done/read_the_title_of_an_issue_with_the_shared_fence_grammar.md`, `issues/done/read_the_goal_of_an_issue_with_the_shared_fence_grammar.md`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
`check-issue` (its title, required sections, `# Tracking` and empty `# Goal`) and the `# Milestone` of `next-issue`
read the `# ` lines of an issue after its front matter through `markdown::fenced` (QUA-12), as the title and the goal
already do; `next-issue` prints `(no title)` for an issue with no title. Out of scope: the other readers of issues.

## Plan
1. [x] `check-issue` headings through `markdown::fenced`: `headings` and `section_is_empty` read the body after the
   front matter, fenced lines never a heading; verified by tests of `crates/cardinal/tests/check_issue.rs` (a fenced
   `# Goal` before the title, `# Approach`/`# Resources` only fenced, a fenced `# ` line inside `# Goal`); makes true
   CKI-6. (autonomous)
2. [x] `next-issue` milestone and missing title: `milestone` reads its section through `markdown::fenced`, `title`
   prints `(no title)` for a readable file with none; verified by tests of `crates/cardinal/tests/next_issue_json.rs`
   (`tests/next_issue.rs` is over the file size limit); makes true NXT-10, NXT-1 (amended). (autonomous)
3. [x] Review of the whole issue: `update-docs` on `base..HEAD` and the `high` review, their fixes; verified by
   `project-check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 1c13e93ccdba1768e6a453b4c0b9ce95ef5c5eaa

## Deliverable 1

- done: `check-issue` reads its headings after the front matter through `markdown::fenced`; `next_issue::body` shared
  by `heading`, `goal` and `check-issue`; CKI-6 added, the CKI requirements under `### Behaviour` / `### Contract`.

| check | result |
|---|---|
| tests | pass (`check_issue` 9/9) |
| lint | pass |
| specs | pass, `check-issue` back to `accepted` |
| docs | n/a: no page describes how `check-issue` reads the headings |
| code review | no findings |

- red: CKI-6: `first heading must be the title, not `# Goal``, `` `# Goal` is empty``
- Code review: no findings
- not verified: none
- decisions needed: none

## Deliverable 2

- done: `milestone` of `next_issue.rs` reads its section after the front matter through `markdown::fenced`; `title`
  prints `(no title)` for a readable issue with no title, `(no file)` only for one it cannot read.

| check | result |
|---|---|
| tests | pass (`next_issue_json` 14/14, `next_issue` 19/19) |
| lint | pass |
| specs | pass, `next-issue` back to `accepted` |
| docs | n/a: no page describes how the milestone or the title is read |
| code review | no findings |

- red: NXT-10, NXT-1: `left: String("One")`, `right: "Two"`; the text printed `Next: issues/a.md` then `(no file)`
- Code review: no findings
- not verified: none
- decisions needed: none

## Deliverable 3

- done: `update-docs` on `base..HEAD` (no page describes these readers: n/a); the `high` review settled: one
  `next_issue::is_heading` (a line opening with `# ` or a bare `#`) shared by `goal`, `milestone` and `check-issue`,
  the goal of `check-issue` empty when it holds no text outside its fenced blocks, `(no file)` only for a file that
  is gone, the quoted messages of CKI-6 fixed; CKI-6, NXT-1, NXT-10 and NXT-19 amended.

| check | result |
|---|---|
| tests | pass (`check_issue` 10/10, `next_issue_json` 16/16, `next_issue` 19/19) |
| lint | pass |
| specs | pass |
| docs | n/a: no page describes how the headings of an issue are read |
| code review | no findings |

- red: CKI-6, NXT-10, NXT-1: `` `# Goal` is empty`` not reported; `unknown milestone of a: #`; `(no file)` printed
  for a non-UTF-8 issue
- High review: 6 findings: 4 fixed (empty goal and bare `#` agreement of the three readers, `(no file)` of a non-UTF-8
  file, the nested backticks of CKI-6); 2 filed as
  [report_an_open_fence_and_a_blank_title_of_an_issue](report_an_open_fence_and_a_blank_title_of_an_issue.md) (the
  open fence, the blank title: new requirements).
- Code review: no findings
- not verified: none
- decisions needed: none
