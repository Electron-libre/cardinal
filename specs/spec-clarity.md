---
id: CLR
status: accepted
adrs: []
---

# Spec clarity checks

## Intent

`check_specs.rs` makes the ambiguities of a spec visible before the user validates it: an open question the model
marks instead of guessing, and the wording of a requirement that leaves its obligation vague. A marker blocks an
`accepted` spec; the wording checks only warn a `draft`, since a word list has false positives. A spec with no happy
path leaves its reader to rebuild the main flow from its requirements: the checker warns of it.

## Requirements

- **CLR-1** (deterministic) The checker must report a problem, and so exit 1, for each `[NEEDS CLARIFICATION:` marker of an `accepted` spec, naming the file and the line.
- **CLR-2** (deterministic) For each `[NEEDS CLARIFICATION: <question>]` marker of a `draft` spec, the checker must print a warning to stderr naming the file, the line and the question, without failing.
- **CLR-3** (deterministic) For each requirement of a `draft` spec whose statement holds one of the words `should`, `may`, `etc.`, `appropriate`, `fast` (whole word, any case), the checker must print a warning to stderr naming the requirement and the word, without failing.
- **CLR-4** (deterministic) For each requirement of a `draft` spec whose statement does not hold exactly one `must` (whole word, any case; `must not` counts as one), the checker must print a warning to stderr naming the requirement and the count, without failing.
- **CLR-5** (deterministic) The marker search of CLR-1 and CLR-2 and the word checks of CLR-3 and CLR-4 must ignore the text inside inline code spans (a run of backticks up to the next run of the same length, as in CommonMark) and inside code blocks (fenced or indented, inside a quote or a list item or not, as RMP-1 of `specs/roadmap.md` reads them).
- **CLR-6** (deterministic) The word checks of CLR-3 and CLR-4 must ignore the text inside double quotes and skip a withdrawn requirement.
- **CLR-7** (deterministic) The checker must not print a wording warning (CLR-3, CLR-4) for an `accepted` spec.
- **CLR-8** (deterministic) The word checks of CLR-3 and CLR-4 must read the statement of a requirement as the paragraph of its list item, as CommonMark reads it: its item line and the lines that continue it, up to the end of that paragraph (a blank line, a list item, a heading, a fence, a quote, an HTML block, a setext underline or a thematic break ends it).

- **CLR-9** (deterministic) For each `draft` spec that holds no happy path (TRC-17 of `specs/spec-tracing.md`), the checker must print a warning to stderr naming the file, without failing.
- **CLR-10** (deterministic) When `accepted` specs hold no happy path, the checker must print to stderr one warning naming their count and their files, without failing.

## Scenarios

- **CLR-H1** (happy; CLR-2, CLR-3, CLR-4, CLR-9) Given a `draft` spec whose requirements each hold one `must` and no
  vague word, with a happy path and no `[NEEDS CLARIFICATION: ...]` marker, when the checker runs, then it prints no
  warning for that spec and exits 0.

- (CLR-1) Given an `accepted` spec whose Scenarios hold `[NEEDS CLARIFICATION: which exit code?]`, when the checker runs, then it exits 1 with a line naming the spec and that line.
- (CLR-4) Given a `draft` requirement "The hook must exit 0 and must not print", when the checker runs, then it warns that the requirement holds 2 `must`.
- (CLR-5) Given a `draft` requirement citing `` `[NEEDS CLARIFICATION: x]` `` or `` `may_fail` ``, when the checker runs, then neither is reported.
- (CLR-6) Given a `draft` requirement "The hook must print \"you may retry\"", when the checker runs, then `may` is not reported.
- (CLR-5) Given a fenced block opened by four backticks that holds a line of three, when the checker runs, then the
  block ends only at the next line of four or more backticks.
- (CLR-6) Given a `draft` requirement with an unclosed quote (an inch mark), when the checker runs, then the text after
  it is still checked; `shouldn't` holds `should` and `mustn't` counts as one `must`.
- (CLR-8) Given a `draft` requirement "The hook, given a file," whose next line, indented, reads "must print its name,
  then may stop.", when the checker runs, then it warns of `may` and not of the count of `must`.
- (CLR-5) Given an `accepted` spec with a line ending in ``Write `the marker`` and the next one reading
  ``[NEEDS CLARIFICATION: x]` like this.``, when the checker runs, then the marker, inside a code span over two lines,
  is not reported.
- (CLR-8) Given a `draft` requirement whose item line holds one `must`, followed by a line `<div>` then a line that may
  stop, when the checker runs, then it warns of neither `may` nor the count of `must`.

- (CLR-9) Given a `draft` spec whose Scenarios hold only `- (CLR-1) Given …` items, or no item, when the checker runs,
  then it warns that the spec has no happy path and exits as the other checks give.
- (CLR-10) Given three `accepted` specs, two of them with no happy path, when the checker runs, then it prints one
  warning naming `2` and those two files; given none without one, it prints no such warning.

## Non-goals

- The format and traceability checks the checker already runs (front matter, sections, ids, citations): out of this
  spec, tested by `test_check_specs.rs`.
- A marker spread over several lines.
- A paragraph of a requirement after a blank line, or the content of a fenced block inside it: not read as its
  statement.
- Failing a spec with no happy path, or warning of more than three happy paths: a nudge, not a rule, so the specs of
  the consumers keep passing.
- Judging whether a requirement is complete or observable: that is the checklist of `skill:write-spec`, read by the
  model, not a check.
