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
