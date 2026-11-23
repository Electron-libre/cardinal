---
id: NXT
status: accepted
adrs: [0019, 0021]
---

# Order of the open issues

## Intent

`next-issue` tells the agent and the user which open issue comes next and which ones can start now. Each issue file
declares its priority and the issues it depends on, and the order is derived from them and from the milestone order
of the roadmap, so a branch that creates, prioritizes or closes an issue edits only its own file and never conflicts
with another branch over a shared list (ADR 0019, ADR 0021).

## Requirements

### Behaviour

- **NXT-1** (deterministic) `next-issue` must order the open issues (the `*.md` files of `issues/`) that are ready (NXT-7) by the milestone of each (NXT-10), then by the `priority` of its front matter (the lines between a first line `---` and the next `---`, a value in quotes read without them), `high` before `medium` before `low`, then the oldest first (the date of the first commit that added the file under `issues/`, a file git does not track or outside a git repository after the tracked ones), then by stem, and print the first free one (PAR-4) with its title and the `why` of its front matter.
- **NXT-7** (deterministic) An open issue is ready when every stem of the `depends_on` of its front matter (`depends_on: [<stem>, <stem>...]` on one line, `[]` or no key for none) is an issue of `issues/done/`, else blocked: `next-issue` must never offer a blocked issue.
- **NXT-11** (deterministic) `next-issue` must list each blocked issue after the ready ones, in the order of NXT-1, on a line `Blocked: <stem> (by <stem>, <stem>...)` naming the stems that are not done.
- **NXT-15** (deterministic) An open issue whose front matter has an `on_hold` key is on hold, its reason the value of the key (unquoted as NXT-1 reads `why`, then trimmed): `next-issue` must never offer it but list it after the blocked ones, in the order of NXT-1, on a line `On hold: <stem> (<reason>)` instead of a `Blocked:` line, followed by `, blocked by <stem>, <stem>...` (the stems of NXT-11) when it is also blocked; an issue whose `depends_on` names it stays blocked (NXT-7).
- **NXT-18** (deterministic) With `--json`, `next-issue` must print instead of its text one JSON document (NXT-19) that a program reads: every open issue in the order of NXT-1, NXT-11 and NXT-15, with its state, what takes it, and the issue it offers, so a tool shows the queue without parsing a text written for the agent; it reports the problems on stderr and exits as without `--json`. An option other than `--json`, or a second directory, must be refused on stderr with exit code 2.

### Contract

- **NXT-2** (deterministic) An open issue whose front matter has no `priority`, or a `priority` other than `high`, `medium` and `low`, must be ordered after the `low` ones of its milestone, reported on stderr as `issue without a priority: <stem>` with exit code 1.
- **NXT-3** (deterministic) withdrawn: issues of the same milestone and priority are ordered by the tie-breaker of NXT-1, not reported.
- **NXT-4** (deterministic) withdrawn: the format 4 `issues/queue.md` is no longer read; every consumer ranks its issue files.
- **NXT-5** (deterministic) withdrawn: `next-issue --convert` is gone with the format 4 queue it converted.
- **NXT-6** (deterministic) withdrawn: no queue is left to hide a rank.
- **NXT-8** (deterministic) A stem of `depends_on` that is neither an open issue nor an issue of `issues/done/`, or a `depends_on` that is not a list of stems, must be reported on stderr as `unknown dependency of <stem>: <dependency>` or `invalid depends_on: <stem>` with exit code 1, the issue counting as blocked by it.
- **NXT-9** (deterministic) Open issues whose dependencies form a cycle must be reported on stderr once per group of issues that depend on each other as `dependency cycle: <stem>, <stem>...` (its stems sorted) with exit code 1, the issues of the cycle staying blocked.
- **NXT-10** (deterministic) The milestone of an issue is the first non-empty line of its `# Milestone` section, and its order the position of the `## <name>` heading outside code blocks and HTML comments (as RMP-1 of `specs/roadmap.md` reads them) of the same name in the `roadmap` documents of `.claude/cardinal.toml` (of the parent of the `issues` directory), read in the listed order: `next-issue` must order the issues of an earlier milestone first, and those with no milestone after those with one.
- **NXT-12** (deterministic) When the free issues are all blocked or on hold (NXT-15), `next-issue` must print `No ready issue: every free one waits on another.` instead of an issue, or `No ready issue: every free one is on hold or waits on another.` when one of them is on hold.
- **NXT-13** (deterministic) A milestone that no roadmap heading names must be reported on stderr as `unknown milestone of <stem>: <name>` with exit code 1, the issue ordered as one with no milestone.
- **NXT-14** (deterministic) With no `roadmap` key, or roadmap documents that cannot be read, `next-issue` must order the ready issues by priority then tie-breaker, with no milestone report.
- **NXT-16** (deterministic) An open issue whose milestone (NXT-10) has a field `On hold: <reason>` in its section (a line read as the `State:` line of RMP-1, its trailing HTML comment left out) must be on hold as by NXT-15, its reason `milestone <name>: <reason>`; the `on_hold` of the issue itself, when present, gives the reason instead.
- **NXT-17** (deterministic) An `on_hold` of an issue, or an `On hold:` of the milestone of an open issue, whose value is empty must be reported on stderr as `on hold without a reason: <stem>` or, once per milestone, `milestone on hold without a reason: <name>`, with exit code 1, the issue or the issues of the milestone staying on hold with the reason `no reason given` (an issue with an empty `on_hold` in a milestone on hold with a reason takes the reason of NXT-16).
- **NXT-19** (deterministic) The document of NXT-18 must be one object on one line with the fields `version` (the integer `1`, raised when a later revision removes or renames a field or changes the type of its value), `next` (the stem of the issue NXT-1 offers, `null` when it offers none: NXT-12, no open issue, every one taken) and `issues` (one object per open issue, in the order of NXT-18, with the fields of the table below).

| Field | Value |
|---|---|
| `stem` | the stem of the issue file |
| `title` | the first `# ` heading after its front matter, `null` when it has none |
| `goal` | the text of its `# Goal` section, up to the next line opening with `# ` outside a code block, its code blocks (fenced by ```` ``` ```` or `~~~`) left out, each paragraph its lines trimmed and joined by one space, a line opening a list item (`* `, `- `, or digits then `. `) starting a paragraph, the paragraphs joined by `"\n"`; `null` when it has no such section or the section holds no text |
| `priority` | `"high"`, `"medium"` or `"low"`; `null` for an issue NXT-2 reports |
| `why` | the `why` of its front matter, `""` when it has none |
| `milestone` | the name of its milestone (NXT-10), `null` when it has none or NXT-13 reports it unknown |
| `depends_on` | the stems of its `depends_on`, `[]` for none, `null` when NXT-8 reports it invalid |
| `state` | `"ready"`, `"blocked"` or `"on hold"` |
| `blocked_by` | the stems NXT-11 names, `[]` when none |
| `hold` | the reason of NXT-15 to NXT-17, `null` when it is not on hold |
| `taken` | `null`; else an object `{"by": "claim", "state": "working" or "merge pending", "worktree", "branch", "session_alive": true or false}` for a claim (CLM-5, CLM-6), or `{"by": "branch", "branch", "worktree": a path or null}` for a branch or worktree (PAR-4) |
| `stale` | `true` when the text output lists it `Stale:` (PAR-4), else `false` |

## Scenarios

- **NXT-H1** (happy; NXT-1, NXT-7, NXT-11) Given open issues of several priorities, one depending on another still
  open, when the agent asks `next-issue`, then it offers the first ready issue by priority and lists the dependent one
  as blocked by the other; once that one is closed, the dependent issue is offered.

- **NXT-H2** (happy; NXT-18, NXT-19) Given an open issue claimed by a session and another one ready, when a program
  runs `next-issue --json`, then it reads which issue comes next, the state and priority of each one and the
  worktree of the session that claimed the first, without parsing text.

- (NXT-1) Given `b.md` of priority `high` and `a.md` of priority `low`, both with no milestone, when `next-issue` runs, then it prints `b` as next.
- (NXT-1) Given `a.md` and `b.md` of priority `medium`, committed in that order, when `next-issue` runs, then it prints `a` as next.
- (NXT-2) Given `a.md` of priority `low` and `c.md` with no front matter, when `next-issue` runs, then it prints `a` as next, lists `c` after it, reports `issue without a priority: c` and exits 1.
- (NXT-2) Given `c.md` of priority `urgent`, when `next-issue` runs, then it reports `issue without a priority: c` and exits 1.
- (NXT-7) Given `a.md` of priority `high` with `depends_on: [b]` and `b.md` of priority `low`, when `next-issue` runs, then it prints `b` as next and `Blocked: a (by b)` (NXT-11); once `b.md` is in `issues/done/`, it prints `a` as next.
- (NXT-12) Given only `a.md` with `depends_on: [b]` and `b.md` with no `depends_on`, `b` taken by a worktree, when `next-issue` runs, then it prints `No ready issue: every free one waits on another.` and `Blocked: a (by b)`.
- (NXT-8) Given `a.md` with `depends_on: [gone]` and no `gone.md` in `issues/` nor `issues/done/`, when `next-issue` runs, then it reports `unknown dependency of a: gone`, lists `Blocked: a (by gone)` and exits 1.
- (NXT-8) Given `a.md` with `depends_on: b`, when `next-issue` runs, then it reports `invalid depends_on: a` and exits 1.
- (NXT-9) Given `a.md` with `depends_on: [b]` and `b.md` with `depends_on: [a]`, when `next-issue` runs, then it reports `dependency cycle: a, b` once, lists both as blocked and exits 1.
- (NXT-10) Given a roadmap with the headings `## One` then `## Two`, `a.md` of priority `high` in milestone `Two` and `b.md` of priority `low` in milestone `One`, when `next-issue` runs, then it prints `b` as next.
- (NXT-10) Given a roadmap with an HTML comment holding `## Two`, then the headings `## One` and `## Two`, `a.md` of priority `high` in milestone `Two` and `b.md` of priority `low` in milestone `One`, when `next-issue` runs, then it prints `b` as next.
- (NXT-13) Given `a.md` in milestone `Gone` that no roadmap heading names, when `next-issue` runs, then it reports `unknown milestone of a: Gone` and exits 1.
- (NXT-15) Given `a.md` of priority `high` with `on_hold: 'waits for the next model'` and `b.md` of priority `low`, when `next-issue` runs, then it prints `b` as next and `On hold: a (waits for the next model)`, and no `Blocked: a` line; once the key is removed, it prints `a` as next.
- (NXT-15) Given `a.md` on hold and `b.md` with `depends_on: [a]`, when `next-issue` runs, then it lists `Blocked: b (by a)` and `On hold: a (...)`.
- (NXT-12) Given only `a.md` on hold and `b.md` with `depends_on: [a]`, when `next-issue` runs, then it prints `No ready issue: every free one is on hold or waits on another.`
- (NXT-16) Given a roadmap with the headings `## One`, holding `- On hold: after the release <!-- the user -->`, and `## Two`, `a.md` in milestone `One` and `b.md` in milestone `Two`, when `next-issue` runs, then it prints `b` as next and `On hold: a (milestone One: after the release)`.
- (NXT-15) Given `a.md` with `on_hold: later` and `depends_on: [x]`, `x.md` open, when `next-issue` runs, then it lists `On hold: a (later), blocked by x`.
- (NXT-17) Given a milestone `Old` of `ROADMAP-done.md` with an empty `- On hold:` and no open issue in it, when `next-issue` runs, then it reports nothing of it.
- (NXT-17) Given `a.md` with `on_hold: ''`, when `next-issue` runs, then it reports `on hold without a reason: a`, lists `On hold: a (no reason given)` and exits 1.
- (NXT-14) Given no `.claude/cardinal.toml`, `a.md` of priority `low` in milestone `One` and `b.md` of priority `high`, when `next-issue` runs, then it prints `b` as next and exits 0.

- (NXT-18) Given `a.md` with no front matter, when `next-issue --json` runs, then its stdout is one JSON object whose `issues` lists `a` with `priority` `null`, its stderr reports `issue without a priority: a` and it exits 1.
- (NXT-19) Given `a.md` of priority `high` with `depends_on: [b]`, `b.md` of priority `low` and `c.md` with `on_hold: later`, when `next-issue --json` runs, then `next` is `"b"` and `issues` lists `b` (`ready`), `a` (`blocked`, `blocked_by` `["b"]`) and `c` (`on hold`, `hold` `"later"`), in that order.
- (NXT-19) Given `a.md` with `depends_on: b`, when `next-issue --json` runs, then `a` has `depends_on` `null` and `state` `"blocked"`.
- (NXT-19) Given only `a.md`, claimed by a session whose process has ended, when `next-issue --json` runs, then `next` is `null` and `a` has `taken` `{"by": "claim", "state": "working", ..., "session_alive": false}`.
- (NXT-19) Given `a.md` with a front matter and no heading, and `b.md` in milestone `Gone` that no roadmap heading names, when `next-issue --json` runs, then `a` has `title` `null` and `b` has `milestone` `null`.
- (NXT-19) Given `a.md` whose `# Goal` holds two lines, an empty line and a third line before `# Findings`, when `next-issue --json` runs, then `a` has `goal` the first two lines joined by a space, `\n`, then the third.
- (NXT-19) Given a `# Goal` that holds a code block with a line `# install`, a list of two items and a `## ` heading, then its `goal` holds no line of the code block, each item on a line of its own, and the `## ` line in the text of its paragraph.
- (NXT-19) Given `a.md` with no `# Goal` section, and `b.md` whose `# Goal` is followed at once by `# Findings`, then both have `goal` `null`.
- (NXT-18) Given `a.md`, when `next-issue --jsn` or `next-issue issues other` runs, then it prints nothing on stdout and exits 2.
- (NXT-19) Given no open issue, when `next-issue --json` runs, then it prints `{"version":1,"next":null,"issues":[]}`.

## Non-goals

- A schema file or a JSON form of the reports of stderr: the program reads the exit code and, when it needs them, the lines of stderr.

- Reading the `rank` of a format 5 issue: an issue without a priority is reported (NXT-2) until `skill:prioritize-issues` gives it one.
- Checking that the issues a roadmap milestone lists are those that name it: the `# Milestone` section of the issue is the source of the order.
- Dependencies on anything but an issue (a measure, a number of sessions): the `why` and the Hints state them.
- A tool that puts an issue or a milestone on hold: the user, or the agent on their request, writes the key or the field and commits it like any edit of the issue or the roadmap.
- Refusing to claim or start an issue on hold: `next-issue` never offers one, and `issue-workflow` warns when the user names one.
- Validating the priority and the dependencies of a new issue file: `check-issue` does it (`specs/check-issue.md`).
