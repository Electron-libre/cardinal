//! The screen of `cardinal-ui` (STU-1): the panels drawn with Ratatui, in a terminal (STU-2) or once as text
//! (STU-4).

use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
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
const HELP: &str = "q quit  r refresh";

/// The rows a panel takes: its lines and its two borders.
fn height(p: &Panel) -> u16 {
    u16::try_from(p.lines.len())
        .unwrap_or(u16::MAX)
        .saturating_add(2)
}

/// Renders the panels into `buf` over `area`: the milestones and the worktrees at their height, the queue in the
/// rest, the help line last.
fn render(area: Rect, buf: &mut Buffer, panels: &[Panel; 3]) {
    let [milestones, queue, worktrees] = panels;
    let areas = Layout::vertical([
        Constraint::Max(height(milestones)),
        Constraint::Min(height(queue).min(3)),
        Constraint::Max(height(worktrees)),
        Constraint::Length(1),
    ])
    .split(area);
    for (p, area) in panels.iter().zip(areas.iter()) {
        let lines: Vec<Line> = p.lines.iter().map(|l| Line::raw(l.as_str())).collect();
        let block = Block::bordered().title(format!(" {} ", p.title));
        Paragraph::new(lines).block(block).render(*area, buf);
    }
    Line::raw(HELP).render(areas[3], buf);
}

/// The screen of `panels` as text, whole, as wide as `PRINT_WIDTH` or its longest line in display columns, each
/// row without its trailing spaces (STU-4). Rendered straight into a buffer: a terminal of the test backend
/// mixes up the cells past the 65,535th.
pub fn text(panels: &[Panel; 3]) -> String {
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
    render(area, &mut buf, panels);
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

/// Runs the screen in the terminal until the user quits (STU-2): `read` gives the panels, run on a thread of its
/// own so the keys are answered during a slow read. The terminal is given back as it was, also on a panic
/// (Ratatui's hook).
pub fn run(read: fn() -> [Panel; 3]) -> std::io::Result<()> {
    let (ask, asked) = mpsc::channel::<()>();
    let (send, read_panels) = mpsc::channel();
    std::thread::spawn(move || {
        while asked.recv().is_ok() {
            if send.send(read()).is_err() {
                break;
            }
        }
    });
    let mut terminal = ratatui::try_init()?;
    let result = (|| {
        let mut panels: Option<[Panel; 3]> = None;
        let mut schedule = Schedule::default();
        loop {
            if schedule.start(Instant::now()) && ask.send(()).is_err() {
                return Err(std::io::Error::other("the reading of the project failed"));
            }
            match read_panels.try_recv() {
                Ok(read) => {
                    panels = Some(read);
                    schedule.ended(Instant::now());
                }
                Err(mpsc::TryRecvError::Empty) => {}
                // the thread of the reads died (a panic): no read would ever come again
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(std::io::Error::other("the reading of the project failed"));
                }
            }
            terminal.draw(|f| match &panels {
                Some(p) => render(f.area(), f.buffer_mut(), p),
                None => Line::raw("reading the project…").render(f.area(), f.buffer_mut()),
            })?;
            if event::poll(TICK)? {
                if let Event::Key(key) = event::read()? {
                    match action(&key) {
                        Action::Quit => return Ok(()),
                        Action::Refresh => schedule.asked = true,
                        Action::Nothing => {}
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

    // spec: STU-2 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn q_esc_and_ctrl_c_quit_and_r_refreshes() {
        assert_eq!(action(&press(KeyCode::Char('q'))), Action::Quit);
        assert_eq!(action(&press(KeyCode::Esc)), Action::Quit);
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(action(&ctrl_c), Action::Quit);
        assert_eq!(action(&press(KeyCode::Char('c'))), Action::Nothing);
        assert_eq!(action(&press(KeyCode::Char('r'))), Action::Refresh);
        assert_eq!(action(&press(KeyCode::Char('x'))), Action::Nothing);
        let mut release = press(KeyCode::Char('q'));
        release.kind = KeyEventKind::Release;
        assert_eq!(action(&release), Action::Nothing);
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
