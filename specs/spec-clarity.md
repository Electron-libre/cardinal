---
id: CLR
status: accepted
adrs: [0029]
---

# Spec clarity checks

## Intent

`check_specs.rs` makes visible, before the user validates a spec, what would make it hard to read: an open question,
a vague obligation, a missing happy path or behaviour, and a spec grown past what one capability holds. It fails
only on a marker in an `accepted` spec or a bound it cannot read: a word list or a bound has false positives.

## Requirements

### Behaviour

- **CLR-1** (deterministic) The checker must report a problem, and so exit 1, for each `[NEEDS CLARIFICATION:` marker of an `accepted` spec, naming the file and the line.
- **CLR-2** (deterministic) For each `[NEEDS CLARIFICATION: <question>]` marker of a `draft` spec, the checker must print a warning to stderr naming the file, the line and the question, without failing.
- **CLR-9** (deterministic) For each `draft` spec that holds no happy path (TRC-17 of `specs/spec-tracing.md`), the checker must print a warning to stderr naming the file, without failing.
- **CLR-10** (deterministic) When `accepted` specs hold no happy path, the checker must print to stderr one warning naming their count and their files, without failing.
- **CLR-11** (deterministic) For each `draft` spec whose `## Requirements` holds no `### Behaviour` heading (or `### Behavior`, at the start of its line), the checker must print a warning to stderr naming the file, without failing.
- **CLR-12** (deterministic) When `accepted` specs hold no `### Behaviour` heading under `## Requirements`, the checker must print to stderr one warning naming their count and their files, without failing.
- **CLR-13** (deterministic) For each `draft` spec over one of the bounds of CLR-15, the checker must print a warning to stderr naming the file and, for each bound passed, its measure and the bound, without failing.
- **CLR-14** (deterministic) When `accepted` specs are over one of the bounds of CLR-15, the checker must print to stderr one warning naming their count and their files, without failing.

### Contract

- **CLR-3** (deterministic) For each requirement of a `draft` spec whose statement holds one of the words `should`, `may`, `etc.`, `appropriate`, `fast` (whole word, any case), the checker must print a warning to stderr naming the requirement and the word, without failing.
- **CLR-4** (deterministic) For each requirement of a `draft` spec whose statement does not hold exactly one `must` (whole word, any case; `must not` counts as one), the checker must print a warning to stderr naming the requirement and the count, without failing.
- **CLR-5** (deterministic) The marker search of CLR-1 and CLR-2 and the word checks of CLR-3 and CLR-4 must ignore the text inside inline code spans (a run of backticks up to the next run of the same length, as in CommonMark) and inside code blocks (fenced or indented, inside a quote or a list item or not, as RMP-1 of `specs/roadmap.md` reads them).
- **CLR-6** (deterministic) The word checks of CLR-3 and CLR-4 must ignore the text inside double quotes and skip a withdrawn requirement.
- **CLR-7** (deterministic) The checker must not print a wording warning (CLR-3, CLR-4) for an `accepted` spec.
- **CLR-8** (deterministic) The word checks of CLR-3 and CLR-4 must read the statement of a requirement as the paragraph of its list item, as CommonMark reads it: its item line and the lines that continue it, up to the end of that paragraph (a blank line, a list item, a heading, a fence, a quote, an HTML block, a setext underline or a thematic break ends it).

- **CLR-15** (deterministic) The checker must read the bounds from the `[project]` table of `.claude/cardinal.toml`, each defaulting when the file or the key is absent, and measure a spec against each as the table below gives.
- **CLR-16** (deterministic) The checker must report a problem, and so exit 1, naming the key and the file, for a bound of CLR-15 that is not a positive integer.

| key | default | measure of a spec |
|---|---|---|
| `spec_max_requirements` | `40` | its requirements under `## Requirements`, withdrawn ones excluded, happy paths excluded |
| `spec_max_bytes` | `25000` | the bytes of its file |
| `spec_max_intent` | `600` | the characters between its `## Intent` heading and the next `## ` heading, trimmed |

A spec over a bound is one whose measure is greater than it.

## Scenarios

- **CLR-H1** (happy; CLR-2, CLR-3, CLR-4, CLR-9, CLR-11) Given a `draft` spec whose requirements each hold one `must`
  and no vague word, under a `### Behaviour` heading, with a happy path and no `[NEEDS CLARIFICATION: ...]` marker,
  when the checker runs, then it prints no warning for that spec and exits 0.

- **CLR-H2** (happy; CLR-13, CLR-15) Given a `draft` spec amended to 41 requirements in a project that sets no bound,
  when the checker runs, then it warns that the spec holds 41 requirements over the bound of 40, and exits 0; given a
  `spec_max_requirements = 60`, it prints no such warning.

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

- (CLR-11) Given a `draft` spec whose requirements sit directly under `## Requirements`, or whose `### Behaviour` sits
  under `## Scenarios`, or is in a code block, when the checker runs, then it warns that the spec has no behaviour.
- (CLR-11) Given a `draft` spec with `### Contract` before `### Behaviour`, or with `### Behaviour` and no
  `### Contract`, when the checker runs, then it prints no behaviour warning.
- (CLR-11) Given a `draft` spec with `### Behavior` or `### Behaviour ###`, when the checker runs, then it prints no
  behaviour warning; given a `  ### Behaviour` indented in a requirement item, it does.
- (CLR-11) Given a `draft` spec with no `## Requirements`, when the checker runs, then it fails on the missing section
  and prints no behaviour warning.
- (CLR-12) Given three `accepted` specs, two of them with no `### Behaviour`, when the checker runs, then it prints one
  warning naming `2` and those two files.

- (CLR-13) Given a `draft` spec of exactly 40 requirements, 25000 bytes and an Intent of 600 characters, when the
  checker runs, then it prints no bound warning; given one of 26000 bytes and an Intent of 700 characters, it prints
  one warning naming both measures and both bounds.
- (CLR-15) Given a `draft` spec of 45 requirements, 5 of them withdrawn, and three happy paths, when the checker runs,
  then it counts 40 requirements and prints no bound warning.
- (CLR-14) Given three `accepted` specs, two of them over a bound, when the checker runs, then it prints one warning
  naming `2` and those two files.
- (CLR-16) Given `spec_max_bytes = 0` or `spec_max_bytes = "25 KB"`, when the checker runs, then it exits 1 with a
  line naming `spec_max_bytes`.

## Non-goals

- The format and traceability checks the checker already runs (front matter, sections, ids, citations): out of this
  spec, tested by `test_check_specs.rs`.
- A marker spread over several lines.
- A paragraph of a requirement after a blank line, or the content of a fenced block inside it: not read as its
  statement.
- Failing a spec with no `### Behaviour`, checking the order of `### Behaviour` and `### Contract`, or that a
  behaviour requirement is written in the terms of the product: the README and `skill:write-spec` ask for it, a reader
  judges it.
- Failing a spec with no happy path, or warning of more than three happy paths: a nudge, not a rule, so the specs of
  the consumers keep passing.
- Failing a spec over a bound, or splitting it: `skill:write-spec` proposes the split, the user decides.
- Bounds per spec, or a bound on another section than the Intent.
- Judging whether a requirement is complete or observable: that is the checklist of `skill:write-spec`, read by the
  model, not a check.
