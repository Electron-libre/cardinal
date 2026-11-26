//! The terminal loop of `cardinal-ui` (STU-2): the reads of the project on a thread of their own, every `REFRESH` and
//! on `r`, and the keys of the screen drawn by `screen.rs`: an issue or a milestone of the queue is selected
//! (STU-14), `s` starts its session (SUA-1), `f` merges it into the trunk (SUM-1); `Tab` moves the selection to the
//! worktrees, where `e` ends what a merged issue left (SUE-1); `E` ends it on every merged worktree (SUE-2); `v`
//! starts the release session (SUA-4), `p` the review of the priorities (SUA-16), `c` the cut of the selected
//! milestone (SUA-19), `a` the audit of the code (SUA-23), each task on a thread of its own; a worktree that starts to wait for the user notifies the
//! desktop (STU-29); `g` switches to the charts of the measures, then to their table, where `m` takes the measures
//! again (STU-17), and back (SUC-1); the mark of an issue at work turns at each tick (SUQ-4); `i` unfolds the summary
//! of the selected issue (SUQ-6); `o` and `O` give the terminal to the viewer or the editor of its document (SUD-1, SUD-2); `n` selects the issue `next-issue` offers (STU-36); `:` opens the command palette
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

/// The job of `action` (see [`job`]) once no task runs; while one runs (`busy`), nothing: neither a start nor
/// the refusal of one (SUA-6).
fn launch(
    action: &Action,
    sel: Selection,
    v: Option<&View>,
    busy: bool,
) -> Result<Option<(Task, String, String)>, String> {
    if busy {
        return Ok(None);
    }
    job(action, sel, v)
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

/// Starts the thread of the reads (STU-2): each `()` sent on the first channel runs `read` once, its view sent on the
/// second; it ends once either channel is dropped.
fn reader(read: impl Fn() -> View + Send + 'static) -> (mpsc::Sender<()>, mpsc::Receiver<View>) {
    let (ask, asked) = mpsc::channel::<()>();
    let (send, read_panels) = mpsc::channel();
    std::thread::spawn(move || {
        while asked.recv().is_ok() {
            if send.send(read()).is_err() {
                break;
            }
        }
    });
    (ask, read_panels)
}

/// Gives the terminal to `run` (a viewer or an editor, SUD-1): the screen leaves it as [`ratatui::restore`] does,
/// then takes it back once `run` returns, raw and on the alternate screen, cleared so the next draw is whole (ADR 0034).
fn hand_over<T>(
    terminal: &mut ratatui::DefaultTerminal,
    run: impl FnOnce() -> T,
) -> std::io::Result<T> {
    use ratatui::crossterm::{execute, terminal as tty};
    ratatui::restore();
    // the draws hid it, and an editor that does not show it again would hide where it edits
    execute!(std::io::stdout(), ratatui::crossterm::cursor::Show)?;
    let ran = run();
    tty::enable_raw_mode()?;
    execute!(std::io::stdout(), tty::EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(ran)
}

/// Runs the viewer or the editor of `action` on the document of what `sel` selects in `v`, the terminal handed to it,
/// its failure or why it runs none on `status` (SUD-1, SUD-2, SUD-7); whether it ran, so the screen reads the project
/// again and an edit shows at once (SUD-2). It runs whether a task runs or not (SUD-8).
fn open(
    terminal: &mut ratatui::DefaultTerminal,
    action: &Action,
    (sel, v): (Selection, Option<&View>),
    status: &mut Status,
) -> std::io::Result<bool> {
    let vars = crate::document::Vars::from_env();
    match crate::document::command(action, sel, v, &vars) {
        Some(Ok(words)) => {
            if let Some(line) = hand_over(terminal, || crate::document::run(&words))? {
                status.set(line);
            }
            Ok(true)
        }
        Some(Err(why)) => {
            status.set(Status::new(why, Kind::Error));
            Ok(false)
        }
        None => Ok(false),
    }
}

/// Runs the screen in the terminal until the user quits (STU-2): `read` gives the panels, run on a thread of its
/// own so the keys are answered during a slow read; `run_task` runs the task of `s` or `f` on the selected issue, of
/// `e` on the selected worktree, of `c` on the selected milestone, or of `v`, `p`, `a` or `E` (SUA-1, SUM-1, SUE-1, SUA-19, SUA-4, SUA-16, SUA-23, SUE-2), on a thread too, one at a time, its result on the status line; `if_stale` starts the measure of a missing or old one once the panic hook is set, and `measure` a measure on `m` of the charts and the measures (SUC-1),
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
    let (ask, read_panels) = reader(read);
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
                        k @ (Action::Open | Action::Edit) => {
                            let on = (selected, view.as_ref());
                            schedule.asked |= open(&mut terminal, &k, on, &mut status)?;
                        }
                        k if task(&k, "").is_some() => {
                            match launch(&k, selected, view.as_ref(), busy) {
                                Ok(Some(t)) => busy = start(run_task, t, done.clone(), &mut status),
                                Err(why) => status.set(Status::new(why, Kind::Error)),
                                Ok(None) => {}
                            }
                        }
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
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    // spec: SUA-6 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn no_key_starts_a_task_nor_shows_its_refusal_while_one_runs() {
        let v = view(&["foo"], &[("/w/foo", Some("foo")), ("/w/main", None)]);
        let sel = Selection::default();
        for c in ['s', 'f', 'e', 'c', 'E', 'v', 'p', 'a'] {
            let key = crate::keys::action(&KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
            assert!(task(&key, "").is_some(), "{c} runs a task");
            assert_eq!(
                launch(&key, sel, Some(&v), true),
                Ok(None),
                "{c} while busy"
            );
        }
        assert!(
            matches!(
                launch(&Action::Audit, sel, Some(&v), false),
                Ok(Some((Task::Audit, _, _)))
            ),
            "a starts the audit once none runs"
        );
        assert!(
            launch(&Action::End, sel, Some(&v), false).is_err(),
            "e on the queue is refused"
        );
    }

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

    /// A read whose issue `foo` has the document `<dir>/foo.md`.
    fn with_document(dir: &std::path::Path) -> View {
        let doc = dir.join("foo.md");
        std::fs::write(&doc, "# Foo\n").unwrap();
        let mut v = view(&["foo"], &[("/w/main", None)]);
        v.documents = vec![Ok(doc)];
        v
    }

    // spec: SUD-8 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn o_and_o_are_no_task_and_act_while_one_runs() {
        let dir = tempfile::tempdir().unwrap();
        let v = with_document(dir.path());
        let vars = crate::document::Vars {
            viewer: Some("view".to_string()),
            visual: Some("edit".to_string()),
            ..Default::default()
        };
        for a in [Action::Open, Action::Edit] {
            assert_eq!(task(&a, ""), None, "{a:?} is no task");
            assert_eq!(launch(&a, Selection::default(), Some(&v), true), Ok(None));
            let words = crate::document::command(&a, Selection::default(), Some(&v), &vars);
            assert!(matches!(words, Some(Ok(_))), "{a:?} while busy: {words:?}");
        }
    }

    // spec: SUD-H1 (internal: the program takes the terminal of a screen that needs a real terminal)
    #[test]
    fn o_runs_the_viewer_on_the_file_of_the_selected_issue() {
        let dir = tempfile::tempdir().unwrap();
        let v = with_document(dir.path());
        let glow = crate::document::tests::recorder(dir.path(), "glow", 0);
        let vars = crate::document::Vars {
            path: Some(dir.path().as_os_str().to_owned()),
            ..Default::default()
        };
        let key = crate::keys::action(&KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
        let words = crate::document::command(&key, Selection::default(), Some(&v), &vars);
        let words = words.unwrap().unwrap();
        assert_eq!(words[..2], ["glow", "-p"]);
        let mut found = words.clone();
        found[0] = glow;
        assert_eq!(crate::document::run(&found), None);
        let args = std::fs::read_to_string(dir.path().join("glow.args")).unwrap();
        assert_eq!(
            args.trim(),
            format!("-p {}", dir.path().join("foo.md").display())
        );
    }

    // spec: SUD-H2 (internal: the program takes the terminal of a screen that needs a real terminal)
    #[test]
    fn the_palette_edits_the_roadmap_document_of_the_selected_milestone() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("ROADMAP.md"), "## One\n").unwrap();
        let mut v = view(&[], &[]);
        v.rows = vec![crate::keys::Row::Milestone("One".to_string())];
        let one = crate::document::tests::milestone("One", Some("ROADMAP.md"));
        v.documents = crate::document::of(dir.path(), None, &[one], &v.rows);
        let vi = crate::document::tests::recorder(dir.path(), "vi", 0);
        let vars = crate::document::Vars {
            editor: Some(vi),
            ..Default::default()
        };
        let mut palette = Some(Palette::default());
        let mut action = Action::Nothing;
        for code in "edit".chars().map(KeyCode::Char).chain([KeyCode::Enter]) {
            let key = KeyEvent::new(code, KeyModifiers::NONE);
            action = crate::palette::pressed(
                &mut palette,
                &key,
                Some(&v),
                (Selection::default(), false),
            );
        }
        assert_eq!((action.clone(), palette), (Action::Edit, None));
        let words = crate::document::command(&action, Selection::default(), Some(&v), &vars);
        assert_eq!(crate::document::run(&words.unwrap().unwrap()), None);
        let args = std::fs::read_to_string(dir.path().join("vi.args")).unwrap();
        assert_eq!(
            args.trim(),
            dir.path().join("ROADMAP.md").display().to_string()
        );
    }
}
