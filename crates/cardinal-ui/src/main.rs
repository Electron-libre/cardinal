//! `cardinal-ui`: the milestones, the queue of issues, the worktrees and the measures of the project of the current
//! directory, in one terminal screen (`specs/steering-ui.md`, its actions in `specs/steering-ui-actions.md`, ADR 0030).
//! It reads the project through `cardinal run <tool> --json` and git, keeps its measures in the cache directory of the
//! user (`measures.rs`), and changes the project only to start an issue session in a zellij tab (`start.rs`), to start
//! the release session of the milestones awaiting it (`release.rs`) or the review of the priorities of the issues
//! (`prioritize.rs`), the audit of the code (`audit.rs`) or the cut of a milestone into issues (`cut.rs`), each in a tab of its own (`skill_tab.rs`), to merge into the trunk an issue closed on its
//! branch (`merge.rs`), or to end what a merged issue left (`end.rs`); it notifies the desktop when a worktree starts
//! to wait for the user (`notify.rs`); `g` shows the charts of the measures (`charts.rs`), then their table (`screen.rs`); `:` opens the command palette (`palette.rs`); `o` and `O` open the Markdown document of the selected row in a viewer or an editor (`document.rs`); `--print` renders as text (`print.rs`). It draws in the colors of the theme file of the user (`theme.rs`).
//! Usage: `cardinal-ui [--print | --start <stem> | --merge <stem> | --end <stem> | --end-merged | --release | --prioritize | --audit | --cut <milestone>]`: with no argument the
//! screen runs in the terminal until `q`; `--print` prints it once as text; `--start` starts (or resumes) the session of an issue;
//! `--merge` merges it; `--end` ends its session and removes its worktree once it is merged; `--end-merged` does so on every
//! merged worktree; `--release` starts the release session in the main worktree; `--prioritize` the review of the
//! priorities there; `--audit` the audit of the code there; `--cut` the cut of a milestone there.

mod audit;
mod charts;
mod clock;
mod cut;
mod document;
mod end;
mod help;
mod keys;
mod limits;
mod mark;
mod measures;
mod merge;
mod notify;
mod palette;
mod panels;
mod panics;
mod print;
mod prioritize;
mod release;
mod screen;
mod skill_tab;
mod source;
mod spinner;
mod start;
mod status;
mod summary;
mod terminal;
mod theme;
mod tree;
mod zellij;

use std::path::Path;

const USAGE: &str =
    "usage: cardinal-ui [--print | --start <stem> | --merge <stem> | --end <stem> | --end-merged | --release | --prioritize | --audit | --cut <milestone>]";

/// Reads the project, with whether a measure runs and the error of the last one.
fn read(measuring: bool, measure_error: Option<String>) -> screen::View {
    let root = source::root(Path::new("."));
    let state = source::read(&root, measuring, measure_error);
    let worktrees = state.worktrees.as_ref().map_or_else(
        |_| Vec::new(),
        |ws| {
            ws.iter()
                .map(|w| (w.path.clone(), w.branch.clone()))
                .collect()
        },
    );
    let waits = state.worktrees.as_deref().ok().map(notify::waits);
    let (panels, rows) = panels::panels(&state);
    let (rows, row_lines): (Vec<keys::Row>, _) = rows.into_iter().unzip();
    let keys = help::Keys::of(&state, &rows, source::in_zellij());
    let milestones = state.milestones.as_ref().map_or(&[][..], |m| &m.list[..]);
    let summaries = summary::of(state.queue.as_ref().ok(), milestones, &rows);
    let documents = document::of(&root, state.queue.as_ref().ok(), milestones, &rows);
    screen::View {
        documents,
        panels,
        rows,
        row_lines,
        worktrees,
        waits,
        charts: state.charts,
        keys,
        summaries,
        next: state.queue.as_ref().ok().and_then(|q| q.next.clone()),
    }
}

/// Starts the session of the issue `stem` of the project of the current directory (SUA-1).
fn start_in_project(stem: &str) -> Result<String, String> {
    start::start(&source::root(Path::new(".")), stem)
}

/// Runs the screen in the terminal (STU-2).
fn interactive() -> i32 {
    let measurer = measures::Measurer::new(source::root(Path::new(".")));
    let reader = measurer.clone();
    let read = move || {
        let (running, error) = reader.status();
        read(running, error)
    };
    let (theme, ignored) = theme::load();
    let status = theme::status(&ignored);
    match terminal::run(
        read,
        run_task,
        (|| measurer.start_if_stale(), || measurer.start()),
        (theme, status),
    ) {
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
    // the lines of the theme file it ignored, the colors themselves never printed (SUS-1, SUS-5)
    for line in theme::load().1 {
        eprintln!("{line}");
    }
    let root = source::root(Path::new("."));
    let error = match measures::directory(&root) {
        None => Some("measures: no home directory".to_string()),
        Some(dir) if measures::stale(&dir) => measures::measure(&root, &dir).err(),
        Some(_) => None,
    };
    let view = read(false, error);
    // the help line of the selection the screen starts on (SUA-15)
    let mut start = keys::Selection::default();
    start.start(&view);
    let help = help::line(&view, start, false);
    let text = print::text(&view.panels, &help);
    // a reader that closed the output early (`| head`) is no failure
    let _ = std::io::Write::write_all(&mut std::io::stdout(), text.as_bytes());
    i32::from(view.panels.iter().any(|p| p.failed))
}

/// Runs the task of a key on the issue `stem` of the project of the current directory (SUA-1, SUM-1, SUE-1), or
/// starts its release session (SUA-4), the review of its priorities (SUA-16), the audit of its code (SUA-23) or the cut of its milestone `stem`
/// (SUA-19).
fn run_task(task: keys::Task, stem: &str) -> Result<String, String> {
    match task {
        keys::Task::Start => start_in_project(stem),
        keys::Task::Merge => merge_then_end(stem),
        keys::Task::End => end::end(&source::root(Path::new(".")), stem),
        keys::Task::Release => release::start(&source::root(Path::new("."))),
        keys::Task::Prioritize => prioritize::start(&source::root(Path::new("."))),
        keys::Task::Audit => audit::start(&source::root(Path::new("."))),
        keys::Task::EndMerged => end_merged_status(),
        keys::Task::Cut => cut::start(&source::root(Path::new(".")), stem),
    }
}

/// Ends every merged worktree of the project of the current directory: the status line of SUE-2, an error when it
/// kept one.
fn end_merged_status() -> Result<String, String> {
    let (lines, all_ended) = end::end_merged(&source::root(Path::new(".")))?;
    end::merged_status(&lines, all_ended)
}

/// Prints one line per merged worktree on stdout, exit code 0 when it ended them all, 1 when it kept one; or why it
/// ends nothing on stderr, exit code 1 (SUE-5).
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

/// Merges the issue `stem` of the project of the current directory into its trunk (SUM-2).
fn merge_in_project(stem: &str) -> Result<String, String> {
    merge::merge(&source::root(Path::new(".")), stem)
}

/// Merges the issue `stem`, then ends what it left once its check passed (SUM-1, SUE-1).
fn merge_then_end(stem: &str) -> Result<String, String> {
    let root = source::root(Path::new("."));
    end::after_merge(merge::merge_branch(&root, stem), |branch| {
        end::end_on(&root, stem, branch)
    })
}

/// Prints what `done` did on stdout, exit code 0, or why it did not on stderr, exit code 1 (SUA-5, SUM-2).
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
        [a] if a == "--prioritize" => report(prioritize::start(&source::root(Path::new(".")))),
        [a] if a == "--audit" => report(audit::start(&source::root(Path::new(".")))),
        [a, stem] if a == "--start" => report(start_in_project(stem)),
        [a, stem] if a == "--merge" => report(merge_in_project(stem)),
        [a, name] if a == "--cut" => report(cut::start(&source::root(Path::new(".")), name)),
        [a, stem] if a == "--end" => report(end::end(&source::root(Path::new(".")), stem)),
        _ => {
            eprintln!("{USAGE}");
            2
        }
    };
    std::process::exit(code);
}
