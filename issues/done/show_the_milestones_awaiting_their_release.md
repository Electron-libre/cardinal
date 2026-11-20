---
priority: high
depends_on: [print_the_release_proposal_as_json]
why: 'the user must see from the UI that a release awaits before starting it'
---
# Show the milestones awaiting their release

# Goal
A milestone set `done` by the close of its last issue waits for a release, but the milestones panel of
`cardinal-ui` shows it as any other `done` one. The outcome: the panel marks each `done` milestone without a
`Released:` line as awaiting its release, with the version `cardinal run release --json` proposes.

# Findings
* STU-1 lists the milestones not yet released (`roadmap --open --json`), each with its state.
* STU-6 shows the error of a tool that fails or prints no document of version 1 in its panel.

# Approach
1. Read `cardinal run release --json` with the other tools (STU-3), in the same read.
2. Show `awaits release v<version>` after such a milestone; an error of the tool in the panel as STU-6 says.

# Specs
* `specs/steering-ui.md`: amends STU-1, STU-3, STU-6.

# Milestone
Releases from the steering UI

# Resources
* `specs/steering-ui.md`, `crates/cardinal-ui/src/panels.rs`, `crates/cardinal-ui/src/source.rs`

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
Mark in the milestones panel each milestone that `release --json` lists (matched on name and document) as
`awaits release v<proposed>` (bare `awaits release` when `proposed` is `null`); run `release` only when a listed
milestone is `done`. Out of scope: preparing or tagging the release from the UI, splitting the spec
(`issues/done/split_the_steering_ui_spec.md`).

## Plan
1. [x] Awaiting release mark: `source.rs` reads `release --json` after `roadmap` when a milestone is `done`, `panels.rs`
   writes the mark, a failed `release` keeps the milestones with its error line under them; verified by
   `tests/print.rs` (fake `cardinal`); makes true STU-1, STU-3, STU-6 (amended). (autonomous)
2. [x] Fixes of the `high` review: `release --json` run again only once `HEAD`, a tag or the roadmap changed, the
   prepared-not-tagged gap a non-goal, two tests assert `release` ran, README reworded; verified by an internal test of
   the memo; makes true STU-3 (amended). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 1e109b1ef199aa3fb5194ac7543244f36e5f4d53
- Questions of the plan stop answered `ok`: `release` only with a `done` milestone (1 a), its error under the milestones (2 a), bare `awaits release` on a `null` proposal (3 a).
- `release` runs only when `roadmap` lists a `done` milestone, so a project with nothing to release pays no read of its commits every 2 s.
- A failed `release` does not hide the milestones (unlike the other tools of STU-6): its line goes under them.

## Deliverable 1: awaiting release mark
done: the milestones panel marks each milestone `release --json` lists `awaits release v<proposed>`, a failed `release` under the milestones.

| check | result |
|---|---|
| tests | `cargo test -p cardinal-ui`: pass (6 new in `tests/print.rs`) |
| lint | clippy `-D warnings`: clean |
| specs | STU-1, STU-3, STU-6 amended, spec back to `accepted` |
| docs | `docs/usage.md`, `docs/architecture.md`, `README.md` |
| code review | see below |

red: STU-1, STU-3, STU-6, STU-8: `no line with awaits release` / `the error goes under the milestones` (the fake `release` never ran)
Code review: no findings
not verified: the mark in the live terminal screen (only `--print` is tested; the panel code is shared).
decisions needed: none

## Deliverable 2: fixes of the `high` review
done: `release --json` is memoized on `HEAD`, the tags and the roadmap document; spec non-goal and README fixed; two tests tightened.

| check | result |
|---|---|
| tests | `cargo test -p cardinal-ui`: pass (1 new internal test) |
| lint | clippy `-D warnings`: clean |
| specs | STU-3 amended, a non-goal added, spec `accepted` |
| docs | `README.md`, `docs/usage.md` |
| code review | see below |

red: STU-3: `assertion left == right failed: left: 2, right: 1` (the memo ran the read again under the same key)
Code review: no findings
`high` review of the issue: 5 findings: the cost of `release` every 2 s fixed (memo); prepared-not-tagged made a
non-goal; two tests now assert `release` ran; README reworded; an older pinned `cardinal` without `release --json`
not fixed: the UI reads the tools of its own pinned revision (ADR 0030).
not verified: the memo in the live screen (only the internal test proves it).
decisions needed: none
