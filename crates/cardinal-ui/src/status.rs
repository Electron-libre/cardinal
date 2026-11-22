//! The status line of the screen (STU-34): what an action did or why it refused, a notification that failed, an
//! ignored theme line, each message in the role of its kind, on a row of its own above the help line.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::theme::{Role, Theme};

/// What a message tells: an error in `fail`, the result of a task that succeeded in `pass`, the others in `text`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Error,
    Done,
    Other,
}

impl Kind {
    fn role(self) -> Role {
        match self {
            Kind::Error => Role::Fail,
            Kind::Done => Role::Pass,
            Kind::Other => Role::Text,
        }
    }
}

/// The messages of the status line, in the order they came; empty when it shows none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status(Vec<(String, Kind)>);

impl Status {
    /// The status line of the single message `text` of `kind`, empty when `text` is.
    pub fn new(text: impl Into<String>, kind: Kind) -> Self {
        let mut status = Status::default();
        status.push(text, kind);
        status
    }

    /// The status line of the end of a task: its line, in `pass` when it succeeded, in `fail` when it did not.
    pub fn of(result: Result<String, String>) -> Self {
        match result {
            Ok(line) => Status::new(line, Kind::Done),
            Err(line) => Status::new(line, Kind::Error),
        }
    }

    /// Adds the message `text` of `kind` after those it shows, beside them rather than over them (STU-30).
    pub fn push(&mut self, text: impl Into<String>, kind: Kind) {
        let text = text.into();
        if !text.is_empty() {
            self.0.push((text, kind));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The rows the status line takes: one while it shows a message, none otherwise.
    pub fn rows(&self) -> u16 {
        u16::from(!self.is_empty())
    }

    /// Draws the messages over `area` on `surface` across its whole width, each in the role of its kind.
    pub fn draw(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if self.is_empty() {
            return;
        }
        let mut spans = Vec::new();
        for (n, (text, kind)) in self.0.iter().enumerate() {
            if n > 0 {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(text.as_str(), theme.fg(kind.role())));
        }
        buf.set_style(area, Style::new().bg(theme.color(Role::Surface)));
        Line::from(spans).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    fn drawn(status: &Status, theme: &Theme) -> Buffer {
        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        status.draw(area, &mut buf, theme);
        buf
    }

    fn at(buf: &Buffer, word: &str) -> u16 {
        let row: String = (0..buf.area.width).map(|x| buf[(x, 0)].symbol()).collect();
        u16::try_from(row[..row.find(word).unwrap()].chars().count()).unwrap()
    }

    // spec: STU-34 (internal: the colors of a screen that needs a real terminal)
    #[test]
    fn an_error_is_drawn_in_fail_and_a_success_in_pass_on_surface() {
        let (theme, _) = crate::theme::parse(
            "surface = \"#000002\"\ntext = \"#000004\"\npass = \"#000008\"\nfail = \"#000009\"\n",
        );
        let rgb = |n| Color::Rgb(0, 0, n);
        let mut status = Status::of(Ok("merged foo (foo) into main".into()));
        status.push("notify: exit 3: no bus", Kind::Error);
        let buf = drawn(&status, &theme);
        let merged = at(&buf, "merged");
        let notify = at(&buf, "notify");
        assert_eq!((buf[(merged, 0)].fg, buf[(merged, 0)].bg), (rgb(8), rgb(2)));
        assert_eq!((buf[(notify, 0)].fg, buf[(notify, 0)].bg), (rgb(9), rgb(2)));
        assert_eq!(buf[(59, 0)].bg, rgb(2), "the whole width");
        let buf = drawn(&Status::of(Err("foo diverged from main".into())), &theme);
        assert_eq!(buf[(0, 0)].fg, rgb(9));
        let buf = drawn(&Status::new("merging foo…", Kind::Other), &theme);
        assert_eq!(buf[(0, 0)].fg, rgb(4));
    }

    // spec: STU-34 (internal: the status line of a screen that needs a real terminal)
    #[test]
    fn no_message_takes_no_row_and_draws_nothing() {
        let status = Status::new("", Kind::Error);
        assert!(status.is_empty());
        assert_eq!(status.rows(), 0);
        assert_eq!(Status::of(Ok("ended foo".into())).rows(), 1);
        let buf = drawn(&status, &Theme::default());
        assert_eq!(buf, Buffer::empty(buf.area));
    }
}
