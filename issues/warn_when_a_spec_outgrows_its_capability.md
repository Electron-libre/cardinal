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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
`check-specs` warns of a spec over a count of requirements, a size in bytes or an Intent length (bounds in
`[project]`, defaults 40 / 25000 / 600), per `draft` spec and in one line for the `accepted` ones (as CLR-10, CLR-12);
`skill:write-spec` proposes a split when the amended spec is over a bound. Out of scope: failing on a bound, splitting
any existing spec. Measured locally on three repositories (the fourth is not on this machine): 40 / 25 KB warn of 2 and 3
specs of 67; an Intent over 600 characters, of 5.

## Plan
1. [x] Bound warnings: CLR-13 to CLR-16 and CLR-H2 in `check_specs.rs`, the keys in the seeded `cardinal.toml` and
   `files/specs/README.md`; verified by tests in `test_check_specs.rs`; makes true CLR-13, CLR-14, CLR-15, CLR-16, CLR-H2. (autonomous)
2. [ ] Split procedure: `skill:write-spec` step 1 proposes, as a question, to split an amended spec over a bound (one
   spec per sub-capability with its own id, the moved requirements `withdrawn: moved to XYZ-n` so every cited id stays
   resolvable, the tests re-cite the new ids); verified by `just check` (skill checks); docs only.

## Decisions
- Mode: autonomous
- Base commit: 29ed2fa23cbef2cad5f5d4f39addd2a674d52fbd
- Bounds: 40 requirements, 25000 bytes, 600 characters of Intent (1 a).
- Split: new ids in the new spec, the old ones `withdrawn: moved to XYZ-n`, the tests re-cite the new ids (2 a).

## Summaries

### 1. Bound warnings
done: `check-specs` warns of a spec over `spec_max_requirements` / `spec_max_bytes` / `spec_max_intent`, fails on a
bound that is not a positive integer; the front matter reading moved out of `parse` (clippy `too_many_lines`).

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | pass |
| specs | pass, `spec-clarity.md` accepted |
| docs | `files/specs/README.md`, `docs/architecture.md`, `seed/.claude/cardinal.toml`, module doc |
| code review | no findings |

red: CLR-13, CLR-14, CLR-15, CLR-16, CLR-H2: `FAIL - a draft of 41 requirements is warned of` (no bound warning printed)
Code review: no findings
not verified: none
decisions needed: none
Here, 5 accepted specs of this repository now warn (`read-hint`, `rule-breaches`, `spec-tracing`, `sync`,
`workspace-lock`): splitting them is out of scope.
