//! The terminal loop of `cardinal-ui` (STU-2): the reads of the project on a thread of their own, every `REFRESH` and
//! on `r`, and the keys of the screen drawn by `screen.rs`: an issue or a milestone of the queue is selected
//! (STU-14), `s` starts its session (SUA-1), `f` merges it into the trunk (SUM-1); `Tab` moves the selection to the
//! worktrees, where `e` ends what a merged issue left (SUE-1); `E` ends it on every merged worktree (SUE-2); `v`
//! starts the release session (SUA-4), `p` the review of the priorities (SUA-16), `c` the cut of the selected
//! milestone (SUA-19), each task on a thread of its own; a worktree that starts to wait for the user notifies the
//! desktop (STU-29); `g` switches to the charts of the measures, then to their table, where `m` takes the measures
//! again (STU-17), and back (SUC-1); the mark of an issue at work turns at each tick (SUQ-4); `i` unfolds the summary
//! of the selected issue (SUQ-6); `n` selects the issue `next-issue` offers (STU-36); `:` opens the command palette
//! (SUP-1).

use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event};

use crate::keys::{ended, job, quits, screen_key, task, Action, Ended, Screen, Selection, Task};
use crate::notify::{self, Notifier};
use crate::palette::Palette;
use crate::panics::{self, Caught};
use crate::screen::{draw, View};
use crate::spinner;
use crate::status::{Kind, Status};
use crate::theme::Theme;

/// How often the screen reads the project again (STU-2).
const REFRESH: Duration = Duration::from_secs(2);

/// How long the loop waits for a key before it looks for a finished read.
const TICK: Duration = Duration::from_millis(100);

/// When to read the project: every `REFRESH` from the end of the last read and on `r`, one read at a time (STU-2).
#[derive(Default)]
struct Schedule {
    /// When the last read ended; `None` before the first.
    last: Option<Instant>,
    running: bool,
    asked: bool,
}

impl Schedule {
    /// Whether a read starts at `now`; it is then running.
    fn start(&mut self, now: Instant) -> bool {
        let due = self.asked || self.last.is_none_or(|t| now.duration_since(t) >= REFRESH);
        if self.running || !due {
            return false;
        }
        self.running = true;
        self.asked = false;
        true
    }

    /// The read started ended at `now`.
    fn ended(&mut self, now: Instant) {
        self.running = false;
        self.last = Some(now);
    }
}

/// Takes the read `reads` finished, if any, into `view`: the selection follows it (STU-14) and its new waits are
/// notified (STU-29). Whether one was taken; an error when the thread of the reads died (a panic), as no read would
/// ever come again.
fn received(
    reads: &mpsc::Receiver<View>,
    view: &mut Option<View>,
    selected: &mut Selection,
    notifier: &mut Notifier,
) -> std::io::Result<bool> {
    match reads.try_recv() {
        Ok(read) => {
            match view.as_ref() {
                Some(before) => selected.reread(before, &read),
                None => selected.start(&read),
            }
            notifier.read(read.waits.as_deref());
            *view = Some(read);
            Ok(true)
        }
        Err(mpsc::TryRecvError::Empty) => Ok(false),
        Err(mpsc::TryRecvError::Disconnected) => {
            Err(std::io::Error::other("the reading of the project failed"))
        }
    }
}

/// Moves `selected` to the issue `next-issue` offers in `view`, or says on `status` why not; once it moves, the
/// line drops that refusal (STU-36).
fn next(view: Option<&View>, selected: &mut Selection, status: &mut Status) {
    match view.map(|v| selected.select_next(v)) {
        Some(Err(why)) => status.set(Status::new(why, Kind::Error)),
        Some(Ok(())) => status.forget(crate::keys::NO_NEXT),
        None => {}
    }
}

/// Runs `job` on `stem` on a thread of its own and sends its result on `send`, a panic as the failed line `the task
/// panicked: <message>`, so the screen learns it ended (SUS-7); an error when the thread could not start.
fn spawn_task(
    run_task: fn(Task, &str) -> Result<String, String>,
    job: Task,
    stem: String,
    send: mpsc::Sender<Result<String, String>>,
) -> std::io::Result<()> {
    // the loop quit with an error: nobody waits for the result
    let end = move |result| drop(send.send(result));
    panics::spawn(Caught::Task, move || run_task(job, &stem), end)
}

/// Starts the task `job` on `stem`, `shown` on `status` while it runs, or says there why it did not start; whether
/// it started (SUA-6).
fn start(
    run_task: fn(Task, &str) -> Result<String, String>,
    (job, shown, stem): (Task, String, String),
    send: mpsc::Sender<Result<String, String>>,
    status: &mut Status,
) -> bool {
    match spawn_task(run_task, job, stem, send) {
        Ok(()) => {
            status.set(Status::new(shown, Kind::Other));
            true
        }
        Err(e) => {
            status.set(Status::new(
                format!("the task did not start: {e}"),
                Kind::Error,
            ));
            false
        }
    }
}

/// Runs the screen in the terminal until the user quits (STU-2): `read` gives the panels, run on a thread of its
/// own so the keys are answered during a slow read; `run_task` runs the task of `s` or `f` on the selected issue, of
/// `e` on the selected worktree, or of `v` or `E` (SUA-1, SUM-1, SUE-1, SUA-4, SUE-2), on a thread too, one at a time, its result on the status line; `if_stale` starts the measure of a missing or old one once the panic hook is set, and `measure` a measure on `m` of the charts and the measures (SUC-1),
/// which runs apart (STU-17); each read notifies the desktop of its new waits (STU-29, STU-30). Gives the result of a task the user quit during, to print once the terminal is given
/// back (SUA-6). It draws in the roles of `theme`, its status line opening on `status` (SUS-1, SUS-5). The terminal
/// is given back as it was, also on a panic of the loop or of the reads (Ratatui's hook); a task, a measure or a
/// notification that panics ends as a failed one, the screen going on (SUS-7, SUS-8).
pub fn run(
    read: impl Fn() -> View + Send + 'static,
    run_task: fn(Task, &str) -> Result<String, String>,
    (if_stale, measure): (impl FnOnce(), impl Fn()),
    (theme, status): (Theme, String),
) -> std::io::Result<Option<String>> {
    let (ask, asked) = mpsc::channel::<()>();
    let (send, read_panels) = mpsc::channel();
    std::thread::spawn(move || {
        while asked.recv().is_ok() {
            if send.send(read()).is_err() {
                break;
            }
        }
    });
    let (done, task_result) = mpsc::channel::<Result<String, String>>();
    let mut terminal = ratatui::try_init()?;
    panics::pass_over_caught();
    // after the hook, so a measure that panics at once prints nothing over the screen (SUS-8)
    if_stale();
    let result = (|| {
        let mut view: Option<View> = None;
        let mut schedule = Schedule::default();
        let mut selected = Selection::default();
        let mut status = Status::new(status, Kind::Error);
        let mut notifier = Notifier::new(notify::program());
        let mut busy = false;
        let mut quitting = false;
        let mut shown = Screen::Panels;
        let mut unfolded = false;
        let mut palette = None;
        let started = Instant::now();
        loop {
            if schedule.start(Instant::now()) && ask.send(()).is_err() {
                return Err(std::io::Error::other("the reading of the project failed"));
            }
            if received(&read_panels, &mut view, &mut selected, &mut notifier)? {
                schedule.ended(Instant::now());
            }
            if let Some(line) = notifier.failure() {
                // beside the line of a task, which it must not hide, nor the task that ends after it
                status.keep(line);
            }
            if let Ok(result) = task_result.try_recv() {
                match ended(result, quitting) {
                    Ended::Quit(line) => return Ok(Some(line)),
                    Ended::Show(line) => status.set(line),
                }
                busy = false;
                schedule.asked = true;
            }
            if let Some(v) = view.as_mut() {
                // the mark at work turns at each tick, a read or not (SUQ-4)
                spinner::turn(&mut v.panels[0], spinner::frame(started.elapsed()));
            }
            let state = (selected, shown, busy, unfolded, palette.as_ref());
            terminal.draw(|f| draw(f, view.as_ref(), state, &status, &theme))?;
            if event::poll(TICK)? {
                if let Event::Key(key) = event::read()? {
                    let pressed = crate::palette::pressed(
                        &mut palette,
                        &key,
                        view.as_ref(),
                        (selected, busy),
                    );
                    let (screen, key) = screen_key(shown, pressed);
                    shown = screen;
                    let (now, later) = quits(&key, busy, quitting);
                    if now {
                        return Ok(None);
                    }
                    if later && !quitting {
                        status.set(Status::new("quitting once the task ends…", Kind::Other));
                    }
                    quitting = later;
                    match key {
                        Action::Quit | Action::Charts => {}
                        Action::Refresh => schedule.asked = true,
                        Action::Palette if view.is_some() => palette = Some(Palette::default()),
                        Action::Next => next(view.as_ref(), &mut selected, &mut status),
                        Action::Summary => {
                            unfolded = crate::summary::toggle(unfolded, selected, view.as_ref());
                        }
                        Action::Measure => {
                            measure();
                            schedule.asked = true;
                        }
                        k if task(&k, "").is_some() => match job(&k, selected, view.as_ref()) {
                            Ok(Some(t)) if !busy => {
                                busy = start(run_task, t, done.clone(), &mut status);
                            }
                            Err(why) if !busy => status.set(Status::new(why, Kind::Error)),
                            _ => {}
                        },
                        a => {
                            if let Some(v) = &view {
                                selected.apply(&a, v);
                            }
                        }
                    }
                }
            }
        }
    })();
    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::tests::view;

    fn panics(_: Task, stem: &str) -> Result<String, String> {
        panic!("boom on {stem}")
    }

    // spec: SUS-7 (internal: the panic of a task of a screen that needs a real terminal)
    #[test]
    fn a_task_that_panics_sends_its_panic_as_a_failure() {
        let _turn = crate::panics::tests::turn();
        let (send, ended) = mpsc::channel();
        spawn_task(panics, Task::Start, "foo".to_string(), send).unwrap();
        assert_eq!(
            ended.recv_timeout(Duration::from_secs(5)),
            Ok(Err("the task panicked: boom on foo".to_string()))
        );
    }

    // spec: STU-2 (internal: the timing of the reads of a screen that needs a real terminal)
    #[test]
    fn a_read_every_two_seconds_on_r_and_one_at_a_time() {
        let t0 = Instant::now();
        let mut s = Schedule::default();
        assert!(s.start(t0), "the first read at once");
        assert!(!s.start(t0 + REFRESH * 3), "never two reads at once");
        s.asked = true;
        assert!(!s.start(t0 + REFRESH * 3), "not even on r");
        s.ended(t0 + REFRESH * 3);
        assert!(s.start(t0 + REFRESH * 3), "the r asked during the read");
        s.ended(t0 + REFRESH * 4);
        assert!(!s.start(t0 + REFRESH * 4 + Duration::from_millis(1999)));
        assert!(
            s.start(t0 + REFRESH * 5),
            "2 s after the end of the last read"
        );
        assert_eq!(REFRESH, Duration::from_secs(2));
    }

    // spec: STU-36 (internal: the status line of a screen that needs a real terminal)
    #[test]
    fn n_says_why_it_moves_nothing_until_it_moves() {
        let mut v = view(&["a", "b"], &[]);
        let mut sel = Selection::default();
        let mut status = Status::default();
        next(Some(&v), &mut sel, &mut status);
        assert_eq!(
            status,
            Status::new("n: no issue is ready to start", Kind::Error)
        );
        v.next = Some("b".to_string());
        next(Some(&v), &mut sel, &mut status);
        assert_eq!((sel.row, status.is_empty()), (1, true));
    }
}
