//! The mark of Cardinal on the top border of the queue panel, in `accent`, taking no row of its own (SUS-6).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::panels::{Panel, QUEUE};
use crate::theme::{Role, Theme};

/// The mark, spaces around it as around the title of a panel.
const MARK: &str = " ✦ Cardinal ";

/// Draws the mark against the right corner of the top border of `p`, the first panel of the screen, over `area` when
/// `p` is the queue panel and its border holds its title, the mark and one cell of border between them (SUS-6).
pub(crate) fn draw(p: &Panel, area: Rect, buf: &mut Buffer, theme: &Theme) {
    if p.title != QUEUE {
        return;
    }
    crate::limits::corner(MARK, p, area, buf, theme.fg(Role::Accent));
}

#[cfg(test)]
mod tests {
    use crate::panels::Panel;
    use crate::screen::render;
    use crate::status::Status;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    fn panel(title: &'static str, lines: &[&str]) -> Panel {
        Panel {
            title: title.into(),
            lines: lines.iter().map(|l| (*l).to_string()).collect(),
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
            corner: None,
        }
    }

    /// The rows of `panels` rendered over `width`×`height` in the theme of `accent = "#000006"`, and the buffer.
    fn drawn(panels: &[Panel], width: u16, height: u16) -> (Vec<String>, Buffer) {
        let (theme, _) = crate::theme::parse("accent = \"#000006\"\n");
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        render(
            area,
            &mut buf,
            (panels, "q quit"),
            None,
            &Status::default(),
            &theme,
        );
        let rows = (0..height)
            .map(|y| (0..width).map(|x| buf[(x, y)].symbol()).collect())
            .collect();
        (rows, buf)
    }

    // spec: SUS-H3, SUS-6 (internal: the colors of a screen that needs a real terminal)
    #[test]
    fn the_mark_ends_the_top_border_of_the_queue_in_accent_and_takes_no_row() {
        let queue: Vec<String> = (0..10).map(|i| format!("  issue{i}")).collect();
        let queue: Vec<&str> = queue.iter().map(String::as_str).collect();
        let panels = [
            panel("Queue", &queue),
            panel("Worktrees", &["w"]),
            panel("Measures", &["x"]),
        ];
        let (rows, buf) = drawn(&panels, 80, 24);
        assert!(rows[0].starts_with("┌ Queue ─"), "{rows:?}");
        assert!(rows[0].ends_with("─ ✦ Cardinal ┐"), "{rows:?}");
        let star = u16::try_from(rows[0].chars().position(|c| c == '✦').unwrap()).unwrap();
        assert_eq!(buf[(star, 0)].fg, Color::Rgb(0, 0, 6), "in accent");
        assert_eq!(buf[(star + 2, 0)].fg, Color::Rgb(0, 0, 6), "in accent");
        for (i, row) in rows[1..=10].iter().enumerate() {
            assert!(
                row.contains(&format!("issue{i}")),
                "every row of the queue kept: {rows:?}"
            );
        }
        assert_eq!(rows.iter().filter(|r| r.contains("Cardinal")).count(), 1);
        assert!(!rows
            .iter()
            .any(|r| r.contains("┌ Worktrees") && r.contains("Cardinal")));
    }

    // spec: SUS-6 (internal: a summary panel the screen unfolds)
    #[test]
    fn a_summary_titled_queue_under_the_queue_bears_no_mark() {
        let panels = [
            panel("Queue", &["> a"]),
            panel("Queue", &["goal"]),
            panel("Worktrees", &["w"]),
        ];
        let (rows, _) = drawn(&panels, 80, 12);
        assert_eq!(
            rows.iter().filter(|r| r.contains("Cardinal")).count(),
            1,
            "{rows:?}"
        );
        assert!(rows[0].contains("Cardinal"), "{rows:?}");
    }

    // spec: SUS-6 (internal: the width of a panel the screen sets)
    #[test]
    fn a_queue_panel_too_narrow_for_the_mark_shows_its_title_alone() {
        let panels = [panel("Queue", &["a"]), panel("Worktrees", &["w"])];
        let (rows, _) = drawn(&panels, 20, 8);
        assert!(rows[0].starts_with("┌ Queue ─"), "{rows:?}");
        assert!(
            !rows[0].contains("Cardinal") && !rows[0].contains('✦'),
            "{rows:?}"
        );
        let (rows, _) = drawn(&panels, 22, 8);
        assert_eq!(
            rows[0], "┌ Queue ─ ✦ Cardinal ┐",
            "the narrowest that holds it"
        );
        let (rows, _) = drawn(&panels, 21, 8);
        assert!(!rows[0].contains("Cardinal"), "{rows:?}");
    }
}
