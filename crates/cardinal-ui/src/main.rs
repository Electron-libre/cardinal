//! `cardinal-ui`: the milestones, the queue of issues and the worktrees of the project of the current directory, in
//! one terminal screen (`specs/steering-ui.md`, ADR 0030). It reads the project through `cardinal run <tool> --json`
//! and git, and writes nothing.
//! Usage: `cardinal-ui [--print]`.

mod panels;
mod source;

use std::path::Path;

const USAGE: &str = "usage: cardinal-ui [--print]";

/// Prints the panels once as text; exit code 1 when one shows an error (STU-4, STU-8).
fn print() -> i32 {
    let panels = panels::panels(&source::read(Path::new(".")));
    for p in &panels {
        println!("== {} ==", p.title);
        for line in &p.lines {
            println!("{line}");
        }
    }
    i32::from(panels.iter().any(|p| p.failed))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.as_slice() {
        [] => print(),
        [a] if a == "--print" => print(),
        _ => {
            eprintln!("{USAGE}");
            2
        }
    };
    std::process::exit(code);
}
