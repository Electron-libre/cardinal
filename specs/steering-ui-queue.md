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

### Contract

- **SUQ-3** (deterministic) An issue row of the queue must read `> ` for the issue `next-issue` offers, else two spaces, then the tree of SUQ-1 (per level above its own, `│  ` when that level has an issue after it, else three spaces; then `├─ ` when its own level has an issue after it, else `└─ `; nothing at the root), its stem, its priority (`no priority` when it has none), its state, then what takes it, `tab` (STU-10) and the words of SUQ-2, separated by two spaces; the priority, the state and what follows it each start at one display column on every issue row of the panel, that of the longest cell before it plus two.

## Scenarios

- **SUQ-H1** (happy; SUQ-1, SUQ-2, SUQ-3) Given the milestone `One` listed by `roadmap` with the open issues
  `a`, `b` that depends on `a`, and `c` that depends on `a` and `b`, and an open issue `d` of no milestone, when the
  user runs `cardinal-ui --print`, then the queue reads the heading of `One` with `3/3 open`, `a` at its root, `b`
  under it with `├─ `, `c` under it with `└─ ` and `also after b`, then `No milestone` with `d`, the priorities of
  the four issues in one column.
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
- (SUQ-3) Given a stem of 40 characters at the third level and one of 3 at the root, then the priorities of both
  start at one column.

## Non-goals

- Folding a section or a subtree, drawing an issue under several parents, or showing the closed issues an open one
  depends on.
