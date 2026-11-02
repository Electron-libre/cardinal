---
priority: medium
why: 'a consumer without Rust is told to run a cargo install it cannot run, while the binary it needs is published'
---
# Name the release download in the install hint

# Goal
When the installed `cardinal` is too old for a project, every error that says what to install prints a
`cargo install`. A project that adopts Cardinal is told by `docs/tutorial.md` to download the released binary, and
needs no Rust toolchain for anything else: `cardinal run` fetches the binary of the pinned commit from its release.
The outcome: for a GitHub source whose commit has a release asset for the platform, the hint names the download of
that asset (and its SHA-256 check), the `cargo install` kept as the fallback for another platform or source.

# Findings
* 2026-11-02, a scratch project adopting this repository as a local source: `cardinal init` refused with
  `needs a cardinal that reads the format 7, this one reads up to 6: install it with `cargo install --force --path
  <source>/crates/cardinal --locked``.
* `launcher::install` (`crates/cardinal/src/launcher.rs`) builds every hint: a `cargo install --force --git <url>
  --rev <rev> --locked` for a git source, `--path` for a local one. Its callers: `install_hint` (`lib.rs`, the format
  refusals SYN-40 and SYN-42), the refused hooks of `check` (BIN-15) and of `update` (BIN-16).
* `.github/workflows/release.yml` publishes `cardinal-<commit>` with the assets `cardinal-<target>` and
  `cardinal-<target>.sha256` for `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and
  `aarch64-apple-darwin`; `run.rs` (`asset_urls`) and `.github/install_cardinal.sh` already fetch them.
* The release is created with `--latest=false`, yet `gh release list` marks the newest one (of the head of `main`)
  `Latest` and `releases/latest/download/cardinal-x86_64-unknown-linux-musl` served it on 2026-11-02; why GitHub
  marks it so was not checked. `docs/tutorial.md` step 1 relies on that URL.

# Approach
1. Make `launcher::install` return the download of the release asset of the commit (`curl` of
   `releases/download/cardinal-<commit>/cardinal-<target>` into the directory of the `cardinal` on the path, the
   SHA-256 compared, `chmod +x`) when the source is on GitHub, the revision a full commit and the platform one with
   an asset, else the `cargo install` of today. A long hint may point at `docs/tutorial.md` step 1 instead of
   spelling the commands.
2. Decide what a `rev` that is not a commit (a branch, a tag) prints: the `cargo install`, or the asset of the
   commit `update` resolved, which BIN-16 already knows.
3. Check why the release of the head of `main` is `Latest` under `--latest=false`, and drop `--latest=false` or
   change step 1 of the tutorial if the URL is not reliable.

# Specs
* `specs/tool-binaries.md`: creates BIN-19, BIN-20, BIN-21; amends BIN-15 and BIN-16 (their install is BIN-20's), which
  SYN-40 and SYN-42 of `specs/sync.md` refer to.

# Resources
* `crates/cardinal/src/launcher.rs`, `crates/cardinal/src/lib.rs` (`install_hint`), `crates/cardinal/src/run.rs`
  (`asset_urls`, the platform names).
* `.github/workflows/release.yml`, `.github/install_cardinal.sh`.
* `docs/tutorial.md` steps 1 and the troubleshooting table, `docs/usage.md` "The installed launcher".
* ADR 0010 (the tools ship in the harness binary).

# Hints
* A local directory source (this repository) keeps the `cargo install --path`: it has no release.
* The hint is printed when every hook refuses, so it must be runnable as is, with no `cardinal` to help.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-02

## Understanding
The installed `cardinal` names a `cargo install` whenever it is too old; a consumer without Rust cannot run it.
Means: a launcher subcommand `cardinal upgrade <source> <commit>` that fetches the binary of the commit as `run`
already does (release asset checked against its digest, else a cargo build) and replaces itself; the hints name it.

## Plan
1. [x] `cardinal upgrade`: the subcommand, through `run::binary_of` then an atomic replace of `current_exe`; verified
   by a binary test with a fake `curl` serving an asset into a copied launcher, and its failures; makes true BIN-19,
   BIN-21. (autonomous)
2. [ ] Hints: `launcher::install` returns `cardinal upgrade <source> <commit>` for a git source at a full commit;
   `docs/tutorial.md` (troubleshooting), `docs/usage.md` ("The installed launcher"), `README.md` (commands); verified
   by the update and check tests of BIN-15, BIN-16 and a format refusal; makes true BIN-20.

## Decisions
- Mode: autonomous
- Base commit: d64e77400378bceeda54878c952b7e117411c371
- The hint names a launcher subcommand, `cardinal upgrade <source> <commit>` (not a shell line, not a link).
- A git source at a revision that is not a full commit keeps the `cargo install` hint.
- `releases/latest` (step 1 of the tutorial): GitHub resolves it to the most recent non-draft, non-prerelease release
  by creation date when none is marked latest, so `--latest=false` still leaves it on the release of the head of
  `main`; no change.

## Deliverable 1
- done: `cardinal upgrade <source> <commit>` fetches the binary of the commit through `run::binary_of` and renames a
  copy over `current_exe`; README and usage list it.
- red: BIN-19, BIN-21: `error: unrecognized subcommand 'upgrade'`.
- Checks: `just check` ok, deliverable-size 151 lines.
- Code review: no findings
