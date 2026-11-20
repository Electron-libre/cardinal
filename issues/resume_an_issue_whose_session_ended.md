---
priority: high
why: 'a session ended by mistake leaves its issue taken with no way out from the steering UI, which stops the issue and the issues that depend on it'
---
# Resume an issue whose session ended

# Goal
When the Claude Code session that claimed an issue ends (quit by mistake, crash, closed tab), its claim stays in the
state `working` and `next-issue` lists the issue as `Taken: ... session ended`. The steering UI then refuses `s`
(STU-9, STU-11: the issue is taken), and nothing says how to get out: the user must know that a new session started
by hand in that same worktree takes the claim back (CLM-2). The user should resume such an issue in one action from
the UI, and `next-issue` should say how.

# Findings
* 2026-11-20, this repository: the session of `show_the_milestones_awaiting_their_release` claimed the issue, then the
  user interrupted it; Claude Code forked it into a background session (`--fork-session --resume`, new session id, new
  process under a `bg-pty-host`). The fork was refused by `workspace-lock check` (the lock named the original session
  and its process, which still ran), then the original process was ended by mistake.
* After that: the claim names the original session and a process that no longer runs, in state `working`; the lock
  file of the worktree is gone (`release` at `SessionEnd`); the worktree is clean; the forked background process still
  runs in the worktree, idle.
* `next-issue` prints `Taken: show_the_milestones_awaiting_their_release (claimed in <worktree>, session ended)` and
  blocks `start_a_release_session_on_the_trunk` behind it; the UI shows `claimed in <worktree>, session ended` and its
  `s` key starts nothing.
* No command frees or hands over a claim except `close-issue` (CLM-7) and a new `issue-claim` from the same worktree
  (CLM-2); the only other way is `git update-ref -d refs/cardinal/claims/<stem>` by hand.

# Approach
1. UI: let `s` / `--start <stem>` resume an issue whose claim is `working` and whose process no longer runs: reuse the
   existing worktree of the claim (no `worktree open`), open the zellij tab there with
   `claude "/issue-workflow issues/<stem>.md"` (or `claude --resume <session>` to keep the conversation), so the new
   session takes the claim back by CLM-2.
2. `next-issue`: after `session ended`, print the action that resumes it (the worktree to open and the command), so
   the user and an agent know the way out without reading the spec.
3. Optionally a `cardinal run issue-claim --release <stem>` that deletes a claim whose process no longer runs, for an
   issue the user wants to give up or restart elsewhere (compare-and-swap, refused while the process runs).
4. Check the fork case: a background fork of a live session is refused by the lock (another process); decide whether
   that is wanted (two processes on one worktree) or whether the fork should be told to wait or to stop.

# Specs
* `specs/steering-ui.md`: amend STU-9 and STU-11 (resume a claim whose session ended).
* `specs/issue-claim.md`: amend CLM-5 (the resume hint), maybe a new CLM-n for `--release`.

# Resources
* `specs/issue-claim.md`, `specs/workspace-lock.md`, `specs/steering-ui.md`
* `crates/cardinal/src/tools/issue_claim.rs`, `crates/cardinal/src/tools/next_issue.rs`, `crates/cardinal-ui/src/`
* ADR 0022 (claims and worktree lock), ADR 0024 (hand over the lock of a clean session)

# Hints
* The process of a claim is `CLAUDE_PID`; a background fork runs under another pid with another session id, so
  neither CLM-2 nor WSL-2 sees it as the same session.
* A resumed session must not lose work in progress: CLM-9 skips the dirty-tree check when a claim is taken back.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
An issue whose claiming session ended is resumed in one action: `cardinal-ui` `s` / `--start` opens the tab in the
claim's worktree (fresh `claude "/issue-workflow …"`, CLM-2 takes the claim back), `next-issue` prints how to resume
or give up, and `issue-claim --release` frees a dead claim. Out of scope: handing a claim to a fork of a live session.

## Plan
1. [x] `issue-claim --release <stem>`: deletes a `working` claim whose process ended (CAS), refuses a live, absent or `merge pending` one; verified by `tests/issue_claim.rs`; makes true CLM-10. (autonomous)
2. [x] `next-issue` resume hint: the taken line of an ended claim says `resume: …` or `worktree gone`, then `give up: …`; verified by `tests/issue_claim.rs`; makes true CLM-5, CLM-H2. (autonomous)
3. [x] `cardinal-ui` resume: `s` / `--start` on a ready issue whose claim's session ended opens the tab in the claim's worktree without `worktree open`, refuses when that worktree is gone; verified by `crates/cardinal-ui/tests/start.rs`; makes true STU-9, STU-11. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 473be06d5ea1ed679339d904ce28f4860cd3ed13
- Resume command: a fresh `claude "/issue-workflow issues/<stem>.md"` in the claim's worktree, not `claude --resume` (robust when the transcript is gone; keeps the pane command STU-25 matches).
- `issue-claim --release` is in scope (deliverable 1): the only way out when the claim's worktree was removed.
- Fork of a live session: the lock keeps refusing it; it takes over once the original session ended (non-goal of CLM).
- The split of `specs/steering-ui.md` (over `spec_max_bytes` before this issue) is a follow-up issue, not done here.

## Deliverable 1: summary
- done: `cardinal run issue-claim --release <stem>` deletes a `working` claim whose process ended (CAS), refuses a missing, `merge pending` or live one.

| check | result |
|---|---|
| tests | `issue_claim` 13 passed |
| lint | clean (project-check) |
| specs | CLM-10 traced; `issue-claim.md`, `steering-ui.md` draft |
| docs | `docs/architecture.md`, `issue-workflow/details.md` |
| code review | see below |

- red: CLM-10: `assertion left == right failed: no claim: usage: issue-claim issues/<stem>.md` (exit 2)
- Code review: no findings
- not verified: none
- decisions needed: none

## Deliverable 2: summary
- done: the taken line of a `working` claim whose session ended says `resume: claude "/issue-workflow issues/<stem>.md" in <worktree>`, or `worktree gone`, then `give up: cardinal run issue-claim --release <stem>`.

| check | result |
|---|---|
| tests | `issue_claim` 14, `next_issue` 19, `next_issue_json` 9 passed |
| lint | clean (project-check) |
| specs | CLM-5, CLM-H2 traced |
| docs | `docs/architecture.md`, `issue-workflow/details.md`, `next_issue.rs` module doc |
| code review | see below |

- red: CLM-5, CLM-H2: `Taken: a (claimed in <ws>, session ended)` printed with no `resume:` nor `give up:`
- Code review: 1 findings: the doc comment of `next` was attached to `way_out` (nit), fixed.
- not verified: none
- decisions needed: none

## Deliverable 3: summary
- done: `cardinal-ui` `s` / `--start` on a ready issue whose `working` claim's session ended opens the tab in the claim's worktree with no `worktree open` (`resumed <stem> in <worktree>`); a removed worktree is refused naming `issue-claim --release`; `issue-claim.md` and `steering-ui.md` back to `accepted`.

| check | result |
|---|---|
| tests | `cardinal-ui` all passed (start: 8) |
| lint | clean (project-check) |
| specs | STU-9, STU-11 traced; both specs `accepted` |
| docs | `docs/usage.md`, `README.md`, `start.rs` and `main.rs` doc |
| code review | see below |

- red: STU-9, STU-11: `--start d` exited 1 with `d: taken` (left: Some(1), right: Some(0))
- Code review: 1 findings: the `ready` check also applies to a resumed issue (important); not changed: STU-9 as validated resumes only a `ready` issue, and `state` (NXT-19) is ready, blocked or on hold whatever takes the issue, so a blocked or held issue is not resumed on purpose.
- not verified: a real zellij tab and `claude` session (the tests use fakes)
- decisions needed: none
