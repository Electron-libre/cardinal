//! The screen of `cardinal-ui` (STU-1): the panels drawn with Ratatui, in a terminal (STU-2) or once as text
//! (STU-4); in the terminal, an issue of the queue is selected and `s` starts its session (STU-9), and `m` takes the
//! measures again (STU-17).

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
const HELP: &str = "q quit  r refresh  ↑↓ select  s start  m measure";

/// What a read gives: the panels, and the stems of the issues of the queue in the order of its lines.
pub struct View {
    pub panels: [Panel; 4],
    pub stems: Vec<String>,
}

/// The rows a panel takes: its lines and its two borders.
fn height(p: &Panel) -> u16 {
    u16::try_from(p.lines.len())
        .unwrap_or(u16::MAX)
        .saturating_add(2)
}

/// Renders the panels into `buf` over `area`: the milestones, the worktrees and the measures at their height, the
/// queue in the rest with its line `selected` highlighted, the help line and `status` last.
fn render(
    area: Rect,
    buf: &mut Buffer,
    panels: &[Panel; 4],
    selected: Option<usize>,
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
            Some(line) if n == 1 => offset(line, area.height.saturating_sub(2)),
            _ => 0,
        };
        let lines: Vec<Line> = p
            .lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let line = Line::raw(l.as_str());
                if n == 1 && selected == Some(i) {
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
    Start,
    Measure,
    Nothing,
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
        KeyCode::Char('s') => Action::Start,
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

/// Whether the screen quits now on `action`, and whether it quits once the start running (`starting`) ends (STU-15).
fn quits(action: &Action, starting: bool, quitting: bool) -> (bool, bool) {
    match action {
        Action::Quit if starting => (false, true),
        Action::Quit => (true, quitting),
        _ => (false, quitting),
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
/// own so the keys are answered during a slow read; `start` starts the session of the selected issue on `s`, on a
/// thread too, its result on the status line (STU-9); `measure` starts a measure on `m`, which runs apart (STU-17).
/// The terminal is given back as it was, also on a panic (Ratatui's hook).
pub fn run(
    read: impl Fn() -> View + Send + 'static,
    start: fn(&str) -> Result<String, String>,
    measure: impl Fn(),
) -> std::io::Result<()> {
    let (ask, asked) = mpsc::channel::<()>();
    let (send, read_panels) = mpsc::channel();
    std::thread::spawn(move || {
        while asked.recv().is_ok() {
            if send.send(read()).is_err() {
                break;
            }
        }
    });
    let (started, start_result) = mpsc::channel::<Result<String, String>>();
    let mut terminal = ratatui::try_init()?;
    let result = (|| {
        let mut view: Option<View> = None;
        let mut schedule = Schedule::default();
        let mut selected = 0;
        let mut status = String::new();
        let mut starting = false;
        let mut quitting = false;
        loop {
            if schedule.start(Instant::now()) && ask.send(()).is_err() {
                return Err(std::io::Error::other("the reading of the project failed"));
            }
            match read_panels.try_recv() {
                Ok(read) => {
                    let before = view.as_ref().map_or(&[][..], |v| v.stems.as_slice());
                    selected = reselect(before, selected, &read.stems);
                    view = Some(read);
                    schedule.ended(Instant::now());
                }
                Err(mpsc::TryRecvError::Empty) => {}
                // the thread of the reads died (a panic): no read would ever come again
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(std::io::Error::other("the reading of the project failed"));
                }
            }
            if let Ok(done) = start_result.try_recv() {
                if quitting {
                    return Ok(());
                }
                status = done.unwrap_or_else(|e| e);
                starting = false;
                schedule.asked = true;
            }
            terminal.draw(|f| match &view {
                // the queue panel holds one line per stem, in order: an empty or failed queue selects nothing
                Some(v) => {
                    let line = (!v.stems.is_empty()).then_some(selected);
                    render(f.area(), f.buffer_mut(), &v.panels, line, &status);
                }
                None => Line::raw("reading the project…").render(f.area(), f.buffer_mut()),
            })?;
            if event::poll(TICK)? {
                if let Event::Key(key) = event::read()? {
                    let len = view.as_ref().map_or(0, |v| v.stems.len());
                    let key = action(&key);
                    let (now, later) = quits(&key, starting, quitting);
                    if now {
                        return Ok(());
                    }
                    if later && !quitting {
                        status = "quitting once the start ends…".to_string();
                    }
                    quitting = later;
                    match key {
                        Action::Quit => {}
                        Action::Refresh => schedule.asked = true,
                        Action::Measure => {
                            measure();
                            schedule.asked = true;
                        }
                        Action::Start => {
                            let stem = view.as_ref().and_then(|v| v.stems.get(selected));
                            if let (Some(stem), false) = (stem.cloned(), starting) {
                                starting = true;
                                status = format!("starting {stem}…");
                                let send = started.clone();
                                std::thread::spawn(move || send.send(start(&stem)));
                            }
                        }
                        a => selected = select(selected, len, &a),
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
        let area = Rect::new(0, 0, 80, 15);
        let mut buf = Buffer::empty(area);
        render(area, &mut buf, &panels, Some(1), "started a in /w");
        let row = |y: u16| (0..80).map(|x| buf[(x, y)].symbol()).collect::<String>();
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
        render(area, &mut buf, &panels, Some(29), "");
        let row = |y: u16| (0..40).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let y = (0..22)
            .find(|y| row(*y).contains("issue30"))
            .expect("the selected issue is shown");
        assert!(buf[(3, y)].modifier.contains(Modifier::REVERSED));
    }

    // spec: STU-15 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn quit_during_a_start_waits_for_it() {
        assert_eq!(quits(&Action::Quit, false, false), (true, false));
        assert_eq!(quits(&Action::Quit, true, false), (false, true));
        assert_eq!(quits(&Action::Refresh, true, true), (false, true));
        assert_eq!(quits(&Action::Down, false, false), (false, false));
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
