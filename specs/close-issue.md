---
id: CLS
status: accepted
adrs: []
---

# Closing an issue

## Intent

`close-issue` moves a finished issue to `issues/done/`; the documents of the repository that cite it must still
point at it afterwards, whether they name it from the root or by a relative Markdown link. The agent and the user
rely on it at every close, and `--relink` repairs the references to issues closed before.

## Requirements

- **CLS-1** (deterministic) After `close-issue issues/<stem>.md`, every path `issues/<stem>.md` that starts a word (after the start of the text, a space or one of `` `"'([<{=,: ``) in a text file of the repository must read `issues/done/<stem>.md`.
- **CLS-2** (deterministic) After `close-issue issues/<stem>.md`, the target of every Markdown link of a `.md` file (`[text](target)` or `[label]: target`) that, resolved against the directory of that file, is the closed issue file must point at the issue in `issues/done/`, keeping its relative form and its `#anchor` (`../issues/foo.md#goal` becomes `../issues/done/foo.md#goal`).
- **CLS-3** (deterministic) `close-issue` must not rewrite a link target that holds a directory before `issues/` (`../issues/foo.md`, `docs/issues/foo.md`) when that target, resolved against the directory of its file, is not the closed issue file.
- **CLS-4** (deterministic) After `close-issue issues/<stem>.md`, every relative Markdown link target of the moved file must designate from `issues/done/` the file or directory it designated from `issues/`, keeping its `#anchor` and its trailing `/`.
- **CLS-5** (deterministic) `close-issue --relink` must rewrite the references of CLS-1 and CLS-2 for every issue file in `issues/done/`.
- **CLS-6** (deterministic) `close-issue` must not rewrite a Markdown link written in an inline code span or a fenced code block, nor the text of a footnote (`[^1]: text`).
- **CLS-7** (deterministic) After moving an issue, `close-issue` must print on stdout, as its last line or lines, the next issue `next-issue` would offer once the closed one is gone (`Next: issues/<stem>.md` and its title, NXT-1, NXT-7 and PAR-4), or `No open issue.`, `Every open issue is taken.` or `No ready issue: every free one waits on another.` when there is none; `--relink` prints none.

## Scenarios

- (CLS-2) Given `docs/a.md` holding `[a](../issues/foo.md)` and `docs/adr/b.md` holding `[b](../../issues/foo.md)`, when `close-issue issues/foo.md` runs, then they read `[a](../issues/done/foo.md)` and `[b](../../issues/done/foo.md)`.
- (CLS-2) Given `issues/bar.md` holding `[foo](foo.md)`, when `close-issue issues/foo.md` runs, then it reads `[foo](done/foo.md)`.
- (CLS-3) Given `README.md` at the root holding `[x](../issues/foo.md)` (outside the repository) and `[y](docs/issues/foo.md)`, when `close-issue issues/foo.md` runs, then both are unchanged.
- (CLS-4) Given `issues/foo.md` holding `[bar](bar.md)`, `[q](gone.md#top)` and `[old](done/old.md)`, when `close-issue issues/foo.md` runs, then `issues/done/foo.md` holds `[bar](../bar.md)`, `[q](../gone.md#top)` and `[old](old.md)`.
- (CLS-4) Given `issues/foo.md` holding `[c](done/)` and `[o](../../out.md)` (outside the repository), when `close-issue issues/foo.md` runs, then `issues/done/foo.md` holds `[c](./)` and `[o](../../../out.md)`.
- (CLS-6) Given `issues/foo.md` holding `` `[e](bar.md)` `` and a fenced block with `[f](bar.md)`, when `close-issue issues/foo.md` runs, then both are unchanged in `issues/done/foo.md`.
- (CLS-7) Given `issues/foo.md` of priority `high` and `issues/bar.md` of priority `low` titled `Bar`, when `close-issue issues/foo.md` runs, then its output ends with `Next: issues/bar.md` and `Bar`; closing `issues/bar.md` then ends with `No open issue.`.
- (CLS-1) Given `docs/a.md` holding `` `issues/foo.md` `` (a path from the root), when `close-issue issues/foo.md` runs, then it reads `` `issues/done/foo.md` ``.

## Non-goals

- Links in HTML (`<a href>`), in files other than Markdown beyond the root form of CLS-1, and URLs (`https://...`).
- Bare file names outside a Markdown link (`` `foo.md` `` in prose).
- Checking that the other links of the repository resolve.
