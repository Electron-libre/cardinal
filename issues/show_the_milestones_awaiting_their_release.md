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
