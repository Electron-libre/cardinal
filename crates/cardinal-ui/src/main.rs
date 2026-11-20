//! `cardinal-ui`: the milestones, the queue of issues and the worktrees of the project of the current directory, in
//! one terminal screen (`specs/steering-ui.md`, ADR 0030). It reads the project through `cardinal run <tool> --json`
//! and git, and changes the project only to start an issue session in a zellij tab (`start.rs`).
//! Usage: `cardinal-ui [--print | --start <stem>]`: with no argument the screen runs in the terminal until `q`;
//! `--print` prints it once as text; `--start` starts the session of an issue.

mod panels;
mod screen;
mod source;
mod start;

use std::path::Path;

const USAGE: &str = "usage: cardinal-ui [--print | --start <stem>]";

fn read() -> screen::View {
    let state = source::read(&source::root(Path::new(".")));
    let stems = state.queue.as_ref().map_or_else(
        |_| Vec::new(),
        |q| q.issues.iter().map(|i| i.stem.clone()).collect(),
    );
    screen::View {
        panels: panels::panels(&state),
        stems,
    }
}

/// Starts the session of the issue `stem` of the project of the current directory (STU-9).
fn start_in_project(stem: &str) -> Result<String, String> {
    start::start(&source::root(Path::new(".")), stem)
}

/// Runs the screen in the terminal (STU-2).
fn interactive() -> i32 {
    match screen::run(read, start_in_project) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("cardinal-ui: {e}");
            1
        }
    }
}

/// Prints the panels once as text; exit code 1 when one shows an error (STU-4, STU-8).
fn print() -> i32 {
    let panels = read().panels;
    // a reader that closed the output early (`| head`) is no failure
    let _ = std::io::Write::write_all(&mut std::io::stdout(), screen::text(&panels).as_bytes());
    i32::from(panels.iter().any(|p| p.failed))
}

/// Starts the session of the issue `stem`: exit code 1 when nothing started (STU-11).
fn start(stem: &str) -> i32 {
    match start_in_project(stem) {
        Ok(done) => {
            println!("{done}");
            0
        }
        Err(e) => {
            eprintln!("cardinal-ui: {e}");
            1
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.as_slice() {
        [] => interactive(),
        [a] if a == "--print" => print(),
        [a, stem] if a == "--start" => start(stem),
        _ => {
            eprintln!("{USAGE}");
            2
        }
    };
    std::process::exit(code);
}
