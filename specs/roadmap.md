---
id: RMP
status: accepted
adrs: []
---

# State of the milestones

## Intent

`roadmap` shows at a glance which milestones of the `roadmap` documents are open and how far their issues went, so
the user and the agent need not read every section of the roadmap. It reads the documents and writes nothing.

## Requirements

- **RMP-1** (deterministic) `roadmap` must print one row per milestone (a `## <name>` heading outside fenced code blocks and HTML comments, its section read without the lines inside them, REL-9) of the `roadmap` documents of `.claude/cardinal.toml` of the current directory, in the order of the documents then of the milestones in each, as a Markdown table of the columns `milestone`, `state`, `released`, `issues` and `document`: its name, the value of its first `State:` line, the value of its `Released:` line (empty when it has none), the issues of RMP-2 and the document as the configuration writes it.
- **RMP-2** (deterministic) The `issues` cell of a milestone must read `<open>/<listed> open`, where `<listed>` counts the links of its `Issues:` list to the `issues` directory (a URL, an anchor or another document aside, resolved against the directory of the document as CLS-8 resolves them) and `<open>` those of them that do not designate an existing file directly under `issues/done/`; a milestone with no such link reads an empty cell.
- **RMP-3** (deterministic) A milestone with no `State:` line must show `(none)` in its `state` cell, and one whose state is not `idea`, `shaped`, `in issues` or `done` its value followed by ` (unknown)`; neither is skipped.
- **RMP-4** (deterministic) With `--open`, `roadmap` must leave out the milestones that have a `Released:` line.
- **RMP-5** (deterministic) When no milestone is left to print (no `roadmap` document readable, none holding a milestone, or every one left out by `--open`), `roadmap` must print `No milestone.` instead of the table; a document that cannot be read is left out, `roadmap` exits 0 and changes no file.

## Scenarios

- (RMP-1) Given `roadmap = ["ROADMAP.md", "ROADMAP-done.md"]`, `ROADMAP.md` holding `Next` (`in issues`) and `ROADMAP-done.md` holding `Old` (`done`, released `v1.0.0`), when `roadmap` runs, then it prints the row of `Next` before the row of `Old`, the latter reading `v1.0.0` and `ROADMAP-done.md`.
- (RMP-2) Given milestone `Next` listing `issues/done/a.md` (existing), `issues/b.md`, `https://example.com/c.md` and `docs/d.md`, when `roadmap` runs, then its `issues` cell reads `1/2 open`.
- (RMP-3) Given milestone `Draft` with no `State:` line and milestone `Odd` with `- State: paused`, when `roadmap` runs, then their state cells read `(none)` and `paused (unknown)`.
- (RMP-4) Given `Old` released and `Next` not, when `roadmap --open` runs, then only `Next` is printed.
- (RMP-5) Given a configuration naming a missing `ROADMAP-done.md` and a `ROADMAP.md` with no milestone, when `roadmap` runs, then it prints `No milestone.` and exits 0.

## Non-goals

- Checking the milestones: a wrong state is shown, not reported as an error, and the exit code says nothing of it.
- Dates, goals or the titles of the issues: the row stays one line.
- Changing a state: `close-issue` sets `done` (CLS-8), `release` sets `Released:` (REL-9).
