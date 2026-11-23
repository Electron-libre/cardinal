//! The screen of `cardinal-ui --print` (STU-4): the panels then the measures, rendered once as plain text.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;

use crate::panels::Panel;
use crate::screen::{height, render};
use crate::status::Status;
use crate::theme::Theme;

/// The width of the text of `--print`, unless a line needs more (STU-4).
const PRINT_WIDTH: u16 = 100;

/// The screen of `panels` and the help line `help` (SUA-15) as text, the measures under the panels, whole, as wide as `PRINT_WIDTH` or its longest line in display columns, each
/// row without its trailing spaces (STU-4). Rendered straight into a buffer: a terminal of the test backend
/// mixes up the cells past the 65,535th.
pub fn text(panels: &[Panel; 3], help: &str) -> String {
    let longest = panels
        .iter()
        .flat_map(|p| p.lines.iter())
        .map(|l| Line::raw(l.as_str()).width().saturating_add(2))
        .chain(std::iter::once(Line::raw(help).width()))
        .max()
        .unwrap_or(0);
    let width = PRINT_WIDTH.max(u16::try_from(longest).unwrap_or(u16::MAX));
    let rows = panels.iter().map(height).fold(1u16, u16::saturating_add);
    let area = Rect::new(0, 0, width, rows);
    let mut buf = Buffer::empty(area);
    // the colors never reach the text (SUS-1)
    render(
        area,
        &mut buf,
        (panels, help),
        None,
        &Status::default(),
        &Theme::default(),
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    // spec: STU-4, SUA-15 (internal: no read gives a help line wider than 100 columns yet)
    #[test]
    fn a_help_line_wider_than_the_panels_is_printed_whole() {
        let panel = |title: &'static str| Panel {
            title: title.into(),
            lines: vec!["x".to_string()],
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
        };
        let help = format!("q quit  {}  g charts", "k key".repeat(30));
        let out = text(
            &[panel("Queue"), panel("Worktrees"), panel("Measures")],
            &help,
        );
        assert_eq!(out.lines().last(), Some(help.as_str()), "{out}");
    }
}
