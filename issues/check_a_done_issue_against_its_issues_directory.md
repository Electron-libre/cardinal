---
priority: low
why: '`check-issue` validates new issues; a done issue is checked only by hand'
---
# Check an issue of issues/done against its issues directory

# Goal
For `issues/done/foo.md`, `check-issue` takes `issues/done` as the issues directory: it always reports
`file must be issues/<snake_case>.md`, and looks for each `depends_on` stem in `issues/done/` and `issues/done/done/`,
so a dependency still open in `issues/` is reported missing. The outcome: either `check-issue` checks a done issue
against `issues/`, or it refuses a file outside `issues/` with one clear message.

# Findings
* `crates/cardinal/src/tools/check_issue.rs`: `front_matter_problems` (the `dir` of the dependencies) and the name check of `problems`.
* Found by the `high` review of `find_the_root_of_a_done_issue`, which fixed only the project root (FEA-9).

# Approach
1. Decide whether `check-issue` accepts a done issue at all (CKI spec, non-goals).
2. If so, take the issues directory as the parent of `done/` for a file in it.

# Specs
* `specs/check-issue.md` CKI-2.

# Resources
* `crates/cardinal/src/tools/check_issue.rs`, `crates/cardinal/tests/check_issue.rs`.
