//! `cardinal-ui`: the milestones, the queue of issues and the worktrees of the project of the current directory, in
//! one terminal screen (`specs/steering-ui.md`, ADR 0030). It reads the project through `cardinal run <tool> --json`
//! and git, and writes nothing.
//! Usage: `cardinal-ui [--print]`: with no argument the screen runs in the terminal until `q`; `--print` prints it
//! once as text.

mod panels;
mod screen;
mod source;

use std::path::Path;

const USAGE: &str = "usage: cardinal-ui [--print]";

fn read() -> [panels::Panel; 3] {
    panels::panels(&source::read(&source::root(Path::new("."))))
}

/// Runs the screen in the terminal (STU-2).
fn interactive() -> i32 {
    match screen::run(read) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("cardinal-ui: {e}");
            1
        }
    }
}

/// Prints the panels once as text; exit code 1 when one shows an error (STU-4, STU-8).
fn print() -> i32 {
    let panels = read();
    // a reader that closed the output early (`| head`) is no failure
    let _ = std::io::Write::write_all(&mut std::io::stdout(), screen::text(&panels).as_bytes());
    i32::from(panels.iter().any(|p| p.failed))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.as_slice() {
        [] => interactive(),
        [a] if a == "--print" => print(),
        _ => {
            eprintln!("{USAGE}");
            2
        }
    };
    std::process::exit(code);
}
