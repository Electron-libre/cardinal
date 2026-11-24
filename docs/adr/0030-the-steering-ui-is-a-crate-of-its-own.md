# 0030. The steering UI is a crate of its own

Status: Accepted, partly superseded by 0033 (the UI shares the crate `cardinal-state` with `cardinal`)
Date: 2026-11-20

## Context
The user steers a project through several commands (`next-issue`, `roadmap`, `git worktree list`) and by opening
each worktree to see which session works there. A terminal UI can show all of it in one screen (milestone "Steering
UI"). `cardinal` is a launcher that a project pins to a revision (ADR 0010, ADR 0028): a subcommand of it would grow
every consumer's binary with a terminal library, and would show each project through the UI of its pinned revision.

## Decision
The UI is the binary `cardinal-ui` of its own crate, `crates/cardinal-ui`, in this workspace. It reads a project
only through `cardinal run <tool> --json` (the documents of NXT-19 and RMP-8, checked by their `version`) and git
commands that read, never through the code of the `cardinal` crate, which it does not depend on: the tools of the
revision the project pins answer, and the UI of any revision reads every project whose tools print version 1. It is
not managed content: `sync` copies nothing of it, and a consumer does not need it. It is installed apart, with
`cargo install` (`just install-ui` here); the release workflow keeps building `cardinal` only.

## Consequences
The UI follows the JSON documents, not the internals, so a change of `cardinal` breaks it only when it raises a
`version`, which the UI reports instead of guessing. A project pinned to a revision without `--json` shows the error
of its tools in the UI. The UI needs Rust to install until a later issue publishes its binary. Its terminal library
stays out of `cardinal` and of its tests.
