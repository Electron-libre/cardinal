---
id: TRC
status: accepted
adrs: []
---

# Boundary proofs of the specs

## Intent

A spec and the tests that trace it are meant to survive a rewrite of the code, so a requirement is proven by a test
that drives the program from its boundary (command line, files, exit code, output). `check-specs` tells such a test
from an internal one by where it sits: a test under a `src` directory calls the code from inside; a test elsewhere
(`tests/`, the managed tests under `.claude/`) drives the boundary. A boundary proof is preferred when it is cheap and
stable, not required: an internal test is the proof when its citation gives the reason, `// spec: ABC-1 (internal:
<reason>)`. The checker reports the requirements whose internal proofs give no reason, as a warning. A spec opens its
scenarios with its happy paths, the main flows of the capability, each with an id a test cites like a requirement's.

## Requirements

- **TRC-1** (deterministic) For each spec holding requirements cited only by tests under a `src` directory, one of those citations at least giving no reason (TRC-6), the checker must print one warning to stderr naming the spec file, the count and the ids of those requirements, without failing.
- **TRC-2** (deterministic) The checker must not list in that warning a requirement that a test outside every `src` directory also cites.
- **TRC-3** (deterministic) The checker must not list in that warning a requirement that no test cites.
- **TRC-4** (deterministic) The checker must not list in that warning a withdrawn requirement.
- **TRC-5** (deterministic) The checker must count a test under `.claude/` as outside every `src` directory, whatever its path below.
- **TRC-6** (deterministic) The checker must not list in that warning a requirement whose every citation under a `src` directory carries a reason, the text `(internal: <reason>)` after its ids with a reason that is not blank.
- **TRC-7** (deterministic) The checker must report as a problem, naming its file and line, a citation whose `(internal: ...)` reason is blank, does not end the line with its `)`, or follows no id.
- **TRC-8** (deterministic) The checker must read from a citation with a reason the same requirement ids as from that citation without it.
- **TRC-9** (deterministic) The checker must read the citations of every `.rs` file under the root, whatever its directory (a crate at the root, `src/` and `tests/`, as well as `crates/` and `.claude/`), except under a directory named `.git` or `node_modules`, or a `target` directory beside a `Cargo.toml` (the build output).
- **TRC-10** (deterministic) The checker must not read the `.rs` files under a directory named like one of the `skipped_dirs` of `.claude/cardinal.toml`.
- **TRC-11** (deterministic) When `.claude/cardinal.toml` sets `spec_files`, the checker must read, instead of the `.rs` files, the files whose name matches one of its patterns, with the exclusions of TRC-9 and TRC-10; a pattern matches the file name alone, `*` standing for any run of characters and every other character for itself.
- **TRC-12** (deterministic) The checker must read as a citation a line whose trimmed text starts with the line-comment marker of its file's extension followed by ` spec:`: `//` for `rs`, `ts`, `tsx`, `js`, `jsx`, `mjs`, `cjs`, `go`, `java`, `kt`, `kts`, `scala`, `swift`, `c`, `cc`, `cpp`, `h`, `hpp`, `cs`, `dart` and `php`; `#` for `py`, `sh`, `bash`, `zsh`, `bats`, `rb`, `pl`, `r` (either case), `ex`, `exs` and `nix`; `--` for `lua`, `sql`, `hs` and `elm`; `;`, once or more, for `clj`, `cljs`, `lisp`, `el`, `scm` and `rkt`; any of the four for another extension.
- **TRC-13** (deterministic) The checker must report as a problem, naming its file and line, a citation of a file other than a `.rs` one whose next line, past the comment lines (any marker of TRC-12) and the decorator lines (trimmed text starting with `@`), is blank or missing; a citation of a `.rs` file keeps the rule of `specs/README.md`, a `fn` past the attributes and comments.
- **TRC-14** (deterministic) When `.claude/cardinal.toml` sets `internal_dirs`, the checker must count as calling the code from inside (TRC-1 to TRC-6) the tests under a directory named like one of its entries, instead of those under a `src` directory.
- **TRC-15** (deterministic) The checker must report as a problem a pattern of `spec_files` or an entry of `internal_dirs` that holds a `/` or a `\`, naming it.
- **TRC-16** (deterministic) The checker must report as a problem a `spec_files` or an `internal_dirs` of `.claude/cardinal.toml` that is not a list of strings, naming the key.
- **TRC-17** (deterministic) The checker must read as a happy path each list item of the `## Scenarios` section that opens with `- **<ID>-H<n>** (happy; <ids>)`, `<ID>` the id of the spec, `<n>` a number and `<ids>` the requirement ids the path walks through, separated by commas.
- **TRC-18** (deterministic) The checker must report as a problem, naming the file and the happy path, a happy path whose list names no id, an id that is not a requirement of the same spec, or a withdrawn requirement.
- **TRC-19** (deterministic) The checker must report as a problem, naming the file and line, a list item of `## Scenarios` that opens with `- (happy` or with `- **<text>** (happy` and that TRC-17 does not read as a happy path.
- **TRC-20** (deterministic) The checker must report as a problem, naming the file and the happy path, a happy path that follows another list item of `## Scenarios` that is not a happy path.
- **TRC-21** (deterministic) The checker must report as a problem, naming the file, a happy path id defined twice.
- **TRC-22** (deterministic) The checker must read a citation of a happy path id as citing that happy path, not as an unknown id.
- **TRC-23** (deterministic) The checker must report as a problem, naming the file and the id, a happy path of an `accepted` spec that no test cites.
- **TRC-24** (deterministic) The checker must list in the warning of TRC-1 a happy path cited only by tests under a `src` directory, one of those citations at least giving no reason, as it lists such a requirement.

## Scenarios

- **TRC-H1** (happy; TRC-9, TRC-17, TRC-22) Given a spec whose requirements and happy path are each cited by a test
  under `tests/`, when the checker runs, then it exits 0 and prints no warning for that spec.
- **TRC-H2** (happy; TRC-6, TRC-8) Given a requirement cited only in `src/a.rs` by
  `// spec: SYN-2 (internal: needs a fake clock)`, when the checker runs, then the requirement is traced and the
  boundary warning does not name it.
- (TRC-1) Given `SYN-2` cited only in `crates/cardinal/src/tests.rs`, when the checker runs, then stderr holds one line
  naming `specs/sync.md`, `1` and `SYN-2`, and the exit code is the one the other checks give.
- (TRC-2) Given `SYN-1` cited in `crates/cardinal/src/tests.rs` and in `crates/cardinal/tests/sync_cli.rs`, when the
  checker runs, then `SYN-1` is not in the warning.
- (TRC-1) Given a spec whose cited requirements all have a test outside `src`, when the checker runs, then it prints
  no boundary warning for that spec.
- (TRC-6) Given `SYN-2` cited in `crates/cardinal/src/a.rs` with `// spec: SYN-2 (internal: needs a fake clock)` and in
  `crates/cardinal/src/b.rs` with `// spec: SYN-2`, when the checker runs, then `SYN-2` is in the warning.
- (TRC-7) Given `// spec: SYN-2 (internal: )`, when the checker runs, then it fails naming the file and line.
- (TRC-7) Given `// spec: SYN-2 (internal: slow) SYN-3`, `// spec: SYN-2 (internal: slow` or
  `// spec: (internal: slow)`, when the checker runs, then it fails naming the file and line.
- (TRC-9) Given a crate at the root, `SYN-1` cited in `src/lib.rs` and `SYN-2` in `tests/cli.rs`, when the checker
  runs, then both are traced and the boundary warning names `SYN-1` alone.
- (TRC-9) Given `// spec: SYN-1` in `build.rs` or `examples/e.rs` above no `fn`, when the checker runs, then it fails
  naming the file and line, as anywhere else.
- (TRC-9) Given a `Cargo.toml` at the root and `// spec: SYN-1` in `target/debug/build/x/out/t.rs` only, when the
  checker runs outside git, then `SYN-1` is not traced; in `crates/a/tests/target/t.rs`, a module, it is.
- (TRC-10) Given `skipped_dirs = ["vendor"]` and `// spec: SYN-9` in `vendor/a/tests/t.rs`, when the checker runs, then
  it reports no unknown id.
- (TRC-8) Given `// spec: SYN-2, SYN-3 (internal: the SYN-9 fixture is slow)`, when the checker runs, then it reads
  `SYN-2` and `SYN-3`, flags no unknown id, and `SYN-9` is not cited.

- (TRC-11) Given `spec_files = ["test_*.py"]`, `// spec: SYN-1` in `src/lib.rs` and `# spec: SYN-2` above `def
  test_x():` in `tests/test_cli.py`, when the checker runs, then `SYN-2` is traced and `SYN-1` is not.
- (TRC-11) Given `spec_files = ["*_test.go"]` and `// spec: SYN-1` above `func TestX(t *testing.T) {` in
  `pkg/a/x_test.go`, when the checker runs, then `SYN-1` is traced.
- (TRC-11) Given no `spec_files` key, when the checker runs, then it reads the `.rs` files as TRC-9 says.
- (TRC-12) Given `spec_files = ["*.rs", "*.py"]` and a line `# spec: SYN-1` in a `.rs` file, when the checker runs,
  then `SYN-1` is not cited by that line.
- (TRC-12) Given `-- spec: SYN-1` in `t/a.foo` matched by `spec_files = ["*.foo"]`, when the checker runs, then
  `SYN-1` is cited.
- (TRC-13) Given `# spec: SYN-1`, then `@pytest.mark.slow`, then `def test_x():` in a `.py` test file, when the
  checker runs, then it reports no problem; given `# spec: SYN-1` on the last line, or followed by a blank line, it
  fails naming the file and line.
- (TRC-14) Given `internal_dirs = ["lib"]`, `SYN-1` cited only in `lib/a_test.py` and `SYN-2` only in `src/b_test.py`,
  when the checker runs, then the boundary warning names `SYN-1` alone.
- (TRC-15) Given `spec_files = ["tests/*.py"]`, when the checker runs, then it fails naming `tests/*.py`; given
  `internal_dirs = ["tests/unit"]`, it fails naming `tests/unit`.
- (TRC-16) Given `spec_files = "test_*.py"`, when the checker runs, then it fails naming `spec_files`.
- (TRC-11) Given `spec_files = ["t[1]_?.py"]`, when the checker runs, then it reads `t[1]_?.py` and not `t1_a.py`.
- (TRC-12) Given `;; spec: SYN-1` above `(deftest a ...)` in a `.clj` test file, when the checker runs, then `SYN-1` is
  cited.

- (TRC-17) Given `- **SYN-H1** (happy; SYN-1, SYN-4) Given …` first in the Scenarios of `specs/sync.md`, when the
  checker runs, then it reads the happy path `SYN-H1` walking through `SYN-1` and `SYN-4`.
- (TRC-18) Given `- **SYN-H1** (happy; SYN-99) Given …` or `- **SYN-H1** (happy; TRC-1) Given …` in `specs/sync.md`,
  when the checker runs, then it fails naming `SYN-H1`; given `- **SYN-H1** (happy) Given …`, or a happy path naming
  a withdrawn requirement, it fails too.
- (TRC-19) Given `- (happy; SYN-1) Given …` or `- **TRC-H1** (happy; SYN-1) Given …` in `specs/sync.md`, when the
  checker runs, then it fails naming the file and the line; given `- (SYN-1) Given an unhappy user …` or
  `- Happy path (SYN-1): Given …`, a scenario written before happy paths had a form, it does not.
- (TRC-20) Given `- (SYN-2) Given …` then `- **SYN-H1** (happy; SYN-1) Given …`, when the checker runs, then it fails
  naming `SYN-H1`.
- (TRC-23) Given an `accepted` spec whose every requirement is cited and whose happy path `SYN-H1` is not, when the
  checker runs, then it fails naming `SYN-H1`; in a `draft` spec, it does not.
- (TRC-22) Given `// spec: SYN-H7` in a test and no happy path `SYN-H7`, when the checker runs, then it fails naming
  `SYN-H7` as an unknown id.
- (TRC-24) Given `SYN-H1` cited only in `crates/cardinal/src/tests.rs` with `// spec: SYN-H1`, when the checker runs,
  then the boundary warning of `specs/sync.md` names `SYN-H1`.

## Non-goals

- Checking that a test outside `src` really runs the program: the place of the test is the convention.
- Failing the check on an internal-only proof: a boundary proof is an improvement, not a rule.
- Judging whether a reason is good: the checker only requires one.
- The other format and traceability checks of `check-specs`.
- Telling the build output in another place than a `target` beside a `Cargo.toml` (a custom `target-dir`), or `.rs`
  fixtures, from the tests: the `skipped_dirs` of `.claude/cardinal.toml` leave them out.
- The build output of the other stacks (`dist`, `build`, `.venv`, `__pycache__`): git ignores it, and outside git the
  `skipped_dirs` leave it out.
- Telling a test declaration from other code in a file other than a `.rs` one: the line after the citation only has
  to be there.
- Block comments (`/* */`, `"""`) as citations: the citation is one line comment.
- Checking that the test citing a happy path drives the whole flow, or that the path is written in the terms of the
  user: the README and `skill:write-spec` ask for it, a reader judges it.
- Withdrawing a happy path: one that no longer holds is rewritten or deleted, its tests with it.
