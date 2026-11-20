//! `cardinal-ui`: the milestones, the queue of issues, the worktrees and the measures of the project of the current
//! directory, in one terminal screen (`specs/steering-ui.md`, ADR 0030). It reads the project through
//! `cardinal run <tool> --json` and git, keeps its measures in the cache directory of the user (`measures.rs`), and
//! changes the project only to start an issue session in a zellij tab (`start.rs`), to merge into the trunk an issue
//! closed on its branch (`merge.rs`), or to end what a merged issue left (`end.rs`).
//! Usage: `cardinal-ui [--print | --start <stem> | --merge <stem> | --end <stem>]`: with no argument the screen runs
//! in the terminal until `q`; `--print` prints it once as text; `--start` starts the session of an issue; `--merge`
//! merges it; `--end` ends its session and removes its worktree once it is merged.

mod end;
mod measures;
mod merge;
mod panels;
mod screen;
mod source;
mod start;

use std::path::Path;

const USAGE: &str = "usage: cardinal-ui [--print | --start <stem> | --merge <stem> | --end <stem>]";

/// Reads the project, with whether a measure runs and the error of the last one.
fn read(measuring: bool, measure_error: Option<String>) -> screen::View {
    let state = source::read(&source::root(Path::new(".")), measuring, measure_error);
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
    let measurer = measures::Measurer::new(source::root(Path::new(".")));
    measurer.start_if_stale();
    let reader = measurer.clone();
    let read = move || {
        let (running, error) = reader.status();
        read(running, error)
    };
    match screen::run(read, run_task, || measurer.start()) {
        Ok(None) => 0,
        // the result of a task the user quit during (STU-15)
        Ok(Some(line)) => {
            eprintln!("cardinal-ui: {line}");
            0
        }
        Err(e) => {
            eprintln!("cardinal-ui: {e}");
            1
        }
    }
}

/// Prints the panels once as text, measuring first when the last measure is missing or over an hour old; exit code 1
/// when one shows an error (STU-4, STU-8, STU-17).
fn print() -> i32 {
    let root = source::root(Path::new("."));
    let error = match measures::directory(&root) {
        None => Some("measures: no home directory".to_string()),
        Some(dir) if measures::stale(&dir) => measures::measure(&root, &dir).err(),
        Some(_) => None,
    };
    let panels = read(false, error).panels;
    // a reader that closed the output early (`| head`) is no failure
    let _ = std::io::Write::write_all(&mut std::io::stdout(), screen::text(&panels).as_bytes());
    i32::from(panels.iter().any(|p| p.failed))
}

/// Runs the task of a key on the issue `stem` of the project of the current directory (STU-9, STU-20).
fn run_task(task: screen::Task, stem: &str) -> Result<String, String> {
    match task {
        screen::Task::Start => start_in_project(stem),
        screen::Task::Merge => merge_in_project(stem),
    }
}

/// Merges the issue `stem` of the project of the current directory into its trunk (STU-20).
fn merge_in_project(stem: &str) -> Result<String, String> {
    merge::merge(&source::root(Path::new(".")), stem)
}

/// Prints what `done` did on stdout, exit code 0, or why it did not on stderr, exit code 1 (STU-11, STU-21).
fn report(done: Result<String, String>) -> i32 {
    match done {
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
        [a, stem] if a == "--start" => report(start_in_project(stem)),
        [a, stem] if a == "--merge" => report(merge_in_project(stem)),
        [a, stem] if a == "--end" => report(end::end(&source::root(Path::new(".")), stem)),
        _ => {
            eprintln!("{USAGE}");
            2
        }
    };
    std::process::exit(code);
}
