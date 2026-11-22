---
id: SUQ
status: accepted
adrs: [0030]
---

# Steering UI queue

## Intent

The queue of `cardinal-ui` (STU-1) groups the open issues under their milestone and draws each one under the issue
it depends on, in aligned columns, so the order of the work reads at a glance.

## Requirements

### Behaviour

- **SUQ-1** (deterministic) The queue must show one section per milestone that `roadmap --open` lists, in its order, headed by its line of STU-1, then one per other milestone an open issue names, in the order of `next-issue`, headed by its name, then `No milestone` for the open issues without one, when there are any; under its heading, the open issues of a section as a tree: an issue under the first stem of its `depends_on` that is an open issue of the same section, at the root when there is none or when the issues it would be under lead back to it (a cycle of NXT-9), the issues of one level in the order of `next-issue`. When `roadmap` or `release` cannot be read, the line of STU-6 heads the panel and each milestone an open issue names gets a section headed by its name, in the order of `next-issue`; when `next-issue` cannot be read, its line heads the panel above the headings of the milestones.
- **SUQ-2** (deterministic) An issue must name, after its row, `also after <stem>, <stem>...` the open issues of its `depends_on` other than the one it is drawn under, in the order of its `depends_on`, so no open dependency is hidden.
- **SUQ-4** (deterministic) An issue claimed by a session at work must show a mark that turns, and one claimed by a session that waits for the user a mark that stays still, between its tree and its stem, so the eye finds them at once: a claim (STU-1) other than `merge pending` whose session has not ended is waiting when the worktree of STU-7 at the path of the claim waits (STU-13), else at work; no issue has a mark while the worktrees cannot be read (STU-6). On the screen the mark at work is one frame of `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` in `accent`, the next one every 100 ms whether a read ended or not, and the waiting mark `‖` in `warn`; `cardinal-ui --print` writes `⠿` at work and `‖` waiting, so its output stays the same from one run to the next.
- **SUQ-5** (deterministic) The heading of a section must read apart from the issue rows: `■ ` before its line of SUQ-1, an empty line above every heading but the first of the panel, and on the screen the whole heading in `accent` and bold, the words of SUS-5 it holds (`(on hold)`, `awaits release`) in their own role and bold; `--print` keeps the `■ ` and the empty lines, so its text tells a milestone from an issue without colors.

### Contract

- **SUQ-3** (deterministic) An issue row of the queue must read `> ` for the issue `next-issue` offers, else two spaces, then the tree of SUQ-1 (per level above its own, `│  ` when that level has an issue after it, else three spaces; then `├─ ` when its own level has an issue after it, else `└─ `; nothing at the root), the mark of SUQ-4 and a space when it has one, its stem, its priority (`no priority` when it has none), its state, then what takes it, `tab` (STU-10) and the words of SUQ-2, separated by two spaces; the priority, the state and what follows it each start at one display column on every issue row of the panel, that of the longest cell before it plus two.

## Scenarios

- **SUQ-H1** (happy; SUQ-1, SUQ-2, SUQ-3, SUQ-5) Given the milestone `One` listed by `roadmap` with the open issues
  `a`, `b` that depends on `a`, and `c` that depends on `a` and `b`, and an open issue `d` of no milestone, when the
  user runs `cardinal-ui --print`, then the queue reads the heading `■ One` with `3/3 open`, `a` at its root, `b`
  under it with `├─ `, `c` under it with `└─ ` and `also after b`, then an empty line and `■ No milestone` with `d`, the priorities of
  the four issues in one column.
- **SUQ-H2** (happy; SUQ-4) Given `a` claimed by a session at work in a linked worktree and `b` claimed by a session
  whose turn ended in another, when the user runs `cardinal-ui --print`, then the row of `a` reads `⠿ a` and that of
  `b` reads `‖ b`; on the screen, the mark of `a` changes from one tick to the next with no read between them, and
  that of `b` does not.
- (SUQ-1) Given an open issue whose only dependency is closed, then it is at the root of its section.
- (SUQ-1) Given `x` of milestone `Two` that depends on the open `a` of `One`, then `x` is at the root of `Two` with
  `also after a`.
- (SUQ-1) Given `a` and `b` of one milestone that depend on each other, then both are at its root, each `also
  after` the other.
- (SUQ-1) Given an open issue of the milestone `Ghost` that `roadmap` does not list, then `Ghost` is a section after
  those of `roadmap`, headed by its name alone.
- (SUQ-1) Given a milestone `done` whose issues are all closed, then its heading is shown with no issue under it.
- (SUQ-3) Given `a` with the children `b` then `c`, and `e` the child of `b`, then the row of `e` opens with
  `│  └─ `; with no `c`, it opens with three spaces then `└─ `.
- (SUQ-4) Given an issue claimed by a session whose process ended, or in `merge pending`, or taken by a branch
  alone, then its row has no mark.
- (SUQ-4) Given a wait mark that names another session than the lock of the worktree (STU-13), then the issue reads
  the mark at work.
- (SUQ-4) Given a live claim and a `git worktree list` that fails, then its row has no mark.
- (SUQ-4) Given a claim whose worktree git no longer lists, then the issue reads the mark at work.
- (SUQ-3) Given an issue with the mark of SUQ-4 and one without, then their priorities start at one column.
- (SUQ-3) Given a stem of 40 characters at the third level and one of 3 at the root, then the priorities of both
  start at one column.
- (SUQ-5) Given the line of STU-6 of `roadmap` above the first heading, then no empty line comes between them.
- (SUQ-5) Given a milestone `(on hold)` on the screen, then its name is in `accent` and bold, `(on hold)` in `warn` and bold.

## Non-goals

- A rule across the panel or a frame around a section: the mark, the empty line and the role tell the heading apart.
- Animating anything else than the mark of SUQ-4 (a worktree row, the measures), or a mark for the worktrees panel:
  its `waiting since` already reads the wait.
- Telling a wait without the mark of SWT-3: in a project that turns off the `session-wait` hooks, a live claim reads
  at work.
- Folding a section or a subtree, drawing an issue under several parents, or showing the closed issues an open one
  depends on.
