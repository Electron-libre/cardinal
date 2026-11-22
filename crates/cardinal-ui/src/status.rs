//! The status line of the screen (SUS-2): what an action did or why it refused, a notification that failed, an
//! ignored theme line, each message in the role of its kind, on rows of their own above the help line, wrapped so a
//! narrow terminal shows them all (SUS-3).

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

/// The messages of the status line, in the order they came, then the failed notification it keeps (STU-30); empty
/// when it shows none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    messages: Vec<(String, Kind)>,
    kept: Option<String>,
}

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

    /// Adds the message `text` of `kind` after those it shows.
    pub fn push(&mut self, text: impl Into<String>, kind: Kind) {
        let text = text.into();
        if !text.is_empty() {
            self.messages.push((text, kind));
        }
    }

    /// Keeps the error `text` after the messages, those set later included: a notification fails once only, so a
    /// task that ends after it must not hide it (STU-30).
    pub fn keep(&mut self, text: impl Into<String>) {
        let text = text.into();
        if !text.is_empty() {
            self.kept = Some(text);
        }
    }

    /// Replaces the messages by those of `next`, the error it keeps left as it is.
    pub fn set(&mut self, next: Status) {
        self.messages = next.messages;
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty() && self.kept.is_none()
    }

    /// The rows the status line takes on a screen over `screen`: as many as its messages need at its width, at most
    /// a third of its height and one at least (SUS-3), none while it shows no message.
    pub fn rows(&self, screen: Rect) -> u16 {
        if self.is_empty() {
            return 0;
        }
        let room = (screen.height / 3).max(1);
        let needed = self.laid(screen.width, room).len();
        u16::try_from(needed).unwrap_or(u16::MAX).min(room)
    }

    /// The messages in the order they came, the kept error last, each with its kind.
    fn messages(&self) -> impl Iterator<Item = (&str, Kind)> {
        let messages = self
            .messages
            .iter()
            .map(|(text, kind)| (text.as_str(), *kind));
        messages.chain(self.kept.iter().map(|text| (text.as_str(), Kind::Error)))
    }

    /// The rows of the messages at `width`, at most `room`: the errors first when the messages need more (SUS-3).
    fn laid(&self, width: u16, room: u16) -> Vec<Vec<(&str, Kind)>> {
        let width = usize::from(width.max(1));
        let rows = wrap(self.messages(), width);
        if rows.len() <= usize::from(room) {
            return rows;
        }
        let errors = self.messages().filter(|(_, kind)| *kind == Kind::Error);
        let others = self.messages().filter(|(_, kind)| *kind != Kind::Error);
        let mut rows = wrap(errors.chain(others), width);
        rows.truncate(usize::from(room));
        rows
    }

    /// Draws the messages over `area` on `surface` across its whole width, each in the role of its kind, a message
    /// that does not fit after the one before it on the row below (SUS-3).
    pub fn draw(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if self.is_empty() {
            return;
        }
        buf.set_style(area, Style::new().bg(theme.color(Role::Surface)));
        for (row, y) in self
            .laid(area.width, area.height)
            .into_iter()
            .zip(area.rows())
        {
            let mut spans = Vec::new();
            for (n, (text, kind)) in row.into_iter().enumerate() {
                if n > 0 {
                    spans.push(Span::raw(GAP));
                }
                spans.push(Span::styled(text, theme.fg(kind.role())));
            }
            Line::from(spans).render(y, buf);
        }
    }
}

/// What parts two messages of one row.
const GAP: &str = "  ";

/// The display columns of `text`.
fn columns(text: &str) -> usize {
    Line::raw(text).width()
}

/// `messages` laid out in rows of `width` columns: each after the one before it on its row when it fits, else on a
/// row of its own, a message wider than a row going on over the rows under it.
fn wrap<'a>(
    messages: impl Iterator<Item = (&'a str, Kind)>,
    width: usize,
) -> Vec<Vec<(&'a str, Kind)>> {
    let mut rows: Vec<Vec<(&str, Kind)>> = Vec::new();
    let mut used = 0;
    for (text, kind) in messages {
        let wide = columns(text);
        if let Some(row) = rows.last_mut().filter(|_| used + GAP.len() + wide <= width) {
            row.push((text, kind));
            used += GAP.len() + wide;
            continue;
        }
        let mut rest = text;
        loop {
            let (head, tail) = split(rest, width);
            rows.push(vec![(head, kind)]);
            used = columns(head);
            if tail.is_empty() {
                break;
            }
            // the row below starts at the next word, not at the blank of the cut
            rest = tail.trim_start();
            if rest.is_empty() {
                break;
            }
        }
    }
    rows
}

/// `text` cut after its first `width` columns between two graphemes, one grapheme at least so a row always takes
/// some.
fn split(text: &str, width: usize) -> (&str, &str) {
    let (mut used, mut at) = (0, 0);
    for grapheme in Span::raw(text).styled_graphemes(Style::new()) {
        used += columns(grapheme.symbol);
        if used > width && at > 0 {
            return text.split_at(at);
        }
        at += grapheme.symbol.len();
    }
    (text, "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    const SCREEN: Rect = Rect::new(0, 0, 60, 15);

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

    // spec: SUS-H2, SUS-2 (internal: the colors of a screen that needs a real terminal)
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

    // spec: STU-30, SUS-2 (internal: the status line of a screen that needs a real terminal)
    #[test]
    fn a_failed_notification_outlives_the_task_that_ends_after_it() {
        let (theme, _) = crate::theme::parse("pass = \"#000008\"\nfail = \"#000009\"\n");
        let mut status = Status::new("merging foo…", Kind::Other);
        status.keep("notify: exit 1: no bus");
        status.set(Status::of(Ok("merged foo (foo) into main".into())));
        let buf = drawn(&status, &theme);
        let row: String = (0..60).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(row.starts_with("merged foo (foo) into main  notify: exit 1: no bus"));
        assert_eq!(buf[(at(&buf, "notify"), 0)].fg, Color::Rgb(0, 0, 9));
        status.set(Status::default());
        assert_eq!(status.rows(SCREEN), 1, "kept until the screen quits");
    }

    // spec: SUS-2 (internal: the status line of a screen that needs a real terminal)
    #[test]
    fn no_message_takes_no_row_and_draws_nothing() {
        let status = Status::new("", Kind::Error);
        assert!(status.is_empty());
        assert_eq!(status.rows(SCREEN), 0);
        assert_eq!(Status::of(Ok("ended foo".into())).rows(SCREEN), 1);
        let buf = drawn(&status, &Theme::default());
        assert_eq!(buf, Buffer::empty(buf.area));
    }

    /// The rows `status` draws on a screen of `width` by `height`, as text.
    fn wrapped(status: &Status, width: u16, height: u16) -> Vec<String> {
        let area = Rect::new(0, 0, width, status.rows(Rect::new(0, 0, width, height)));
        let mut buf = Buffer::empty(area);
        status.draw(area, &mut buf, &Theme::default());
        (0..area.height)
            .map(|y| (0..width).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .map(|row| row.trim_end().to_string())
            .collect()
    }

    // spec: SUS-3 (internal: the status line of a screen that needs a real terminal)
    #[test]
    fn a_message_that_does_not_fit_starts_the_row_below() {
        let (theme, _) = crate::theme::parse("surface = \"#000002\"\nfail = \"#000009\"\n");
        let mut status = Status::default();
        for w in ["a", "b", "c"] {
            status.push(format!("ended worktree-{w} (branch-{w}x)"), Kind::Done);
        }
        status.keep("notify: exit 3: no bus");
        let rows = wrapped(&status, 80, 24);
        assert_eq!(
            rows,
            [
                "ended worktree-a (branch-ax)  ended worktree-b (branch-bx)",
                "ended worktree-c (branch-cx)  notify: exit 3: no bus",
            ]
        );
        let area = Rect::new(0, 0, 80, 2);
        let mut buf = Buffer::empty(area);
        status.draw(area, &mut buf, &theme);
        assert_eq!(buf[(30, 1)].fg, Color::Rgb(0, 0, 9), "notify in fail");
        for y in 0..2 {
            assert_eq!(
                buf[(79, y)].bg,
                Color::Rgb(0, 0, 2),
                "the whole width of each row"
            );
        }
    }

    // spec: SUS-3 (internal: the status line of a screen that needs a real terminal)
    #[test]
    fn a_message_wider_than_the_screen_goes_on_over_the_rows_under_it() {
        let status = Status::new("merged foo into main…done", Kind::Done);
        assert_eq!(
            wrapped(&status, 10, 30),
            ["merged foo", "into main…", "done"]
        );
        assert_eq!(wrapped(&status, 30, 30), ["merged foo into main…done"]);
    }

    // spec: SUS-3 (internal: the status line of a screen that needs a real terminal)
    #[test]
    fn a_line_with_too_few_rows_shows_its_errors_first() {
        let mut status = Status::default();
        for n in 1..=4 {
            status.push(format!("ended worktree-{n}"), Kind::Done);
        }
        status.push("foo diverged from main", Kind::Error);
        status.keep("notify: exit 1: no bus");
        let all = wrapped(&status, 24, 30);
        assert_eq!(all.len(), 6, "six rows on a screen tall enough");
        let rows = wrapped(&status, 24, 9);
        assert_eq!(
            rows,
            [
                "foo diverged from main",
                "notify: exit 1: no bus",
                "ended worktree-1"
            ]
        );
        assert_eq!(status.rows(Rect::new(0, 0, 24, 2)), 1, "one row at least");
    }
}
