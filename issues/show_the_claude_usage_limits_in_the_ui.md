---
priority: medium
why: 'starting an issue near a usage limit leaves it half done; the limits are read only inside a Claude Code session'
---
# Show the Claude usage limits in the UI

# Goal
Before starting a session from the UI, the user cannot see how much of the 5-hour and weekly limits of their Claude
plan is used. The expected outcome is the used share and the reset time of each limit on the screen.

# Findings
* Nothing in `cardinal-ui` nor in the tools reads a usage limit today.
* The source of these numbers is not established yet.

# Approach
Measure first: where the numbers can be read without network in the tests and without a credential in the UI.
1. The JSON Claude Code hands to a status line command, if it carries the limits: a status line script of the
   harness writes them to the cache directory (STU-18), the UI reads that file and shows its age.
2. The `/usage` view of Claude Code, if it can be run headless.
3. An estimate from the session transcripts (tokens over the last 5 hours and 7 days), labelled as such.
Show `used <n>% · resets <HH:MM>` per limit on the status or measures line, `unknown` when no source answers.

# Specs
* `specs/steering-ui.md`: a new requirement; the spec of the status line script if 1 is chosen.

# Milestone
Cardinal UI

# Resources
* `crates/cardinal-ui/src/status.rs`, `measures.rs`; `specs/steering-ui.md` (STU-16 to STU-19).

# Hints
* Never read nor copy the OAuth credentials of Claude Code.
