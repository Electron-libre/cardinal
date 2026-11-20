//! The screen of `cardinal-ui` (STU-1): the panels drawn with Ratatui, in a terminal (STU-2) or once as text
//! (STU-4); in the terminal, an issue of the queue is selected, `s` starts its session (SUA-1), `f` merges it into the
//! trunk (SUA-2); `Tab` moves the selection to the worktrees, where `e` ends what a merged issue left (SUA-3); `E`
//! ends it on every merged worktree (SUA-13); `m` takes the measures again (STU-17); `v` starts the release session
//! (SUA-4); a worktree that starts to wait for the user notifies the desktop (STU-29).

use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::keys::{action, ended, job, quits, task, Action, Ended, Focus, Selection, Task};
use crate::notify::{self, Notifier, Wait};
use crate::panels::Panel;

/// How often the screen reads the project again (STU-2).
pub const REFRESH: Duration = Duration::from_secs(2);

/// How long the loop waits for a key before it looks for a finished read.
const TICK: Duration = Duration::from_millis(100);

/// The width of the text of `--print`, unless a line needs more (STU-4).
const PRINT_WIDTH: u16 = 100;

/// The help line under the panels.
pub(crate) const HELP: &str =
    "q quit  r refresh  ↑↓ select  tab panel  s start  f merge  e end  E end merged  v release  m measure";

/// What a read gives: the panels, the stems of the issues of the queue and the path and branch of the worktrees, in
/// the order of the lines of their panels, and the waits of the worktrees, `None` when they could not be read.
pub struct View {
    pub panels: [Panel; 4],
    pub stems: Vec<String>,
    pub worktrees: Vec<(String, Option<String>)>,
    pub waits: Option<Vec<Wait>>,
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
            if let Some(before) = view.as_ref() {
                selected.reread(before, &read);
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

/// Runs the screen in the terminal until the user quits (STU-2): `read` gives the panels, run on a thread of its
/// own so the keys are answered during a slow read; `run_task` runs the task of `s` or `f` on the selected issue, of
/// `e` on the selected worktree, or of `v` or `E` (SUA-1, SUA-2, SUA-3, SUA-4, SUA-13), on a thread too, one at a time, its result on the status line; `measure` starts a measure on `m`,
/// which runs apart (STU-17); each read notifies the desktop of its new waits (STU-29, STU-30). Gives the result of a task the user quit during, to print once the terminal is given
/// back (SUA-6). The terminal is given back as it was, also on a panic (Ratatui's hook).
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
        let mut notifier = Notifier::new(notify::program());
        let mut busy = false;
        let mut quitting = false;
        loop {
            if schedule.start(Instant::now()) && ask.send(()).is_err() {
                return Err(std::io::Error::other("the reading of the project failed"));
            }
            if received(&read_panels, &mut view, &mut selected, &mut notifier)? {
                schedule.ended(Instant::now());
            }
            if let Some(line) = notifier.failure() {
                status = line;
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
                        k if task(&k, "").is_some() => match job(&k, selected, view.as_ref()) {
                            Ok(Some((job, shown, stem))) if !busy => {
                                busy = true;
                                status = shown;
                                let send = done.clone();
                                std::thread::spawn(move || send.send(run_task(job, &stem)));
                            }
                            Err(why) if !busy => status = why,
                            _ => {}
                        },
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
    use crate::keys::reselect;
    use crate::keys::tests::view;

    // spec: SUA-1 (internal: the highlight of a screen that needs a real terminal)
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

    // spec: SUA-3 (internal: the selection of a screen that needs a real terminal)
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

    // spec: SUA-3 (internal: the layout of a screen that needs a real terminal)
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
