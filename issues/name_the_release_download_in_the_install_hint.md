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
* `specs/tool-binaries.md` BIN-15 and BIN-16, `specs/sync.md` SYN-40 and SYN-42: the form of `<command>`.

# Resources
* `crates/cardinal/src/launcher.rs`, `crates/cardinal/src/lib.rs` (`install_hint`), `crates/cardinal/src/run.rs`
  (`asset_urls`, the platform names).
* `.github/workflows/release.yml`, `.github/install_cardinal.sh`.
* `docs/tutorial.md` steps 1 and the troubleshooting table, `docs/usage.md` "The installed launcher".
* ADR 0010 (the tools ship in the harness binary).

# Hints
* A local directory source (this repository) keeps the `cargo install --path`: it has no release.
* The hint is printed when every hook refuses, so it must be runnable as is, with no `cardinal` to help.
