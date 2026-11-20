//! The screen of `cardinal-ui` (STU-1): the panels drawn with Ratatui, in a terminal (STU-2) or once as text
//! (STU-4); in the terminal, an issue of the queue is selected, `s` starts its session (STU-9), `f` merges it into the
//! trunk (STU-20); `Tab` moves the selection to the worktrees, where `e` ends what a merged issue left (STU-22); `m`
//! takes the measures again (STU-17); `v` starts the release session (STU-26).

use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::panels::Panel;

/// How often the screen reads the project again (STU-2).
pub const REFRESH: Duration = Duration::from_secs(2);

/// How long the loop waits for a key before it looks for a finished read.
const TICK: Duration = Duration::from_millis(100);

/// The width of the text of `--print`, unless a line needs more (STU-4).
const PRINT_WIDTH: u16 = 100;

/// The help line under the panels.
const HELP: &str =
    "q quit  r refresh  ↑↓ select  tab panel  s start  f merge  e end  v release  m measure";

/// What a read gives: the panels, the stems of the issues of the queue and the path and branch of the worktrees, in
/// the order of the lines of their panels.
pub struct View {
    pub panels: [Panel; 4],
    pub stems: Vec<String>,
    pub worktrees: Vec<(String, Option<String>)>,
}

/// The panel the selection is in (STU-22).
#[derive(Debug, PartialEq, Clone, Copy)]
enum Focus {
    Queue,
    Worktrees,
}

/// The selected line of the queue and of the worktrees, and the panel the keys move it in (STU-9, STU-22).
#[derive(Debug, Clone, Copy)]
struct Selection {
    focus: Focus,
    issue: usize,
    worktree: usize,
}

impl Selection {
    /// Moves the selection on `action` in the view `v`: `Tab` to the other panel, the arrows within it.
    fn apply(&mut self, action: &Action, v: &View) {
        match (action, self.focus) {
            (Action::Switch, Focus::Queue) => self.focus = Focus::Worktrees,
            (Action::Switch, Focus::Worktrees) => self.focus = Focus::Queue,
            (_, Focus::Queue) => self.issue = select(self.issue, v.stems.len(), action),
            (_, Focus::Worktrees) => {
                self.worktree = select(self.worktree, v.worktrees.len(), action);
            }
        }
    }

    /// Keeps the selection on the same issue and the same worktree path from the read `before` to the read `after`
    /// (STU-14, STU-22).
    fn reread(&mut self, before: &View, after: &View) {
        self.issue = reselect(&before.stems, self.issue, &after.stems);
        let paths = |v: &View| v.worktrees.iter().map(|w| w.0.clone()).collect::<Vec<_>>();
        self.worktree = reselect(&paths(before), self.worktree, &paths(after));
    }

    /// The panel and the line to highlight in `v`: none in an empty or failed panel.
    fn line(&self, v: &View) -> Option<(usize, usize)> {
        match self.focus {
            Focus::Queue => (!v.stems.is_empty()).then_some((1, self.issue)),
            Focus::Worktrees => (!v.worktrees.is_empty()).then_some((2, self.worktree)),
        }
    }
}

/// The rows a panel takes: its lines and its two borders.
fn height(p: &Panel) -> u16 {
    u16::try_from(p.lines.len())
        .unwrap_or(u16::MAX)
        .saturating_add(2)
}

/// Renders the panels into `buf` over `area`: the milestones, the worktrees and the measures at their height, the
/// queue in the rest, the line `selected` (a panel and a line) highlighted, the help line and `status` last.
fn render(
    area: Rect,
    buf: &mut Buffer,
    panels: &[Panel; 4],
    selected: Option<(usize, usize)>,
    status: &str,
) {
    let [milestones, queue, worktrees, measures] = panels;
    let areas = Layout::vertical([
        Constraint::Max(height(milestones)),
        Constraint::Min(height(queue).min(3)),
        Constraint::Max(height(worktrees)),
        Constraint::Max(height(measures)),
        Constraint::Length(1),
    ])
    .split(area);
    for (n, (p, area)) in panels.iter().zip(areas.iter()).enumerate() {
        let scroll = match selected {
            Some((panel, line)) if panel == n => offset(line, area.height.saturating_sub(2)),
            _ => 0,
        };
        let lines: Vec<Line> = p
            .lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let line = Line::raw(l.as_str());
                if selected == Some((n, i)) {
                    line.style(Style::new().add_modifier(Modifier::REVERSED))
                } else {
                    line
                }
            })
            .collect();
        let block = Block::bordered().title(format!(" {} ", p.title));
        Paragraph::new(lines)
            .block(block)
            .scroll((scroll, 0))
            .render(*area, buf);
    }
    let help = if status.is_empty() {
        HELP.to_string()
    } else {
        format!("{HELP}  {status}")
    };
    Line::raw(help).render(areas[4], buf);
}

/// The screen of `panels` as text, whole, as wide as `PRINT_WIDTH` or its longest line in display columns, each
/// row without its trailing spaces (STU-4). Rendered straight into a buffer: a terminal of the test backend
/// mixes up the cells past the 65,535th.
pub fn text(panels: &[Panel; 4]) -> String {
    let longest = panels
        .iter()
        .flat_map(|p| p.lines.iter())
        .map(|l| Line::raw(l.as_str()).width().saturating_add(2))
        .max()
        .unwrap_or(0);
    let width = PRINT_WIDTH.max(u16::try_from(longest).unwrap_or(u16::MAX));
    let rows = panels.iter().map(height).fold(1u16, u16::saturating_add);
    let area = Rect::new(0, 0, width, rows);
    let mut buf = Buffer::empty(area);
    render(area, &mut buf, panels, None, "");
    let mut out = String::new();
    for y in 0..rows {
        let mut row = String::new();
        let mut x = 0;
        while x < width {
            let symbol = buf.cell((x, y)).map_or(" ", ratatui::buffer::Cell::symbol);
            row.push_str(symbol);
            // a wide character covers the next cells, which hold no symbol of their own
            let columns = u16::try_from(Line::raw(symbol).width()).unwrap_or(1).max(1);
            x = x.saturating_add(columns);
        }
        out.push_str(row.trim_end());
        out.push('\n');
    }
    out
}

/// What a key does in the screen (STU-2).
#[derive(Debug, PartialEq)]
pub enum Action {
    Quit,
    Refresh,
    Up,
    Down,
    Switch,
    Start,
    Merge,
    End,
    Release,
    Measure,
    Nothing,
}

/// A task on the selected issue that changes the project, one at a time (STU-15).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Task {
    Start,
    Merge,
    End,
    Release,
}

/// The task `action` runs on the issue `stem`, with the status line shown while it runs.
fn task(action: &Action, stem: &str) -> Option<(Task, String)> {
    match action {
        Action::Start => Some((Task::Start, format!("starting {stem}…"))),
        Action::Merge => Some((Task::Merge, format!("merging {stem}…"))),
        Action::End => Some((Task::End, format!("ending {stem}…"))),
        Action::Release => Some((Task::Release, "starting the release…".to_string())),
        _ => None,
    }
}

/// The task `action` runs on what `sel` selects in `v`: the task, the status line while it runs and the stem;
/// `None` for another key or an empty panel; why it runs none when the key belongs to the other panel or the worktree
/// is detached (STU-9, STU-20, STU-22).
fn job(
    action: &Action,
    sel: Selection,
    v: &View,
) -> Result<Option<(Task, String, String)>, String> {
    let stem = match (action, sel.focus) {
        // the release covers every milestone awaiting it: no selection
        (Action::Release, _) => Some(String::new()),
        (Action::Start | Action::Merge, Focus::Queue) => v.stems.get(sel.issue).cloned(),
        (Action::Start | Action::Merge, Focus::Worktrees) => {
            return Err("s and f act on the queue: Tab to it".to_string())
        }
        (Action::End, Focus::Worktrees) => match v.worktrees.get(sel.worktree) {
            Some((path, None)) => return Err(format!("{path}: detached, no issue to end")),
            Some((_, Some(branch))) => Some(branch.clone()),
            None => None,
        },
        (Action::End, Focus::Queue) => {
            return Err("e acts on a worktree: Tab to the worktrees".to_string())
        }
        _ => return Ok(None),
    };
    Ok(stem.and_then(|s| task(action, &s).map(|(t, shown)| (t, shown, s))))
}

/// The action of the key `key`.
pub fn action(key: &KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::Nothing;
    }
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Up | KeyCode::Char('k') => Action::Up,
        KeyCode::Down | KeyCode::Char('j') => Action::Down,
        KeyCode::Tab => Action::Switch,
        KeyCode::Char('s') => Action::Start,
        KeyCode::Char('f') => Action::Merge,
        KeyCode::Char('e') => Action::End,
        KeyCode::Char('v') => Action::Release,
        KeyCode::Char('m') => Action::Measure,
        _ => Action::Nothing,
    }
}

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

/// The first line of the queue shown in `rows` rows, so the line `selected` is visible (STU-14).
fn offset(selected: usize, rows: u16) -> u16 {
    let first = selected
        .saturating_add(1)
        .saturating_sub(usize::from(rows.max(1)));
    u16::try_from(first).unwrap_or(u16::MAX)
}

/// The selection in the queue `stems` of a new read, given the one `selected` in the queue `before`: the line of the
/// same issue, else the same line kept in the queue (STU-14).
fn reselect(before: &[String], selected: usize, stems: &[String]) -> usize {
    before
        .get(selected)
        .and_then(|stem| stems.iter().position(|s| s == stem))
        .unwrap_or_else(|| select(selected, stems.len(), &Action::Nothing))
}

/// Whether the screen quits now on `action`, and whether it quits once the task running (`busy`) ends (STU-15).
fn quits(action: &Action, busy: bool, quitting: bool) -> (bool, bool) {
    match action {
        Action::Quit if busy => (false, true),
        Action::Quit => (true, quitting),
        _ => (false, quitting),
    }
}

/// What the end of a task gives (STU-15).
#[derive(Debug, PartialEq)]
enum Ended {
    /// The line the status line shows.
    Show(String),
    /// The user quit during the task: the screen quits, and this line is printed once the terminal is given back.
    Quit(String),
}

/// The end of a task with `result`, the user having quit during it or not (STU-15).
fn ended(result: Result<String, String>, quitting: bool) -> Ended {
    let line = result.unwrap_or_else(|e| e);
    if quitting {
        Ended::Quit(line)
    } else {
        Ended::Show(line)
    }
}

/// The selection after `action` in a queue of `len` issues, kept in it (STU-9).
fn select(selected: usize, len: usize, action: &Action) -> usize {
    let moved = match action {
        Action::Up => selected.saturating_sub(1),
        Action::Down => selected.saturating_add(1),
        _ => selected,
    };
    moved.min(len.saturating_sub(1))
}

/// Runs the screen in the terminal until the user quits (STU-2): `read` gives the panels, run on a thread of its
/// own so the keys are answered during a slow read; `run_task` runs the task of `s` or `f` on the selected issue, or
/// of `e` on the selected worktree (STU-9, STU-20, STU-22), on a thread too, one at a time, its result on the status line; `measure` starts a measure on `m`,
/// which runs apart (STU-17). Gives the result of a task the user quit during, to print once the terminal is given
/// back (STU-15). The terminal is given back as it was, also on a panic (Ratatui's hook).
pub fn run(
    read: impl Fn() -> View + Send + 'static,
    run_task: fn(Task, &str) -> Result<String, String>,
    measure: impl Fn(),
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
    let result = (|| {
        let mut view: Option<View> = None;
        let mut schedule = Schedule::default();
        let mut selected = Selection {
            focus: Focus::Queue,
            issue: 0,
            worktree: 0,
        };
        let mut status = String::new();
        let mut busy = false;
        let mut quitting = false;
        loop {
            if schedule.start(Instant::now()) && ask.send(()).is_err() {
                return Err(std::io::Error::other("the reading of the project failed"));
            }
            match read_panels.try_recv() {
                Ok(read) => {
                    if let Some(before) = &view {
                        selected.reread(before, &read);
                    }
                    view = Some(read);
                    schedule.ended(Instant::now());
                }
                Err(mpsc::TryRecvError::Empty) => {}
                // the thread of the reads died (a panic): no read would ever come again
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(std::io::Error::other("the reading of the project failed"));
                }
            }
            if let Ok(result) = task_result.try_recv() {
                match ended(result, quitting) {
                    Ended::Quit(line) => return Ok(Some(line)),
                    Ended::Show(line) => status = line,
                }
                busy = false;
                schedule.asked = true;
            }
            terminal.draw(|f| match &view {
                // a panel holds one line per stem or worktree, in order: an empty or failed one selects nothing
                Some(v) => render(
                    f.area(),
                    f.buffer_mut(),
                    &v.panels,
                    selected.line(v),
                    &status,
                ),
                None => Line::raw("reading the project…").render(f.area(), f.buffer_mut()),
            })?;
            if event::poll(TICK)? {
                if let Event::Key(key) = event::read()? {
                    let key = action(&key);
                    let (now, later) = quits(&key, busy, quitting);
                    if now {
                        return Ok(None);
                    }
                    if later && !quitting {
                        status = "quitting once the task ends…".to_string();
                    }
                    quitting = later;
                    match key {
                        Action::Quit => {}
                        Action::Refresh => schedule.asked = true,
                        Action::Measure => {
                            measure();
                            schedule.asked = true;
                        }
                        Action::Start | Action::Merge | Action::End | Action::Release => {
                            match view.as_ref().map(|v| job(&key, selected, v)) {
                                Some(Ok(Some((job, shown, stem)))) if !busy => {
                                    busy = true;
                                    status = shown;
                                    let send = done.clone();
                                    std::thread::spawn(move || send.send(run_task(job, &stem)));
                                }
                                Some(Err(why)) if !busy => status = why,
                                _ => {}
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

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    // spec: STU-2, STU-17 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn q_esc_and_ctrl_c_quit_and_r_refreshes() {
        assert_eq!(action(&press(KeyCode::Char('q'))), Action::Quit);
        assert_eq!(action(&press(KeyCode::Esc)), Action::Quit);
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(action(&ctrl_c), Action::Quit);
        assert_eq!(action(&press(KeyCode::Char('c'))), Action::Nothing);
        assert_eq!(action(&press(KeyCode::Char('r'))), Action::Refresh);
        assert_eq!(action(&press(KeyCode::Char('m'))), Action::Measure);
        assert_eq!(action(&press(KeyCode::Char('x'))), Action::Nothing);
        let mut release = press(KeyCode::Char('q'));
        release.kind = KeyEventKind::Release;
        assert_eq!(action(&release), Action::Nothing);
    }

    // spec: STU-9 (internal: the keys and the selection of a screen that needs a real terminal)
    #[test]
    fn the_arrows_and_j_k_move_the_selection_and_s_starts() {
        assert_eq!(action(&press(KeyCode::Up)), Action::Up);
        assert_eq!(action(&press(KeyCode::Char('k'))), Action::Up);
        assert_eq!(action(&press(KeyCode::Down)), Action::Down);
        assert_eq!(action(&press(KeyCode::Char('j'))), Action::Down);
        assert_eq!(action(&press(KeyCode::Char('s'))), Action::Start);
        assert_eq!(select(0, 3, &Action::Down), 1);
        assert_eq!(select(2, 3, &Action::Down), 2, "kept on the last issue");
        assert_eq!(select(0, 3, &Action::Up), 0, "kept on the first issue");
        assert_eq!(
            select(5, 2, &Action::Nothing),
            1,
            "a shorter queue moves it back"
        );
        assert_eq!(select(0, 0, &Action::Down), 0);
    }

    // spec: STU-20 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn f_merges_the_selected_issue() {
        assert_eq!(action(&press(KeyCode::Char('f'))), Action::Merge);
        assert!(HELP.contains("f merge"), "{HELP}");
        assert_eq!(
            task(&Action::Merge, "foo").map(|(_, s)| s),
            Some("merging foo…".to_string())
        );
        assert_eq!(
            task(&Action::Start, "foo").map(|(_, s)| s),
            Some("starting foo…".to_string())
        );
        assert!(task(&Action::Refresh, "foo").is_none());
    }

    // spec: STU-26, STU-15 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn v_starts_the_release_from_either_panel() {
        assert_eq!(action(&press(KeyCode::Char('v'))), Action::Release);
        assert!(HELP.contains("v release"), "{HELP}");
        let v = view(&["bar"], &[("/main", Some("trunk"))]);
        for focus in [Focus::Queue, Focus::Worktrees] {
            let sel = Selection {
                focus,
                issue: 0,
                worktree: 0,
            };
            assert_eq!(
                job(&Action::Release, sel, &v),
                Ok(Some((
                    Task::Release,
                    "starting the release…".to_string(),
                    String::new()
                )))
            );
        }
        assert_eq!(quits(&Action::Quit, true, false), (false, true));
    }

    // spec: STU-9 (internal: the highlight of a screen that needs a real terminal)
    #[test]
    fn the_selected_issue_is_highlighted_and_the_status_shown() {
        let panel = |title, lines: &[&str]| Panel {
            title,
            lines: lines.iter().map(|l| (*l).to_string()).collect(),
            failed: false,
        };
        let panels = [
            panel("Milestones", &["m"]),
            panel("Queue", &["> b", "  a"]),
            panel("Worktrees", &["w"]),
            panel("Measures", &["x"]),
        ];
        let area = Rect::new(0, 0, 120, 15);
        let mut buf = Buffer::empty(area);
        render(area, &mut buf, &panels, Some((1, 1)), "started a in /w");
        let row = |y: u16| (0..120).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let a = (0..15).find(|y| row(*y).contains("  a")).unwrap();
        let b = (0..15).find(|y| row(*y).contains("> b")).unwrap();
        assert!(buf[(3, a)].modifier.contains(Modifier::REVERSED));
        assert!(!buf[(3, b)].modifier.contains(Modifier::REVERSED));
        assert!((0..15).any(|y| row(y).contains("m measure  started a in /w")));
    }

    // spec: STU-14 (internal: the selection of a screen that needs a real terminal)
    #[test]
    fn the_selection_follows_its_issue_and_stays_visible() {
        let v = |s: &[&str]| s.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
        assert_eq!(reselect(&v(&["b", "foo", "c"]), 1, &v(&["foo", "c"])), 0);
        assert_eq!(
            reselect(&v(&["b", "foo", "c"]), 1, &v(&["b", "c"])),
            1,
            "gone: same line"
        );
        assert_eq!(
            reselect(&v(&["b", "foo"]), 1, &v(&["b"])),
            0,
            "gone: kept in the queue"
        );
        assert_eq!(reselect(&[], 0, &v(&["a"])), 0);
        assert_eq!(offset(5, 15), 0);
        assert_eq!(offset(29, 15), 15);
        assert_eq!(offset(3, 0), 3);
        let panel = |title, lines: Vec<String>| Panel {
            title,
            lines,
            failed: false,
        };
        let queue = (1..=40).map(|i| format!("  issue{i:02}")).collect();
        let panels = [
            panel("Milestones", vec!["m".to_string()]),
            panel("Queue", queue),
            panel("Worktrees", vec!["w".to_string()]),
            panel("Measures", vec!["x".to_string()]),
        ];
        let area = Rect::new(0, 0, 40, 22);
        let mut buf = Buffer::empty(area);
        render(area, &mut buf, &panels, Some((1, 29)), "");
        let row = |y: u16| (0..40).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let y = (0..22)
            .find(|y| row(*y).contains("issue30"))
            .expect("the selected issue is shown");
        assert!(buf[(3, y)].modifier.contains(Modifier::REVERSED));
    }

    // spec: STU-15 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn quit_during_a_start_or_a_merge_waits_for_it() {
        assert_eq!(quits(&Action::Quit, false, false), (true, false));
        assert_eq!(quits(&Action::Quit, true, false), (false, true));
        assert_eq!(quits(&Action::Refresh, true, true), (false, true));
        assert_eq!(quits(&Action::Down, false, false), (false, false));
        assert_eq!(
            ended(Ok("merged".into()), false),
            Ended::Show("merged".into())
        );
        assert_eq!(
            ended(Err("merged; cardinal check: exit 1".into()), true),
            Ended::Quit("merged; cardinal check: exit 1".into()),
            "printed once the terminal is given back"
        );
    }

    fn view(stems: &[&str], worktrees: &[(&str, Option<&str>)]) -> View {
        let panel = |title| Panel {
            title,
            lines: Vec::new(),
            failed: false,
        };
        View {
            panels: [
                panel("Milestones"),
                panel("Queue"),
                panel("Worktrees"),
                panel("Measures"),
            ],
            stems: stems.iter().map(|s| (*s).to_string()).collect(),
            worktrees: worktrees
                .iter()
                .map(|(p, b)| ((*p).to_string(), b.map(str::to_string)))
                .collect(),
        }
    }

    // spec: STU-22 (internal: the keys and the selection of a screen that needs a real terminal)
    #[test]
    fn tab_moves_to_the_worktrees_where_e_ends_the_selected_one() {
        assert_eq!(action(&press(KeyCode::Tab)), Action::Switch);
        assert_eq!(action(&press(KeyCode::Char('e'))), Action::End);
        assert!(
            HELP.contains("tab panel") && HELP.contains("e end"),
            "{HELP}"
        );
        let v = view(
            &["bar"],
            &[("/main", Some("trunk")), ("/ws", Some("foo")), ("/d", None)],
        );
        let on = |focus, worktree| Selection {
            focus,
            issue: 0,
            worktree,
        };
        assert_eq!(
            job(&Action::End, on(Focus::Worktrees, 1), &v),
            Ok(Some((
                Task::End,
                "ending foo…".to_string(),
                "foo".to_string()
            )))
        );
        assert_eq!(
            job(&Action::Start, on(Focus::Queue, 1), &v),
            Ok(Some((
                Task::Start,
                "starting bar…".to_string(),
                "bar".to_string()
            )))
        );
        assert!(job(&Action::End, on(Focus::Queue, 1), &v).is_err_and(|e| e.contains("Tab")));
        assert!(job(&Action::Merge, on(Focus::Worktrees, 1), &v).is_err_and(|e| e.contains("Tab")));
        assert!(job(&Action::End, on(Focus::Worktrees, 2), &v)
            .is_err_and(|e| e.contains("/d: detached")));
        assert_eq!(job(&Action::Refresh, on(Focus::Queue, 0), &v), Ok(None));
        let mut sel = on(Focus::Queue, 0);
        sel.apply(&Action::Switch, &v);
        assert_eq!(sel.focus, Focus::Worktrees);
        sel.apply(&Action::Down, &v);
        sel.apply(&Action::Down, &v);
        sel.apply(&Action::Down, &v);
        assert_eq!((sel.issue, sel.worktree), (0, 2), "kept in the worktrees");
        sel.apply(&Action::Switch, &v);
        assert_eq!(sel.focus, Focus::Queue);
    }

    // spec: STU-22 (internal: the selection of a screen that needs a real terminal)
    #[test]
    fn the_selected_worktree_follows_its_path_and_is_highlighted() {
        let before = view(&[], &[("/main", Some("trunk")), ("/ws", Some("foo"))]);
        let after = view(
            &[],
            &[
                ("/main", Some("trunk")),
                ("/a", Some("a")),
                ("/ws", Some("foo")),
            ],
        );
        let mut sel = Selection {
            focus: Focus::Worktrees,
            issue: 0,
            worktree: 1,
        };
        sel.reread(&before, &after);
        assert_eq!(sel.worktree, 2);
        let panel = |title, lines: &[&str]| Panel {
            title,
            lines: lines.iter().map(|l| (*l).to_string()).collect(),
            failed: false,
        };
        let panels = [
            panel("Milestones", &["m"]),
            panel("Queue", &["> b"]),
            panel("Worktrees", &["/main", "/ws"]),
            panel("Measures", &["x"]),
        ];
        let area = Rect::new(0, 0, 80, 15);
        let mut buf = Buffer::empty(area);
        render(area, &mut buf, &panels, Some((2, 1)), "");
        let row = |y: u16| (0..80).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let ws = (0..15).find(|y| row(*y).contains("/ws")).unwrap();
        let b = (0..15).find(|y| row(*y).contains("> b")).unwrap();
        assert!(buf[(3, ws)].modifier.contains(Modifier::REVERSED));
        assert!(!buf[(3, b)].modifier.contains(Modifier::REVERSED));
    }

    // spec: STU-22 (internal: the layout of a screen that needs a real terminal)
    #[test]
    fn the_selected_worktree_stays_visible_in_a_short_panel() {
        let panel = |title, lines: Vec<String>| Panel {
            title,
            lines,
            failed: false,
        };
        let worktrees = (1..=20).map(|i| format!("/ws{i:02}")).collect();
        let panels = [
            panel("Milestones", vec!["m".to_string()]),
            panel("Queue", vec!["> b".to_string()]),
            panel("Worktrees", worktrees),
            panel("Measures", vec!["x".to_string()]),
        ];
        let area = Rect::new(0, 0, 40, 16);
        let mut buf = Buffer::empty(area);
        render(area, &mut buf, &panels, Some((2, 17)), "");
        let row = |y: u16| (0..40).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let y = (0..16)
            .find(|y| row(*y).contains("/ws18"))
            .expect("the selected worktree is shown");
        assert!(buf[(3, y)].modifier.contains(Modifier::REVERSED));
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
}
