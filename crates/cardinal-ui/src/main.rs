//! `cardinal-ui`: the milestones, the queue of issues, the worktrees and the measures of the project of the current
//! directory, in one terminal screen (`specs/steering-ui.md`, its actions in `specs/steering-ui-actions.md`, ADR 0030).
//! It reads the project through `cardinal run <tool> --json` and git, keeps its measures in the cache directory of the
//! user (`measures.rs`), and changes the project only to start an issue session in a zellij tab (`start.rs`), to start
//! the release session of the milestones awaiting it (`release.rs`), to merge into the trunk an issue closed on its
//! branch (`merge.rs`), or to end what a merged issue left (`end.rs`); it notifies the desktop when a worktree starts
//! to wait for the user (`notify.rs`). It draws in the colors of the theme file of the user (`theme.rs`).
//! Usage: `cardinal-ui [--print | --start <stem> | --merge <stem> | --end <stem> | --end-merged | --release]`: with no argument the
//! screen runs in the terminal until `q`; `--print` prints it once as text; `--start` starts (or resumes) the session of an issue;
//! `--merge` merges it; `--end` ends its session and removes its worktree once it is merged; `--end-merged` does so on every
//! merged worktree; `--release` starts the release session in the main worktree.

mod end;
mod keys;
mod measures;
mod merge;
mod notify;
mod panels;
mod release;
mod screen;
mod source;
mod start;
mod theme;

use std::path::Path;

const USAGE: &str =
    "usage: cardinal-ui [--print | --start <stem> | --merge <stem> | --end <stem> | --end-merged | --release]";

/// Reads the project, with whether a measure runs and the error of the last one.
fn read(measuring: bool, measure_error: Option<String>) -> screen::View {
    let state = source::read(&source::root(Path::new(".")), measuring, measure_error);
    let stems = state.queue.as_ref().map_or_else(
        |_| Vec::new(),
        |q| q.issues.iter().map(|i| i.stem.clone()).collect(),
    );
    let worktrees = state.worktrees.as_ref().map_or_else(
        |_| Vec::new(),
        |ws| {
            ws.iter()
                .map(|w| (w.path.clone(), w.branch.clone()))
                .collect()
        },
    );
    let waits = state.worktrees.as_deref().ok().map(notify::waits);
    screen::View {
        panels: panels::panels(&state),
        stems,
        worktrees,
        waits,
    }
}

/// Starts the session of the issue `stem` of the project of the current directory (SUA-1).
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
    let (theme, ignored) = theme::load(theme::path().as_deref());
    let status = theme::status(&ignored);
    match screen::run(read, run_task, || measurer.start(), (theme, status)) {
        Ok(None) => 0,
        // the result of a task the user quit during (SUA-6)
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
    // the lines of the theme file it ignored, the colors themselves never printed (STU-31, STU-33)
    for line in theme::load(theme::path().as_deref()).1 {
        eprintln!("{line}");
    }
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

/// Runs the task of a key on the issue `stem` of the project of the current directory (SUA-1, SUA-2, SUA-3), or
/// starts its release session (SUA-4).
fn run_task(task: keys::Task, stem: &str) -> Result<String, String> {
    match task {
        keys::Task::Start => start_in_project(stem),
        keys::Task::Merge => merge_then_end(stem),
        keys::Task::End => end::end(&source::root(Path::new(".")), stem),
        keys::Task::Release => release::start(&source::root(Path::new("."))),
        keys::Task::EndMerged => end_merged_status(),
    }
}

/// Ends every merged worktree of the project of the current directory: the status line of SUA-13, an error when it
/// kept one.
fn end_merged_status() -> Result<String, String> {
    let (lines, all_ended) = end::end_merged(&source::root(Path::new(".")))?;
    end::merged_status(&lines, all_ended)
}

/// Prints one line per merged worktree on stdout, exit code 0 when it ended them all, 1 when it kept one; or why it
/// ends nothing on stderr, exit code 1 (SUA-14).
fn end_merged() -> i32 {
    match end::end_merged(&source::root(Path::new("."))) {
        Ok((lines, all_ended)) => {
            for line in end::merged_lines(&lines) {
                println!("{line}");
            }
            i32::from(!all_ended)
        }
        Err(e) => {
            eprintln!("cardinal-ui: {e}");
            1
        }
    }
}

/// Merges the issue `stem` of the project of the current directory into its trunk (SUA-7).
fn merge_in_project(stem: &str) -> Result<String, String> {
    merge::merge(&source::root(Path::new(".")), stem)
}

/// Merges the issue `stem`, then ends what it left once its check passed (SUA-2, SUA-3).
fn merge_then_end(stem: &str) -> Result<String, String> {
    let root = source::root(Path::new("."));
    end::after_merge(merge::merge_branch(&root, stem), |branch| {
        end::end_on(&root, stem, branch)
    })
}

/// Prints what `done` did on stdout, exit code 0, or why it did not on stderr, exit code 1 (SUA-5, SUA-7).
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
        [a] if a == "--end-merged" => end_merged(),
        [a] if a == "--release" => report(release::start(&source::root(Path::new(".")))),
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
