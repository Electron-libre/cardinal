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
