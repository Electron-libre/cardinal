//! `project-check`: the check of a project through `cardinal` alone (ADR 0028). It runs the `check_command` of the
//! `[project]` table of `.claude/cardinal.toml` with `sh -c` from the root of the work tree, then `check-specs` unless
//! the feature `specs` is off; quiet on success, since the agent reads all of its output, and never a pass when no
//! check command is declared. Spec: `specs/project-check.md`.

use std::io::Write;
use std::process::Command;

use super::{check_specs, project_str, repo_root};
use crate::features;

const USAGE: &str = "usage: cardinal run project-check";
const UNSET: &str =
    "project-check: declare the project's check (format, lint, tests) as `check_command` in \
                     [project] of .claude/cardinal.toml";

pub(super) fn main(args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("{USAGE}");
        return 2;
    }
    let root = match repo_root() {
        Ok(root) => root,
        Err(e) => {
            eprintln!("project-check: {e}");
            return 1;
        }
    };
    let Some(command) = project_str(&root, "check_command") else {
        eprintln!("{UNSET}");
        return 1;
    };
    let out = match Command::new("sh")
        .args(["-c", &command])
        .current_dir(&root)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            eprintln!("project-check: cannot run sh: {e}");
            return 1;
        }
    };
    if !out.status.success() {
        let _ = std::io::stdout().write_all(&out.stdout);
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().write_all(&out.stderr);
        return 1;
    }
    if features::is_off(&root, features::SPECS) {
        return 0;
    }
    let problems = check_specs::problems(&root, false);
    for p in &problems {
        println!("{p}");
    }
    u8::from(!problems.is_empty())
}
