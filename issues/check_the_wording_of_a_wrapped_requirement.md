---
priority: medium
why: 'a false warning on every wrapped draft requirement teaches the agent to ignore the clarity checks; no spec of this repository wraps, so only consumers see it'
---
# Check the wording of a wrapped requirement

# Goal
`check-specs` checks the wording of a `draft` requirement (CLR-3, CLR-4) on the first line of its list item only.
A requirement wrapped onto an indented continuation line gets a false "holds 0 `must`" warning when its `must`
sits on a continuation line, and the vague words of its continuation lines are never reported. The checks should
read the whole statement of the requirement, whatever the number of lines it spans.

# Findings
* Reproduced on 2026-11-04 with a minimal consumer whose `specs/foo.md` (`status: draft`) holds:

  ```markdown
  - **FOO-1** (deterministic) The hook, given a file,
    must print its name.
  ```

  `cardinal run check-specs` prints `warning: specs/foo.md: requirement FOO-1: holds 0 `must`, expected 1`.
* Cause: `parse_requirements` in `crates/cardinal/src/tools/check_specs.rs` handles only the lines of
  `## Requirements` that start with `- `; it passes the rest of that line to `check_wording` and skips every other
  line, the continuation lines included.
* `specs/README.md` does not say a requirement holds on one line; no spec of this repository wraps one, the
  report came from an agent working in a consumer.

# Approach
1. Join the continuation lines of a requirement (the indented, non-blank lines that follow its `- ` line, up to the
   next item, blank line or heading) to its statement before `check_wording`, and before the `withdrawn:` test.
2. Or state in `specs/README.md` that a requirement holds on one line, and report a continuation line as a problem
   instead of reading it.

Option 1 matches what a Markdown list item means and what a wrapping editor produces; option 2 is simpler but
breaks the specs of the consumers that already wrap.

# Specs
* `specs/spec-clarity.md`: amend CLR-3 and CLR-4 (or add a requirement) so the statement spans the continuation
  lines of the item.

# Resources
* `crates/cardinal/src/tools/check_specs.rs` (`parse_requirements`, `check_wording`)
* `crates/cardinal/tests/check_specs.rs`
* `specs/spec-clarity.md`, `files/specs/README.md`

# Hints
* A fenced code block inside a list item (CLR-5) must stay ignored once the continuation lines are joined.
* An indented continuation line must not be read as a malformed requirement line (`requirement line must start
  with ...`).
