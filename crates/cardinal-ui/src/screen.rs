//! The screen of `cardinal-ui` (STU-1): the panels drawn with Ratatui in a terminal (STU-2), once as text by
//! `print.rs` (STU-4): the queue with its selected issue or milestone highlighted (STU-14), the worktrees, the
//! charts of the measures or their table (SUC-1, SUC-6), the summary of the selected issue under the queue (SUQ-6),
//! the help line of the keys of the selection (SUA-15) and the command palette over the panels (SUP-1); the keys
//! that drive it are read by `terminal.rs`.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::charts::{head, render_charts, Chart};
use crate::keys::{Screen, Selection};
use crate::notify::Wait;
use crate::palette::Palette;
use crate::panels::{Mark, Panel};
use crate::status::Status;
use crate::theme::{Role, Theme};

/// The help line under the table of the measures (SUC-6).
const MEASURES_HELP: &str = "q quit  r refresh  m measure  g panels";

/// What a read gives: the panels, the rows of the queue the selection rests on and the path and branch of the
/// worktrees, in the order of the lines of their panels, and the waits of the worktrees, `None` when they could not be read.
pub struct View {
    pub panels: [Panel; 3],
    pub rows: Vec<crate::keys::Row>,
    /// The line of the queue panel of each row, the other headings and the empty lines between them (STU-14).
    pub row_lines: Vec<usize>,
    pub worktrees: Vec<(String, Option<String>)>,
    pub waits: Option<Vec<Wait>>,
    /// The charts of the last measure (SUC-2).
    pub charts: [Chart; 4],
    /// The keys of the tasks that would act on each issue and worktree, for the help line (SUA-15).
    pub keys: crate::help::Keys,
    /// The summary of each row of the queue, none on a milestone (SUQ-6).
    pub summaries: Vec<Option<crate::summary::Summary>>,
    /// The stem of the issue `next-issue` offers, `None` when it offers none or the queue could not be read (STU-36).
    pub next: Option<String>,
    /// The Markdown document of each row of the queue, or why it has none (SUD-4).
    pub documents: Vec<Result<std::path::PathBuf, String>>,
}

/// The rows a panel takes: its lines and its two borders.
pub(crate) fn height(p: &Panel) -> u16 {
    u16::try_from(p.lines.len())
        .unwrap_or(u16::MAX)
        .saturating_add(2)
}

/// The line `text` with the words of `marks` (in the order of the line) in their role (SUS-1).
pub(crate) fn styled<'a>(
    text: &'a str,
    marks: impl Iterator<Item = &'a Mark>,
    theme: &Theme,
) -> Line<'a> {
    let mut spans = Vec::new();
    let mut at = 0;
    for m in marks {
        let (Some(before), Some(word)) = (text.get(at..m.range.start), text.get(m.range.clone()))
        else {
            continue;
        };
        spans.push(Span::raw(before));
        spans.push(Span::styled(word, theme.fg(m.role)));
        at = m.range.end;
    }
    spans.push(Span::raw(text.get(at..).unwrap_or("")));
    Line::from(spans)
}

/// Renders `panels` one above the other into `buf` over `area` in the roles of `theme` (SUS-1): the first in the
/// rows the others leave, each other at its height, the line `selected` (a panel and a line) highlighted, then the
/// rows of `status` while it shows a message (SUS-2, SUS-3) and the line `help`.
pub(crate) fn render(
    area: Rect,
    buf: &mut Buffer,
    (panels, help): (&[Panel], &str),
    selected: Option<(usize, usize)>,
    status: &Status,
    theme: &Theme,
) {
    buf.set_style(area, theme.screen());
    let areas = layout(area, panels, status);
    for (n, (p, area)) in panels.iter().zip(areas.iter()).enumerate() {
        let line = selected.and_then(|(panel, line)| (panel == n).then_some(line));
        draw_panel(p, *area, buf, line, theme);
        crate::limits::draw(p, *area, buf, theme);
    }
    // the queue is the first panel of the screen: a summary titled `Queue` lower down bears no mark (SUS-6)
    if let (Some(p), Some(area)) = (panels.first(), areas.first()) {
        crate::mark::draw(p, *area, buf, theme);
    }
    let n = panels.len();
    status.draw(areas[n], buf, theme);
    help_line(help, areas[n + 1], buf, theme);
}

/// The areas of `panels` over `area` as [`render`] lays them out, then those of the status line and of the help.
pub(crate) fn layout(area: Rect, panels: &[Panel], status: &Status) -> std::rc::Rc<[Rect]> {
    let first = panels.first().map_or(0, |p| height(p).min(3));
    let constraints = std::iter::once(Constraint::Min(first))
        .chain(panels.iter().skip(1).map(|p| Constraint::Max(height(p))))
        .chain([Constraint::Length(status.rows(area)), Constraint::Length(1)]);
    Layout::vertical(constraints).split(area)
}

/// Draws `p` bordered over `area`, its line `selected` highlighted across the inner width and kept visible, in the
/// roles of `theme` (SUS-1).
pub(crate) fn draw_panel(
    p: &Panel,
    area: Rect,
    buf: &mut Buffer,
    selected: Option<usize>,
    theme: &Theme,
) {
    let scroll = selected.map_or(0, |line| offset(line, area.height.saturating_sub(2)));
    let lines: Vec<Line> = p
        .lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let line = styled(l, p.marks.iter().filter(|m| m.line == i), theme);
            if p.headings.contains(&i) {
                // the words of a heading in their role keep its bold (SUQ-5)
                line.style(theme.fg(Role::Accent).add_modifier(Modifier::BOLD))
            } else {
                line
            }
        })
        .collect();
    let block = Block::bordered()
        .border_style(theme.fg(Role::Border))
        .title(format!(" {} ", p.title))
        .title_style(theme.fg(Role::Accent));
    let inner = block.inner(area);
    Paragraph::new(lines)
        .block(block)
        .scroll((scroll, 0))
        .render(area, buf);
    // the whole inner row, not only its characters: the paragraph styles the graphemes alone
    let row = selected
        .and_then(|line| u16::try_from(line).ok())
        .and_then(|line| line.checked_sub(scroll))
        .filter(|row| *row < inner.height);
    if let Some(row) = row {
        buf.set_style(
            Rect::new(inner.x, inner.y + row, inner.width, 1),
            theme.selected(),
        );
    }
}

/// The help `help` in `muted` on `surface` over `area` (SUS-1).
pub(crate) fn help_line(help: &str, area: Rect, buf: &mut Buffer, theme: &Theme) {
    buf.set_style(area, theme.help());
    Line::raw(help).render(area, buf);
}

/// The first line of the queue shown in `rows` rows, so the line `selected` is visible (STU-14).
fn offset(selected: usize, rows: u16) -> u16 {
    let first = selected
        .saturating_add(1)
        .saturating_sub(usize::from(rows.max(1)));
    u16::try_from(first).unwrap_or(u16::MAX)
}

/// Draws the screen `shown` of `view` in the frame `f` (SUC-1): its charts, the table of its measures (SUC-6), or
/// its queue and worktrees with the line `selected` of [`Selection::line`] highlighted (STU-1) over the help line of
/// the selection, no key of a task while one runs (`busy`, SUA-15), or `reading the
/// project…` before the first read, in the roles of `theme` (SUS-1); the summary of the selected issue under the
/// queue when it is `unfolded` (SUQ-6), and the command `palette` over them when it is open (SUP-1).
pub(crate) fn draw(
    f: &mut ratatui::Frame,
    view: Option<&View>,
    (selected, shown, busy, unfolded, palette): (Selection, Screen, bool, bool, Option<&Palette>),
    status: &Status,
    theme: &Theme,
) {
    let area = f.area();
    if let Some(v) = view {
        let buf = f.buffer_mut();
        match shown {
            Screen::Charts => {
                render_charts(area, buf, &head(&v.panels[2]), &v.charts, status, theme);
            }
            Screen::Measures => {
                render(
                    area,
                    buf,
                    (&v.panels[2..], MEASURES_HELP),
                    None,
                    status,
                    theme,
                );
            }
            // the selected line comes from `Selection::line`: a stem through its line of the queue, a worktree by
            // its rank
            Screen::Panels => render(
                area,
                buf,
                (
                    &crate::summary::panels(v, selected, unfolded, (area, status)),
                    &crate::help::line(v, selected, busy),
                ),
                selected.line(v),
                status,
                theme,
            ),
        }
        if let Some(p) = palette.filter(|_| shown == Screen::Panels) {
            let all = crate::palette::commands(view, selected, busy);
            crate::palette::draw(p, &all, area, f.buffer_mut(), theme);
        }
    } else {
        f.buffer_mut().set_style(area, theme.screen());
        Line::raw("reading the project…").render(area, f.buffer_mut());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Kind;

    /// A help line under the panels (SUA-15).
    const HELP: &str = ": commands  ↑↓ select  tab panel  q quit";
    use crate::keys::tests::view;
    use crate::keys::{reselect, Focus};
    use ratatui::style::{Color, Modifier};

    // spec: SUA-1, SUS-2 (internal: the highlight of a screen that needs a real terminal)
    #[test]
    fn the_selected_issue_is_highlighted_and_the_status_shown() {
        let panel = |title: &'static str, lines: &[&str]| Panel {
            title: title.into(),
            lines: lines.iter().map(|l| (*l).to_string()).collect(),
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
            corner: None,
        };
        let panels = [
            panel("Queue", &["> b", "  a"]),
            panel("Worktrees", &["w"]),
            panel("Measures", &["x"]),
        ];
        let area = Rect::new(0, 0, 120, 15);
        let mut buf = Buffer::empty(area);
        render(
            area,
            &mut buf,
            (&panels, HELP),
            Some((0, 1)),
            &Status::new("started a in /w", Kind::Done),
            &Theme::default(),
        );
        let row = |y: u16| (0..120).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let a = (0..15).find(|y| row(*y).contains("  a")).unwrap();
        let b = (0..15).find(|y| row(*y).contains("> b")).unwrap();
        assert!(buf[(3, a)].modifier.contains(Modifier::REVERSED));
        assert!(!buf[(3, b)].modifier.contains(Modifier::REVERSED));
        let help = (0..15).find(|y| row(*y).starts_with(": commands")).unwrap();
        assert!(
            row(help - 1).starts_with("started a in /w "),
            "on its own row above the help"
        );
    }

    // spec: SUS-3 (internal: the screen needs a real terminal)
    #[test]
    fn on_a_narrow_screen_the_status_line_wraps_above_the_help() {
        let panel = |title: &'static str| Panel {
            title: title.into(),
            lines: vec!["x".to_string()],
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
            corner: None,
        };
        let panels = [panel("Queue"), panel("Worktrees"), panel("Measures")];
        let mut status = Status::default();
        for w in ["a", "b", "c"] {
            status.push(format!("ended worktree-{w} (branch-{w}x)"), Kind::Done);
        }
        status.keep("notify: exit 3: no bus");
        let area = Rect::new(0, 0, 80, 15);
        let mut buf = Buffer::empty(area);
        render(
            area,
            &mut buf,
            (&panels, HELP),
            None,
            &status,
            &Theme::default(),
        );
        let row = |y: u16| (0..80).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let help = (0..15).find(|y| row(*y).starts_with(": commands")).unwrap();
        assert!(row(help - 2).starts_with("ended worktree-a"));
        assert!(
            row(help - 1).contains("notify: exit 3: no bus"),
            "{}",
            row(help - 1)
        );
    }

    // spec: SUS-2 (internal: the layout of a screen that needs a real terminal)
    #[test]
    fn with_no_message_the_panels_keep_the_row_of_the_status_line() {
        let panel = |title: &'static str, lines: &[&str]| Panel {
            title: title.into(),
            lines: lines.iter().map(|l| (*l).to_string()).collect(),
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
            corner: None,
        };
        let panels = [
            panel("Queue", &["> b", "  a"]),
            panel("Worktrees", &["w"]),
            panel("Measures", &["x"]),
        ];
        let area = Rect::new(0, 0, 120, 15);
        let mut buf = Buffer::empty(area);
        render(
            area,
            &mut buf,
            (&panels, HELP),
            None,
            &Status::default(),
            &Theme::default(),
        );
        let row = |y: u16| (0..120).map(|x| buf[(x, y)].symbol()).collect::<String>();
        assert!(row(14).starts_with(": commands"));
        assert!(
            row(13).starts_with('└'),
            "the last panel right above the help"
        );
    }

    // spec: SUS-H1, SUS-1, SUS-2 (internal: the colors of a screen that needs a real terminal)
    #[test]
    fn the_frame_is_drawn_in_the_roles_of_the_theme() {
        let panel = |title: &'static str, lines: &[&str], marks| Panel {
            title: title.into(),
            lines: lines.iter().map(|l| (*l).to_string()).collect(),
            marks,
            headings: Vec::new(),
            failed: false,
            corner: None,
        };
        let ready = crate::panels::Mark {
            line: 0,
            range: 5..10,
            role: Role::Pass,
        };
        let panels = [
            panel("Queue", &["> b  ready", "  a"], vec![ready]),
            panel("Worktrees", &["w"], Vec::new()),
            panel("Measures", &["x"], Vec::new()),
        ];
        let (theme, ignored) = crate::theme::parse(
            "background = \"#000001\"\nsurface = \"#000002\"\nborder = \"#112233\"\ntext = \"#000004\"\nmuted = \"#000005\"\naccent = \"#000006\"\nselection = \"#000007\"\npass = \"#000008\"\n",
        );
        assert_eq!(ignored, Vec::<String>::new());
        let rgb = |n| Color::Rgb(0, 0, n);
        let area = Rect::new(0, 0, 140, 15);
        let mut buf = Buffer::empty(area);
        render(
            area,
            &mut buf,
            (&panels, HELP),
            Some((0, 1)),
            &Status::new("theme: line 9 ignored", Kind::Error),
            &theme,
        );
        let row = |y: u16| (0..140).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let find = |s: &str| (0..15).find(|y| row(*y).contains(s)).unwrap();
        let corner = &buf[(0, 0)];
        assert_eq!(
            (corner.symbol(), corner.fg),
            ("┌", Color::Rgb(0x11, 0x22, 0x33))
        );
        let title = row(0).find("Queue").unwrap();
        assert_eq!(buf[(u16::try_from(title).unwrap(), 0)].fg, rgb(6));
        let m = find("│w");
        assert_eq!((buf[(1, m)].fg, buf[(1, m)].bg), (rgb(4), rgb(1)));
        let b = find("> b  ready");
        let word = u16::try_from(row(b).chars().position(|c| c == 'r').unwrap()).unwrap();
        assert_eq!(buf[(word, b)].fg, rgb(8), "ready in pass");
        let a = find("  a");
        assert_eq!(buf[(3, a)].bg, rgb(7));
        let border =
            u16::try_from(row(a).chars().skip(1).position(|c| c == '│').unwrap() + 1).unwrap();
        assert_eq!(buf[(border - 1, a)].bg, rgb(7), "up to the right border");
        assert_eq!(buf[(border, a)].bg, rgb(1), "the border keeps the screen");
        assert!(!buf[(3, a)].modifier.contains(Modifier::REVERSED));
        let help = find("q quit");
        assert_eq!((buf[(0, help)].fg, buf[(0, help)].bg), (rgb(5), rgb(2)));
        assert_eq!(buf[(139, help)].bg, rgb(2), "the whole line");
        let status = find("theme: line 9 ignored");
        assert_eq!(status + 1, help, "the status line above the help");
        assert_eq!(
            (buf[(0, status)].fg, buf[(0, status)].bg),
            (Color::Red, rgb(2)),
            "an error in fail on surface"
        );
        assert_eq!(buf[(139, status)].bg, rgb(2), "the whole width");
    }

    // spec: SUS-1, SUQ-5 (internal: the colors of a screen that needs a real terminal)
    #[test]
    fn the_marked_words_are_drawn_in_their_role_also_when_selected() {
        let panel = |title: &'static str, lines: &[&str], marks| Panel {
            title: title.into(),
            lines: lines.iter().map(|l| (*l).to_string()).collect(),
            marks,
            headings: if title == "Queue" {
                vec![0]
            } else {
                Vec::new()
            },
            failed: false,
            corner: None,
        };
        let ready = |line| crate::panels::Mark {
            line,
            range: 5..10,
            role: Role::Pass,
        };
        let on_hold = crate::panels::Mark {
            line: 0,
            range: 8..17,
            role: Role::Warn,
        };
        let panels = [
            panel(
                "Queue",
                &["■ One (on hold)", "  a  ready", "  b  ready"],
                vec![on_hold, ready(1), ready(2)],
            ),
            panel("Worktrees", &["w"], Vec::new()),
            panel("Measures", &["x"], Vec::new()),
        ];
        let (theme, _) = crate::theme::parse("pass = \"#00aa00\"\nselection = \"#333333\"");
        let area = Rect::new(0, 0, 140, 15);
        let mut buf = Buffer::empty(area);
        render(
            area,
            &mut buf,
            (&panels, HELP),
            Some((0, 2)),
            &Status::default(),
            &theme,
        );
        let row = |y: u16| (0..140).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let a = (0..15).find(|y| row(*y).contains("  a  ready")).unwrap();
        let b = (0..15).find(|y| row(*y).contains("  b  ready")).unwrap();
        let green = Color::Rgb(0, 0xaa, 0);
        assert_eq!(buf[(6, a)].fg, green, "{}", row(a));
        assert_eq!(buf[(3, a)].fg, Color::Reset, "the stem in text");
        assert_eq!(
            (buf[(6, b)].fg, buf[(6, b)].bg),
            (green, Color::Rgb(0x33, 0x33, 0x33))
        );
        // the heading in accent and bold, its `(on hold)` in warn and bold
        let one = (0..15).find(|y| row(*y).contains("■ One")).unwrap();
        let (name, hold) = (&buf[(3, one)], &buf[(7, one)]);
        assert_eq!(
            (name.fg, hold.fg),
            (Color::Cyan, Color::Yellow),
            "{}",
            row(one)
        );
        assert!(name.modifier.contains(Modifier::BOLD) && hold.modifier.contains(Modifier::BOLD));
        assert!(
            !buf[(3, a)].modifier.contains(Modifier::BOLD),
            "an issue row is not bold"
        );
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
        let panel = |title: &'static str, lines: Vec<String>| Panel {
            title: title.into(),
            lines,
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
            corner: None,
        };
        let queue = (1..=40).map(|i| format!("  issue{i:02}")).collect();
        let panels = [
            panel("Queue", queue),
            panel("Worktrees", vec!["w".to_string()]),
            panel("Measures", vec!["x".to_string()]),
        ];
        let area = Rect::new(0, 0, 40, 22);
        let mut buf = Buffer::empty(area);
        render(
            area,
            &mut buf,
            (&panels, HELP),
            Some((0, 29)),
            &Status::default(),
            &Theme::default(),
        );
        let row = |y: u16| (0..40).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let y = (0..22)
            .find(|y| row(*y).contains("issue30"))
            .expect("the selected issue is shown");
        assert!(buf[(3, y)].modifier.contains(Modifier::REVERSED));
    }

    // spec: SUE-1 (internal: the selection of a screen that needs a real terminal)
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
            row: 0,
            worktree: 1,
        };
        sel.reread(&before, &after);
        assert_eq!(sel.worktree, 2);
        let panel = |title: &'static str, lines: &[&str]| Panel {
            title: title.into(),
            lines: lines.iter().map(|l| (*l).to_string()).collect(),
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
            corner: None,
        };
        let panels = [
            panel("Queue", &["> b"]),
            panel("Worktrees", &["/main", "/ws"]),
            panel("Measures", &["x"]),
        ];
        let area = Rect::new(0, 0, 80, 15);
        let mut buf = Buffer::empty(area);
        render(
            area,
            &mut buf,
            (&panels, HELP),
            Some((1, 1)),
            &Status::default(),
            &Theme::default(),
        );
        let row = |y: u16| (0..80).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let ws = (0..15).find(|y| row(*y).contains("/ws")).unwrap();
        let b = (0..15).find(|y| row(*y).contains("> b")).unwrap();
        assert!(buf[(3, ws)].modifier.contains(Modifier::REVERSED));
        assert!(!buf[(3, b)].modifier.contains(Modifier::REVERSED));
    }

    // spec: SUE-1 (internal: the layout of a screen that needs a real terminal)
    #[test]
    fn the_selected_worktree_stays_visible_in_a_short_panel() {
        let panel = |title: &'static str, lines: Vec<String>| Panel {
            title: title.into(),
            lines,
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
            corner: None,
        };
        let worktrees = (1..=20).map(|i| format!("/ws{i:02}")).collect();
        let panels = [
            panel("Queue", vec!["> b".to_string()]),
            panel("Worktrees", worktrees),
            panel("Measures", vec!["x".to_string()]),
        ];
        let area = Rect::new(0, 0, 40, 16);
        let mut buf = Buffer::empty(area);
        render(
            area,
            &mut buf,
            (&panels, HELP),
            Some((1, 17)),
            &Status::default(),
            &Theme::default(),
        );
        let row = |y: u16| (0..40).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let y = (0..16)
            .find(|y| row(*y).contains("/ws18"))
            .expect("the selected worktree is shown");
        assert!(buf[(3, y)].modifier.contains(Modifier::REVERSED));
        assert!(
            buf[(38, y)].modifier.contains(Modifier::REVERSED),
            "the whole inner row of the scrolled panel"
        );
        assert!(!buf[(39, y)].modifier.contains(Modifier::REVERSED));
    }

    /// The screen `shown` of a view whose panels hold `lines` (the measures third), drawn in 120 by 15 cells.
    fn drawn(lines: [&[&str]; 3], shown: Screen) -> Vec<String> {
        let mut v = view(&["b"], &[("/main", Some("trunk"))]);
        for (p, l) in v.panels.iter_mut().zip(lines) {
            p.lines = l.iter().map(|x| (*x).to_string()).collect();
        }
        let selected = Selection {
            focus: Focus::Queue,
            row: 0,
            worktree: 0,
        };
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 15)).unwrap();
        terminal
            .draw(|f| {
                draw(
                    f,
                    Some(&v),
                    (selected, shown, false, false, None),
                    &Status::default(),
                    &Theme::default(),
                );
            })
            .unwrap();
        let buf = terminal.backend().buffer();
        (0..15)
            .map(|y| (0..120).map(|x| buf[(x, y)].symbol()).collect())
            .collect()
    }

    // spec: STU-1, SUC-6 (internal: the screens are drawn only in the terminal)
    #[test]
    fn the_panels_show_no_measures_and_the_measures_screen_shows_them_whole() {
        let table = [
            "measured 19:23",
            "by revision:",
            "r1  12 sessions",
            "by milestone:",
        ];
        let panels = drawn([&["> b"], &["/main"], &table], Screen::Panels).join("\n");
        assert!(
            panels.contains(" Queue ") && panels.contains(" Worktrees "),
            "{panels}"
        );
        assert!(
            !panels.contains("Measures") && !panels.contains("measured"),
            "{panels}"
        );
        let rows = drawn([&["> b"], &["/main"], &table], Screen::Measures);
        let screen = rows.join("\n");
        assert!(rows[0].contains(" Measures "), "{screen}");
        for line in table {
            assert!(screen.contains(line), "{line} in {screen}");
        }
        assert!(!screen.contains("Queue"), "{screen}");
        assert!(rows[14].starts_with(MEASURES_HELP), "{screen}");
        assert!(MEASURES_HELP.contains("m measure") && MEASURES_HELP.contains("g panels"));
        let tall: Vec<String> = (0..30).map(|i| format!("row{i:02}")).collect();
        let tall: Vec<&str> = tall.iter().map(String::as_str).collect();
        let screen = drawn([&["> b"], &["/main"], &tall], Screen::Measures).join("\n");
        assert!(
            screen.contains("row00") && !screen.contains("row29"),
            "its first rows: {screen}"
        );
    }
}
