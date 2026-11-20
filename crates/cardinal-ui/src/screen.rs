//! The screen of `cardinal-ui` (STU-1): the panels drawn with Ratatui, in a terminal (STU-2) or once as text
//! (STU-4).

use std::time::{Duration, Instant};

use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::{Frame, Terminal};

use crate::panels::Panel;

/// How often the screen reads the project again (STU-2).
pub const REFRESH: Duration = Duration::from_secs(2);

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

/// Draws the panels: the milestones and the worktrees at their height, the queue in the rest, the help line last.
fn draw(frame: &mut Frame, panels: &[Panel; 3]) {
    let [milestones, queue, worktrees] = panels;
    let areas = Layout::vertical([
        Constraint::Max(height(milestones)),
        Constraint::Min(height(queue).min(3)),
        Constraint::Max(height(worktrees)),
        Constraint::Length(1),
    ])
    .split(frame.area());
    for (p, area) in panels.iter().zip(areas.iter()) {
        let lines: Vec<Line> = p.lines.iter().map(|l| Line::raw(l.as_str())).collect();
        let block = Block::bordered().title(format!(" {} ", p.title));
        frame.render_widget(Paragraph::new(lines).block(block), *area);
    }
    frame.render_widget(Line::raw(HELP), areas[3]);
}

/// The screen of `panels` as text, as wide as `PRINT_WIDTH` or its longest line, each row without its trailing
/// spaces (STU-4).
pub fn text(panels: &[Panel; 3]) -> String {
    let longest = panels
        .iter()
        .flat_map(|p| p.lines.iter())
        .map(|l| l.chars().count() + 2)
        .max()
        .unwrap_or(0);
    let width = PRINT_WIDTH.max(u16::try_from(longest).unwrap_or(u16::MAX));
    let rows = panels.iter().map(height).sum::<u16>() + 1;
    let mut terminal =
        Terminal::new(TestBackend::new(width, rows)).expect("a test backend never fails");
    terminal
        .draw(|f| draw(f, panels))
        .expect("a test backend never fails");
    let buffer = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..rows {
        let row: String = (0..width)
            .filter_map(|x| buffer.cell((x, y)).map(ratatui::buffer::Cell::symbol))
            .collect();
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
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('r') => Action::Refresh,
        _ => Action::Nothing,
    }
}

/// Runs the screen in the terminal until the user quits: `read` gives the panels, called again every `REFRESH`
/// and on `r` (STU-2). The terminal is given back as it was, also on a panic (Ratatui's hook).
pub fn run(read: impl Fn() -> [Panel; 3]) -> std::io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = (|| {
        let mut panels = read();
        let mut last = Instant::now();
        loop {
            terminal.draw(|f| draw(f, &panels))?;
            let wait = REFRESH.saturating_sub(last.elapsed());
            let mut refresh = !event::poll(wait)?;
            if !refresh {
                if let Event::Key(key) = event::read()? {
                    match action(&key) {
                        Action::Quit => return Ok(()),
                        Action::Refresh => refresh = true,
                        Action::Nothing => {}
                    }
                }
            }
            // any other event (a key, a resize) leaves the interval running: it never delays a read
            if refresh || last.elapsed() >= REFRESH {
                panels = read();
                last = Instant::now();
            }
        }
    })();
    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    // spec: STU-2 (internal: the keys and the interval of a screen that needs a real terminal)
    #[test]
    fn q_and_esc_quit_r_refreshes_and_the_interval_is_two_seconds() {
        assert_eq!(action(&press(KeyCode::Char('q'))), Action::Quit);
        assert_eq!(action(&press(KeyCode::Esc)), Action::Quit);
        assert_eq!(action(&press(KeyCode::Char('r'))), Action::Refresh);
        assert_eq!(action(&press(KeyCode::Char('x'))), Action::Nothing);
        let mut release = press(KeyCode::Char('q'));
        release.kind = KeyEventKind::Release;
        assert_eq!(action(&release), Action::Nothing);
        assert_eq!(REFRESH, Duration::from_secs(2));
    }
}
