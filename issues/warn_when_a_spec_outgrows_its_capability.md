---
priority: medium
why: 'a spec of tens of kilobytes is read whole by every issue that amends it, and nobody reads it as a whole'
---
# Warn when a spec outgrows its capability

# Goal
`skill:write-spec` says to amend the spec of a capability rather than create a second one, and nothing bounds a
spec: every issue of a milestone appends to the same file, and its Intent turns into the list of what was appended.
The outcome: `check-specs` warns when a spec passes a size the project sets, `skill:write-spec` says how to split
one into narrower capabilities, and the Intent stays the two sentences the template asks.

# Findings
* Measured 2026-11-12 over four repositories: median spec 5 to 22 KB, largest 34 to 82 KB; the largest holds 210
  requirements. Median Intent 939 characters in one repository, about 330 in the three others; the template asks
  "one or two sentences".
* The largest spec grew from 9 KB and 36 requirements to 82 KB and 210 requirements in 28 hours, over 65 commits of
  one milestone.
* Its Intent lists every part the issues added ("…, and the resize, …, and the text written and edited in place,
  and the fog zones…").

# Approach
1. `check-specs` warns, per spec, above a count of requirements and/or a size in bytes, read from
   `.claude/cardinal.toml` with defaults (e.g. 40 requirements, 25 KB), and above a length of the Intent.
2. `skill:write-spec` step 1: when the amended spec is over the bound, propose to split it (one spec per
   sub-capability, its own id; the moved requirements keep a `withdrawn:` line pointing to their new id, or the ids
   move with their prefix), as a question to the user.
3. Measure first which bound would have warned on which spec of the four repositories, to pick the defaults.

# Specs
* `specs/spec-clarity.md` (the new warnings).

# Milestone
Specs that read as a product

# Resources
* `files/specs/README.md`, `files/specs/TEMPLATE.md`, `files/.claude/skills/write-spec/SKILL.md`,
  `crates/cardinal/src/tools/check_specs.rs`.

# Hints
* Renumbering an `accepted` spec breaks the citations of the tests and the issues: a split must keep every cited id
  resolvable.
