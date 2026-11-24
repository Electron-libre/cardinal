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
* `high` review of `issues/read_the_sections_of_an_issue_with_the_shared_fence_grammar.md` (2026-11-24), read from
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
* `issues/read_the_sections_of_an_issue_with_the_shared_fence_grammar.md`.
