---
priority: low
why: 'only an issue with a fence left open or a blank `# ` title is misread'
---
# Report an open fence and a blank title of an issue

# Goal
`check-issue` and `next-issue` read the headings of an issue outside its fenced code blocks (CKI-6, NXT-10, NXT-19),
but two malformed issues still get a misleading answer: a fence left open hides every heading after it, and a blank
title `# ` counts as a title. `check-issue` should name the real problem, and `next-issue` print no empty title.

# Findings
* `high` review of `issues/done/read_the_sections_of_an_issue_with_the_shared_fence_grammar.md` (2026-11-24), read from
  the code, not run.
* `crates/cardinal/src/tools/check_issue.rs`: with a fence opened in the goal and never closed, `markdown::fenced`
  reads every line after it as fenced, so `check-issue` reports the real `# Approach` and `# Resources` (or the title)
  missing, with no word of the open fence.
* `crates/cardinal/src/tools/next_issue.rs` (`heading`): a title line `# ` gives `Some("")`; `check-issue` accepts it,
  `next-issue` prints an empty title and `--json` gives `title: ""`, where NXT-1 promises `(no title)` and NXT-19
  `null`.

# Approach
1. Have `check-issue` report a fenced block left open at the end of the body, naming its line, before the heading
   problems it causes.
2. Have `heading` skip a blank title, and `check-issue` report `no # <Title> heading` when the first heading is blank.
3. A test of each.

# Specs
* `specs/check-issue.md` CKI-6, amended (the open fence, the blank title).
* `specs/next-issue.md` NXT-19 (`title`), amended.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/check_issue.rs`, `crates/cardinal/src/tools/next_issue.rs`, `crates/cardinal/src/markdown.rs`.
* `issues/done/read_the_sections_of_an_issue_with_the_shared_fence_grammar.md`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
`check-issue` names a fenced block left open (its opening line) before the heading problems it causes, and a blank
first heading counts as no title in both tools. The title of `next-issue` becomes the text of the first heading as
CKI-6 reads it (a bare `#` included), so the two tools agree on which line is the title.

## Plan
1. [ ] Blank title: `next_issue::heading` returns the trimmed text of the first heading (`is_heading`, outside fences),
   `None` when blank; `check-issue` reports ``no `# <Title>` heading`` on it; verified by a `check_issue.rs` test and a
   `next_issue_json.rs` test; makes true CKI-6, NXT-19.
2. [x] Open fence: `markdown` exposes the line of a fence left open at the end of the lines; `check-issue` reports
   `line <n>: fenced code block left open` first; verified by a `check_issue.rs` test; makes true CKI-7. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 2c9bab78935c488c61b5e6ff2698da8c9d1889fe

## Summary of deliverable 1
done: a blank first heading (`# ` or a bare `#`) is no title: `check-issue` reports ``no `# <Title>` heading``,
`next-issue` prints `(no title)` and `--json` gives `title: null`; one `next_issue::title_of` read by both tools.

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | clean |
| specs | NXT accepted, CKI draft until CKI-7 |
| docs | n/a: no doc describes the title outside the specs |
| code review | see below |

red: CKI-6, NXT-19, NXT-1: `check-issue` accepted the blank title; `--json` gave `title: ""`, not `null`.

## Summary of deliverable 2
done: `check-issue` reports `line <n>: fenced code block left open` (the line of its opening fence in the file) before
the heading problems; `markdown::open_fence` reads it with the fence grammar of QUA-12.

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | clean |
| specs | CKI accepted |
| docs | n/a: no doc describes the problems of `check-issue` outside the spec |
| code review | see below |

red: CKI-7: `check-issue` reported only the missing `# Approach` and `# Resources`, no open fence.
Code review: no findings

not verified: none. decisions needed: none.
