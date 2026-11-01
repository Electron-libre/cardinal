---
id: FEA
status: accepted
adrs: [0014, 0025]
---

# Features a project turns off

## Intent

A source groups its managed files and settings entries into named features; a project lists the ones it turns off in
`.claude/cardinal.toml`, and `sync` leaves them out, so a feature turned off costs no token and no hook call. The
checks of the binary that a feature carries stay quiet in a project that turned it off.

## Requirements

- **FEA-1** (deterministic) `sync` must read the features of the source from `features.toml` at its root: one table per feature, named after it, whose `files` lists paths under `files/` and whose `entries` lists managed entries of `entries/.claude/settings.json` written as `sync` prints them (`allow <rule>`, `deny <rule>` or `hook <event> <matcher> <command>`, SYN-34); a source with no `features.toml` declares no feature.
- **FEA-2** (deterministic) `sync` must read the features the project turns off from the `disabled` list of strings of the `[cardinal]` table of `.claude/cardinal.toml`; a project with no `disabled` key turns off no feature.
- **FEA-3** (deterministic) `sync` must handle each file and each entry of a feature the project turns off as one the source does not manage: not copied nor added, removed when the lock recorded it and the project left it unchanged (SYN-31), kept otherwise, and not recorded in the lock.
- **FEA-4** (deterministic) `check` must not report a file or an entry of a feature the project turned off, once a `sync` ran with it turned off.
- **FEA-5** (deterministic) `sync` must fail with the error `<config>: disabled names the unknown feature <name>; the source declares <names>` (exit 3, SYN-37) and write nothing, `--dry-run` included, when `disabled` names a feature the source does not declare.
- **FEA-6** (deterministic) `sync` must fail with an error (exit 3) naming the feature and the path or entry, and write nothing, when `features.toml` names a file absent from `files/` or an entry absent from `entries/.claude/settings.json`.
- **FEA-7** (deterministic) `sync` must fail with an error (exit 3) and write nothing when `features.toml` is not valid TOML in the shape of FEA-1, or when `disabled` is not a list of strings.
- **FEA-8** (deterministic) `commit-check` and `check-commits` must not report an attribution line when the project turns off the feature `no-ai-attribution`.
- **FEA-9** (deterministic) `check-issue` must not report an attribution line when the project turns off the feature `no-ai-attribution`.
- **FEA-10** (deterministic) `check`, `sync` and `update` must not print the warning of SYN-26 when the project turns off the feature `no-ai-attribution`.
- **FEA-11** (deterministic) `rule-breaches` must not apply rule `commit-attribution` (BRE-9) to a commit whose `.claude/cardinal.toml`, as of that commit, turns off the feature `no-ai-attribution`.
- **FEA-12** (deterministic) The source of Cardinal must declare the feature `no-ai-attribution`, whose files hold the rule against AI attribution in commits.
- **FEA-13** (deterministic) `files/AGENTS.md` must not hold the rule against AI attribution: a project that turns the feature off loads it nowhere.
- **FEA-14** (deterministic) `update` must not refuse a revision for a hook the installed `cardinal` cannot run (BIN-16) when that hook is an entry of a feature the project turns off.
- **FEA-15** (deterministic) `sync` must not seed a path of `seed/` that is also a file of a feature the project turns off.
- **FEA-16** (deterministic) The source of Cardinal must declare the features `lint-gate`, `format-code` and `language-hint`, each made of the one hook entry of `entries/.claude/settings.json` whose command is `cardinal run <feature>`, and no file.

## Scenarios

- (FEA-3) Given a project synced with every feature on, when it adds `no-ai-attribution` to `disabled` and runs `sync`, then the files of the feature are reported `removed` and are gone, and the lock no longer records them.
- (FEA-3) Given a project that edited a file of a feature, when it turns the feature off and runs `sync`, then the file is reported `kept` and stays as the project wrote it.
- (FEA-3) Given a project that turned a feature off, when it removes the name from `disabled` and runs `sync`, then the files and entries of the feature are added back.
- (FEA-3) Given a feature whose entry is a hook, when the project turns it off, then `sync` removes the hook object from `.claude/settings.json`, and a group left empty with it (SYN-31).
- (FEA-5) Given `disabled = ["no-ai-atribution"]`, when `sync` runs, then it fails naming the unknown feature and the features the source declares.
- (FEA-8) Given `disabled = ["no-ai-attribution"]` and a message with a co-author trailer, when `commit-check` reads it, then it prints nothing about the trailer and still checks the subject.

## Non-goals

- Features that need one another: no feature of the source needs another yet; the first that does adds the rule.
- A person turning off a feature for themselves in `.claude/cardinal.local.toml`: `disabled` changes the committed
  files of the project, so it is the project's choice (ADR 0018).
- Turning off the core of `cardinal` (`sync`, `check`, the lock, the formats and migrations).
- Rewriting the seeded files of a project whose feature is turned off: the `attribution` of its seeded
  `.claude/settings.json` stays the project's to change.
- Declaring the features other than `no-ai-attribution` and the hooks of FEA-16: each comes with its own issue.
- Silencing rule `user-language` of `rule-breaches` when `language-hint` is off: the rule is the user's and stays
  loaded; the feature only drops the hook that enforces it, whose hints then no longer appear (BRE-13).
