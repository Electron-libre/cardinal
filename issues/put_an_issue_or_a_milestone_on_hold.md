---
priority: medium
why: 'deferred work keeps coming up as next today, and the only way out is a fake dependency or a lowered priority that lose the real order'
---
# Put an issue or a milestone on hold

# Goal
A user who defers an issue, or a whole milestone, has no way to say so: `next-issue` keeps offering it as long as it
is ready (NXT-1, NXT-7), and a queue handed to the agent picks it up. Lowering its `priority`, inventing a
`depends_on` or moving the file out of `issues/` loses its real order or breaks the links of the roadmap. The
outcome: an issue or a milestone can be put on hold with a reason, `next-issue` never offers it and lists it apart
with that reason, and taking it off hold restores its place in the order.

# Findings
* `crates/cardinal/src/tools/next_issue.rs` `order`: the sort key is `(blocked, milestone place, priority, age,
  stem)`; only `depends_on` keeps a ready issue from being offered, and nothing reads the state of a milestone.
* `specs/roadmap.md` RMP-3: the milestone states are `idea`, `shaped`, `in issues` and `done`; any other value is
  shown as `<value> (unknown)`, so a `paused` state today is displayed but changes no order.
* `specs/issue-claim.md` non-goals: claiming a blocked issue is not refused, `issue-workflow` warns about it; a held
  issue named explicitly by the user would need the same treatment.

# Approach
1. Issue hold: a front matter key (e.g. `on_hold: '<reason>'`), read like `why`. `next-issue` leaves a held issue
   out of the offer and lists it after the blocked ones as `On hold: <stem> (<reason>)`; an issue whose
   `depends_on` names a held issue stays blocked as today (it is not done).
2. Milestone hold, candidates:
   a. a new state `on hold` in the roadmap (`- State: on hold`), added to the known states of RMP-3; or
   b. a separate field `- On hold: <reason>` kept next to the state, so the state still says how far the milestone
      went and `close-issue` (CLS-8) and `release` (REL-9) keep their logic.
   Every open issue of a held milestone is then held, its reason naming the milestone.
3. Surface it where the order is read: `roadmap` shows the hold, `check-issue` validates the key (CKI), and
   `skill:issue-workflow` warns when the user names a held issue, as for a blocked one;
   `skill:prioritize-issues` and `skill:create-issue` mention the key.
4. Decide whether an older `cardinal` that ignores the key and offers a held issue is a misreading that raises
   `cardinal-format` (ADR 0014), or a harmless degradation.

# Specs
* `specs/next-issue.md`: new requirements for a held issue and a held milestone (offer, listing, report of an
  invalid hold), and a scenario each.
* `specs/roadmap.md`: RMP-1 or RMP-3 amended for the milestone hold.
* `specs/check-issue.md`: a requirement for the hold key.

# Resources
* `specs/next-issue.md`, `specs/roadmap.md`, `specs/check-issue.md`, `specs/issue-claim.md`.
* `crates/cardinal/src/tools/next_issue.rs`, `crates/cardinal/src/tools/roadmap.rs`,
  `crates/cardinal/src/tools/roadmap_state.rs`, `crates/cardinal/src/tools/check_issue.rs`.
* ADR 0019, ADR 0021 (order from the issue files), ADR 0014 (format raise), ADR 0026 (queue chaining).
* `issues/done/warn_when_starting_a_blocked_issue.md`: the precedent of a guard on an issue `next-issue` would not offer.

# Hints
* Keep the hold in the issue file and the roadmap, never in a shared list: a branch that holds an issue must edit
  only that issue (ADR 0021).
* NXT-12 prints `No ready issue: every free one waits on another.` when all free issues are blocked; with held
  issues the message must stay true (held issues do not wait on another).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-03

## Understanding
Hold an issue with a front matter key `on_hold: '<reason>'`, a milestone with a field `- On hold: <reason>` (approach
2b: the state keeps saying how far it went); `next-issue` never offers a held issue and lists it as `On hold:`.
Out of scope: a tool that sets the hold, refusing a claim. No format raise: the key is project data, an older
`cardinal` only offers the held issue.

## Plan
1. [x] Hold in `next-issue`: the `on_hold` key, the milestone field, the `On hold:` lines, the reports and the
   amended NXT-12 message; verified by tests of `crates/cardinal/tests/next_issue.rs`; makes true NXT-12, NXT-15,
   NXT-16, NXT-17. (autonomous)
2. [x] Hold in `roadmap` and `check-issue`: the ` (on hold)` state cell and the empty reason report; verified by
   tests of `crates/cardinal/tests/roadmap.rs` and `check_issue.rs`; makes true RMP-6, CKI-5. (autonomous)
3. [x] Skills and docs: `issue-workflow` warns on a held issue named by the user, `prioritize-issues` and
   `create-issue` name the key, `seed/ROADMAP.md` and `ROADMAP.md` name the field (`files/`, then `just sync`);
   verified by `just check-cardinal` and `just check`. (autonomous)

4. [x] Fixes of the `high` review: a held and blocked issue names its blockers, a reason of spaces is no reason, an
   empty `on_hold` takes the milestone reason, a milestone hold with no reason is reported once and only for the
   milestone of an open issue, the field read by `Milestone::hold`, the groups an enum, the CKI-5 message as the
   spec writes it; verified by tests of `next_issue.rs` and `check_issue.rs`; makes true NXT-15, NXT-17, CKI-5.
   (autonomous)

## Decisions
- Mode: autonomous
- Base commit: a0ed329380daac6d2843d9ded52fb0465fb622e8
- Milestone hold: a separate `On hold:` field, not a state (question 1 a).
- No format raise (question 2 a).

## Summary of deliverable 1
done: `next-issue` holds the issues with an `on_hold` key or in a milestone with an `On hold:` field, lists them as
`On hold:` after the blocked ones and reports a hold with no reason; the three specs amended (draft).

| check | result |
|---|---|
| tests | ok (`next_issue`: 17 passed) |
| lint | ok (clippy -D warnings) |
| specs | ok (`check-specs`) |
| docs | user docs in deliverable 3 |
| code review | low, no findings |

red: NXT-15, NXT-12, NXT-16, NXT-17: the held issue `a` was offered as `Next: issues/a.md`.
Code review: no findings
not verified: none
decisions needed: none

## Summary of deliverable 2
done: `roadmap` marks a held milestone `<state> (on hold)`, `check-issue` reports an `on_hold` with no reason; the
three specs set `accepted`.

| check | result |
|---|---|
| tests | ok (`roadmap` 6, `check_issue` 7, `next_issue` 17) |
| lint | ok |
| specs | ok (accepted, all traced) |
| docs | user docs in deliverable 3 |
| code review | low, no findings |

red: CKI-5: `check-issue` accepted `on_hold: ''`; RMP-6: the state cell read `shaped`.
Code review: no findings
not verified: none
decisions needed: none

## Summary of deliverable 3
done: `issue-workflow` asks before starting a held issue, `prioritize-issues` and `create-issue` name the key and
leave the hold to the user, the seeded and own `ROADMAP.md`, `docs/usage.md` and `docs/workflow.md` describe it.

| check | result |
|---|---|
| tests | ok (`just check`) |
| lint | ok |
| specs | ok |
| docs | `update-docs` map: seeded file changed, so `docs/usage.md` says what a project seeded before does |
| code review | low, see below |

red: none
Code review: no findings
not verified: the wording of the skills is read by the model only; no test drives a held issue through a session
decisions needed: none

## Summary of deliverable 4
done: the fixes of the `high` review of the whole issue (NXT-15, NXT-17 and CKI-5 amended), and `issue-workflow` never
starts a held issue that is also blocked.

| check | result |
|---|---|
| tests | ok (`next_issue` 19, `check_issue` 7, `roadmap` 6) |
| lint | ok |
| specs | ok (accepted) |
| docs | n/a: the user docs name no message these fixes change |
| code review | low, see below |

red: NXT-15, NXT-17, CKI-5: no `blocked by x` on the `On hold:` line, the milestone reason lost, `on_hold: '  '`
accepted (tests run on the code before the fix).
Code review (high, whole issue): 10 findings; 7 fixed here; not fixed: `on_hold: false` still holds (the key holds,
the docs say to remove it), an indented `On hold:` sub-bullet reads as the field (the grammar `State:` and
`Released:` share, RMP-1), no format raise (decided at the plan, question 2 a).
Code review: no findings
not verified: none
decisions needed: none
