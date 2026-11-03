---
priority: medium
why: 'the roadmap grows with every release and its open milestones get hard to find; nothing waits on it'
---
# Archive the released milestones

# Goal
Released milestones stay in the roadmap forever: `close-issue` sets them `done` (CLS-8), `release` adds
`- Released: v<version>` (REL-9), and nothing ever moves them. The roadmap soon holds more finished milestones than
open ones, and the user can no longer see which are open. The outcome: `release <version>` moves each milestone it
marks released out of the main roadmap into an archive document, so the main one holds only the milestones still to
land, with no manual step.

# Findings
* 2026-11-02, this repository: `ROADMAP.md` holds 6 milestones, 5 of them `done` with a `Released:` line, one
  `in issues`.
* `release` already lists the released milestones under `### Milestones` of `CHANGELOG.md` (REL-9), by name only:
  their goal, why and issue list live only in the roadmap.
* `next-issue` reports a milestone no roadmap heading names (NXT-13), and `close-issue` rewrites the issue links of
  the `roadmap` documents (CLS-2): an archive left out of `roadmap` would break both for a late issue of a released
  milestone.
* REL-5 restores the `roadmap` documents when `release_version_command` fails.

# Approach
1. Decided with the user: the archive stays listed under `roadmap`, so NXT-10/13 and CLS-2 keep reading it.
2. Name the archive, candidates:
   a. a new key `roadmap_archive` of `.claude/cardinal.toml` (no archive without it, today's behaviour);
   b. the last document of `roadmap` when it lists two or more;
   c. a fixed name next to the first document (`ROADMAP-done.md`).
3. `release <version>` moves the whole section of each milestone it marks (heading to the next heading, `Released:`
   line included) to the top of the archive (newest first) or its end, creating the archive with a short header
   when missing; links stay valid when both documents share a directory, else they are rewritten.
4. Seed: the seeded `ROADMAP.md` names the archive in its states paragraph; a seeded archive is optional.

# Specs
* `specs/release.md`: REL-9 amended or a new REL requirement for the move, REL-5 extended to the archive.

# Resources
* `crates/cardinal/src/tools/release.rs` (the roadmap writes), `crates/cardinal/src/tools/roadmap.rs` (the shared
  grammar).
* `specs/release.md`, `specs/close-issue.md` (CLS-2, CLS-8), `specs/next-issue.md` (NXT-10, NXT-13).
* `seed/ROADMAP.md`, `docs/workflow.md`, `docs/usage.md`.

# Hints
* A milestone section ends at the next `##` heading outside fenced blocks and HTML comments: reuse the grammar of
  `roadmap.rs`, never a raw text search (the template milestone of the seed sits in an HTML comment).
* The move must keep the order of the milestones still in the main roadmap: `next-issue` orders by it.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-03

## Understanding
`release <version>` moves every released milestone (newly marked or released before) of the main `roadmap` documents
to the end of an archive named by a new key `roadmap_archive`, itself listed in `roadmap`; no key, no move. Out of
scope: moving a milestone back, archiving at `close-issue`.

## Plan
1. [x] Archive on release: `roadmap_archive` read by `release`, the section move through `roadmap.rs` (a section
   range per milestone), links rebased with the helpers of `close_issue.rs`, REL-5 restore extended, the seeded
   `ROADMAP.md`, `.claude/cardinal.toml` and `ROADMAP-done.md` header, this repository's config, `docs/usage.md`,
   `docs/workflow.md`; verified by `tests/release.rs` (move, refusal, link rebase, restore on a failed version
   command) and a `roadmap.rs` unit test of the section range; makes true REL-13, REL-5. (~350 lines) (autonomous)

## Decisions
- Mode: autonomous
- No seeded `ROADMAP-done.md`: `release` creates it, so a consumer without the key gets no orphan file;
  `seeded_roadmap_is_seeded` exempts the seeded `roadmap_archive`.
- Base commit: e8110012510a8e0810d6a0bd842f349fba342017
- Archive named by the key `roadmap_archive`, refused when `roadmap` does not list it (1 a).
- Moved milestones go to the end of the archive, release order (2 a).
- Every milestone with a `Released:` line moves, the earlier releases included (3 a).
- The seeded `.claude/cardinal.toml` sets `roadmap_archive = "ROADMAP-done.md"` (4 a).

## Deliverable 1
- done: `release` moves the released milestones to the `roadmap_archive`, rebasing their links; seed, config and docs name it.
- checks: `just check` ok (fmt, clippy, tests, specs); docs updated (`docs/usage.md`, `docs/workflow.md`, seeds,
  `ROADMAP.md`); code review low done.
- red: REL-13: `moves_the_released_milestones_to_the_archive` failed on the `ROADMAP.md` assertion (the milestones
  stayed), `refuses_an_archive_the_roadmap_does_not_list` on the exit (accepted).
- Code review: 1 findings: an archive named `./ROADMAP-done.md` was not recognised as the archive (its own released
  milestones taken out, two writes of one file): fixed (`same_path` everywhere, the archive kept apart in the loop),
  test `keeps_the_milestones_of_an_archive_named_with_a_leading_dot`.
- not verified: a real release of this repository (the next one moves its 6 released milestones).
