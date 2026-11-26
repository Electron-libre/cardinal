---
id: SUQ
status: draft
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
- **SUQ-6** (deterministic) On the panels, with the selection in the queue (STU-14) on an issue or on the heading of a milestone, the key `i` must unfold the summary of the selected row and, pressed again, fold it: a panel under the queue, above the worktrees; for an issue, titled with its `title` (its stem when it has none) and holding its `goal` (NXT-19), `no goal` when it is `null`; for a milestone, titled with its name and holding three paragraphs, `state: <state>` (`(none)` when it has none) followed by `, on hold: <reason>` when it is on hold (`, on hold` when the reason is empty), `goal: <goal>` and `why: <why>` (RMP-8), `no goal` and `no why` when they are `null`; the summary is folded when the screen starts, stays unfolded or folded across the reads and the moves of the selection, from an issue to a heading and back, showing the row selected, and is not drawn while the selection is in the worktrees or the queue holds no issue nor heading the selection stops on (an empty queue, or a `next-issue` that cannot be read and no milestone, STU-6); `--print` shows it folded.

### Contract

- **SUQ-3** (deterministic) An issue row of the queue must read `> ` for the issue `next-issue` offers, else two spaces, then the tree of SUQ-1 (per level above its own, `│  ` when that level has an issue after it, else three spaces; then `├─ ` when its own level has an issue after it, else `└─ `; nothing at the root), the mark of SUQ-4 and a space when it has one, its stem, its priority (`no priority` when it has none), its state, then what takes it, `tab` (STU-10) and the words of SUQ-2, separated by two spaces; the priority, the state and what follows it each start at one display column on every issue row of the panel, that of the longest cell before it plus two.
- **SUQ-7** (deterministic) The summary of SUQ-6 must show its text (the goal of an issue, the paragraphs of a milestone) wrapped at the words to the inner width of its panel, a word wider than that cut across the rows, each paragraph from a row of its own, on the rows it needs plus its two borders, at most a third of the height of the screen and no more than the other panels, the status line and the help leave it (one row of text at least), its last row ending with `…` when the text needs more, the queue keeping the other rows.

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
- **SUQ-H3** (happy; SUQ-6, SUQ-7) Given the issue `a` titled `Do a` selected, whose `# Goal` reads two sentences,
  when the user presses `i`, then a panel titled `Do a` under the queue reads the two sentences; when they press `↓`
  to the issue `b`, it reads the title and the goal of `b`; when they press `i` again, the panel is gone.
- **SUQ-H4** (happy; SUQ-6, SUQ-7) Given the heading of the milestone `One` selected, `- State: in issues`,
  `- On hold: after v2`, a `- Goal:` and a `- Why:` line, when the user presses `i`, then a panel titled `One` under
  the queue reads `state: in issues, on hold: after v2`, then `goal: ` and its goal, then `why: ` and its why, each
  from a row of its own; when they press `↓` to its first issue, it reads the title and the goal of that issue.
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
- (SUQ-6) Given the summary unfolded, when the user presses `Tab`, then no summary is drawn; when they press `Tab`
  again, it shows the issue selected in the queue.
- (SUQ-6) Given the summary unfolded, when a read of STU-2 ends, then it is still unfolded.
- (SUQ-6) Given the summary of an issue unfolded, when the user presses `↑` to the heading of its milestone, then the
  panel shows the milestone, still unfolded.
- (SUQ-6) Given a milestone with no `State:`, `Goal:` nor `Why:` line selected, when the summary is unfolded, then its
  panel reads `state: (none)`, `no goal` and `no why`.
- (SUQ-6) Given a `next-issue` that cannot be read and the milestone `One` listed by `roadmap`, when the user presses
  `i` on its heading, then the summary of `One` is drawn.
- (SUQ-6) Given an issue with no `# Goal` section selected, when the summary is unfolded, then its panel reads `no goal`.
- (SUQ-6) Given the charts of SUC-1 shown, when the user presses `i`, then nothing changes.
- (SUQ-7) Given a screen 15 rows high and a goal that needs 10 rows at the width of its panel, then the panel shows
  5 rows of text, the last ending with `…`.
- (SUQ-7) Given a screen 15 rows high whose worktrees panel needs 10 rows and a goal of 20 rows, then the last row
  the summary shows ends with `…`.
- (SUQ-7) Given a goal of two paragraphs that fit on one row each, then the panel shows them on two rows.

## Non-goals

- A rule across the panel or a frame around a section: the mark, the empty line and the role tell the heading apart.
- Animating anything else than the mark of SUQ-4 (a worktree row, the measures), or a mark for the worktrees panel:
  its `waiting since` already reads the wait.
- Telling a wait without the mark of SWT-3: in a project that turns off the `session-wait` hooks, a live claim reads
  at work.
- Folding a section or a subtree, drawing an issue under several parents, or showing the closed issues an open one
  depends on.
- The goal of an issue as the branch of a linked worktree words it: the summary reads the issue file of the work tree
  `cardinal-ui` reads (STU-5), the one of the trunk when it runs in the main worktree.
- Showing other sections of the issue (findings, approach, the plan of its `# Tracking`) or rendering their Markdown.
