//! The command palette (SUP): `:` opens over the panels the list of the commands that would act on what is selected,
//! the help line's (`help::offered`) then those that always act, filtered by the text typed; `Enter` gives the action
//! of the highlighted command to the screen, which runs it as its key. This module holds its state and its keys; the
//! screen draws it and runs what it gives.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Widget};

use crate::keys::{action, Action, Selection};
use crate::screen::View;
use crate::theme::{Role, Theme};

/// A command of the palette: its key, its name and the action of its key.
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Command {
    pub(crate) key: &'static str,
    pub(crate) name: &'static str,
    pub(crate) action: Action,
}

/// The commands that always act, listed after those of the selection (SUP-1).
const ALWAYS: [(&str, Action); 3] = [
    ("r refresh", Action::Refresh),
    ("g charts", Action::Charts),
    ("q quit", Action::Quit),
];

/// The commands of the palette in the read `v` with `sel` selected, a task running when `busy` (SUP-1).
pub(crate) fn commands(v: Option<&View>, sel: Selection, busy: bool) -> Vec<Command> {
    v.map_or_else(Vec::new, |v| crate::help::offered(v, sel, busy))
        .into_iter()
        .chain(ALWAYS)
        .map(|(label, action)| {
            let (key, name) = label.split_once(' ').unwrap_or((label, label));
            Command { key, name, action }
        })
        .collect()
}

/// What a key pressed in the palette asks of the screen (SUP-3, SUP-4).
#[derive(Debug, PartialEq)]
pub(crate) enum Outcome {
    /// The palette stays open, its text or its highlight maybe changed.
    Stay,
    /// The palette closes doing nothing.
    Close,
    /// The palette closes and the screen does what this key does on the panels.
    Run(Action),
}

/// The state of an open palette: the text typed and the name of the highlighted command, the first one listed when
/// none or no longer listed (SUP-2, SUP-4).
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Palette {
    pub(crate) text: String,
    highlight: Option<&'static str>,
}

impl Palette {
    /// The commands of `all` whose name holds the text, whatever its case, in their order (SUP-2).
    pub(crate) fn shown<'a>(&self, all: &'a [Command]) -> Vec<&'a Command> {
        let text = self.text.to_lowercase();
        all.iter()
            .filter(|c| c.name.to_lowercase().contains(&text))
            .collect()
    }

    /// The line of `shown` highlighted (SUP-2, SUP-4).
    pub(crate) fn highlighted(&self, shown: &[&Command]) -> usize {
        self.highlight
            .and_then(|name| shown.iter().position(|c| c.name == name))
            .unwrap_or(0)
    }

    /// Answers `key` with the commands `all` of the read shown (SUP-2 to SUP-4).
    pub(crate) fn key(&mut self, key: &KeyEvent, all: &[Command]) -> Outcome {
        if key.kind != KeyEventKind::Press {
            return Outcome::Stay;
        }
        let shown = self.shown(all);
        let at = self.highlighted(&shown);
        let mut move_to = |line: usize| self.highlight = shown.get(line).map(|c| c.name);
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Outcome::Run(Action::Quit)
            }
            KeyCode::Esc => return Outcome::Close,
            KeyCode::Enter => {
                return shown
                    .get(at)
                    .map_or(Outcome::Close, |c| Outcome::Run(c.action.clone()))
            }
            KeyCode::Up => move_to(at.saturating_sub(1)),
            KeyCode::Down => move_to((at + 1).min(shown.len().saturating_sub(1))),
            KeyCode::Backspace => {
                self.text.pop();
                self.highlight = None;
            }
            // a printable character only: a letter held with Ctrl or Alt is no text
            KeyCode::Char(c) if (key.modifiers - KeyModifiers::SHIFT).is_empty() => {
                self.text.push(c);
                self.highlight = None;
            }
            _ => {}
        }
        Outcome::Stay
    }
}

/// The action of `key` on the screen, the palette `open` or not, with the read `v` shown and `sel` selected, a task
/// running when `busy`: the key's own action when the palette is closed (`:` gives [`Action::Palette`], which the
/// screen opens it on), else what the palette answers, closing it on `Esc` and `Enter` (SUP-3, SUP-4).
pub(crate) fn pressed(
    open: &mut Option<Palette>,
    key: &KeyEvent,
    v: Option<&View>,
    (sel, busy): (Selection, bool),
) -> Action {
    let Some(p) = open.as_mut() else {
        return action(key);
    };
    match p.key(key, &commands(v, sel, busy)) {
        Outcome::Stay => Action::Nothing,
        Outcome::Close => {
            *open = None;
            Action::Nothing
        }
        Outcome::Run(a) => {
            *open = None;
            a
        }
    }
}

/// Draws the palette `p` listing `all` over the top of `area`, centred, its list scrolled to keep the highlight
/// drawn on a short screen, in the roles of `theme` (SUP-1, SUP-2).
pub(crate) fn draw(p: &Palette, all: &[Command], area: Rect, buf: &mut Buffer, theme: &Theme) {
    let shown = p.shown(all);
    let width = area.width.min(40);
    let height = area.height.min(
        u16::try_from(shown.len())
            .unwrap_or(u16::MAX)
            .saturating_add(3),
    );
    let over = Rect::new(area.x + (area.width - width) / 2, area.y, width, height);
    // the rows of the list under the text line, scrolled so the highlight stays drawn
    let rows = usize::from(height.saturating_sub(3)).max(1);
    let at = p.highlighted(&shown);
    let first = (at + 1).saturating_sub(rows);
    let lines: Vec<Line> = std::iter::once(Line::raw(format!(": {}", p.text)))
        .chain(
            shown
                .iter()
                .skip(first)
                .map(|c| Line::raw(format!("{}  {}", c.key, c.name))),
        )
        .collect();
    let block = Block::bordered()
        .border_style(theme.fg(Role::Border))
        .title(" commands ")
        .title_style(theme.fg(Role::Accent));
    let inner = block.inner(over);
    Clear.render(over, buf);
    buf.set_style(over, theme.screen());
    Paragraph::new(lines).block(block).render(over, buf);
    // the highlight across the inner row, under the text line
    let row = u16::try_from(at - first + 1).unwrap_or(u16::MAX);
    if !shown.is_empty() && row < inner.height {
        buf.set_style(
            Rect::new(inner.x, inner.y + row, inner.width, 1),
            theme.selected(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::help::Keys;
    use crate::keys::tests::view;
    use crate::keys::Focus;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn labels(list: &[Command]) -> Vec<String> {
        list.iter()
            .map(|c| format!("{} {}", c.key, c.name))
            .collect()
    }

    fn typed(p: &mut Palette, text: &str, all: &[Command]) {
        for c in text.chars() {
            assert_eq!(p.key(&press(KeyCode::Char(c)), all), Outcome::Stay);
        }
    }

    fn names(p: &Palette, all: &[Command]) -> Vec<&'static str> {
        p.shown(all).iter().map(|c| c.name).collect()
    }

    fn every_key() -> View {
        let mut v = view(&["foo"], &[("/main", Some("trunk")), ("/bar", Some("bar"))]);
        v.keys = Keys {
            start: vec![true],
            merge: vec![true],
            merged: vec![false, true],
            release: true,
            prioritize: true,
        };
        v
    }

    // spec: SUP-1 (internal: the palette opens only in a screen that needs a real terminal)
    #[test]
    fn the_palette_lists_the_commands_that_act_in_the_order_of_the_help_line() {
        let v = every_key();
        let queue = Selection::default();
        assert_eq!(
            labels(&commands(Some(&v), queue, false)),
            [
                "i summary",
                "s start",
                "f merge",
                "E end merged",
                "v release",
                "p prioritize",
                "r refresh",
                "g charts",
                "q quit"
            ]
        );
        let worktree = Selection {
            focus: Focus::Worktrees,
            issue: 0,
            worktree: 1,
        };
        assert_eq!(
            labels(&commands(Some(&v), worktree, false))[..2],
            ["e end", "E end merged"]
        );
        assert_eq!(
            labels(&commands(Some(&v), queue, true)),
            ["i summary", "r refresh", "g charts", "q quit"]
        );
        assert_eq!(
            labels(&commands(Some(&view(&[], &[])), queue, false)),
            ["r refresh", "g charts", "q quit"]
        );
        assert_eq!(commands(Some(&v), queue, false)[1].action, Action::Start);
    }

    // spec: SUP-2 (internal: the keys typed in the palette need a real terminal)
    #[test]
    fn typing_filters_the_names_whatever_their_case_and_backspace_widens_again() {
        let all = commands(Some(&every_key()), Selection::default(), false);
        let mut p = Palette::default();
        typed(&mut p, "sta", &all);
        assert_eq!(p.text, "sta");
        assert_eq!(names(&p, &all), ["start"]);
        p = Palette::default();
        typed(&mut p, "END", &all);
        assert_eq!(names(&p, &all), ["end merged"]);
        p = Palette::default();
        typed(&mut p, "xyz", &all);
        assert_eq!(names(&p, &all), Vec::<&str>::new());
        for _ in 0..3 {
            assert_eq!(p.key(&press(KeyCode::Backspace), &all), Outcome::Stay);
        }
        assert_eq!(p.text, "");
        assert_eq!(names(&p, &all).len(), all.len());
    }

    // spec: SUP-2 (internal: the keys typed in the palette need a real terminal)
    #[test]
    fn the_arrows_move_the_highlight_within_the_list_and_a_new_text_puts_it_first() {
        let all = commands(Some(&view(&[], &[])), Selection::default(), false);
        let mut p = Palette::default();
        assert_eq!(p.highlighted(&p.shown(&all)), 0);
        p.key(&press(KeyCode::Up), &all);
        assert_eq!(p.highlighted(&p.shown(&all)), 0);
        for _ in 0..5 {
            p.key(&press(KeyCode::Down), &all);
        }
        assert_eq!(p.highlighted(&p.shown(&all)), 2);
        p.key(&press(KeyCode::Up), &all);
        assert_eq!(p.highlighted(&p.shown(&all)), 1);
        typed(&mut p, "r", &all);
        assert_eq!(p.highlighted(&p.shown(&all)), 0);
    }

    // spec: SUP-3 (internal: the keys typed in the palette need a real terminal)
    #[test]
    fn enter_gives_the_action_of_the_highlighted_command_and_esc_closes() {
        let all = commands(Some(&view(&[], &[])), Selection::default(), false);
        let mut p = Palette::default();
        p.key(&press(KeyCode::Down), &all);
        assert_eq!(
            p.key(&press(KeyCode::Enter), &all),
            Outcome::Run(Action::Charts)
        );
        assert_eq!(p.key(&press(KeyCode::Esc), &all), Outcome::Close);
        typed(&mut p, "xyz", &all);
        assert_eq!(p.key(&press(KeyCode::Enter), &all), Outcome::Close);
    }

    // spec: SUP-4 (internal: the keys typed in the palette need a real terminal)
    #[test]
    fn the_letters_of_the_shortcuts_are_typed_and_only_ctrl_c_quits() {
        let all = commands(Some(&view(&[], &[])), Selection::default(), false);
        let mut p = Palette::default();
        typed(&mut p, "q", &all);
        assert_eq!(names(&p, &all), ["quit"]);
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(p.key(&ctrl_c, &all), Outcome::Run(Action::Quit));
        let mut release = press(KeyCode::Char('x'));
        release.kind = KeyEventKind::Release;
        assert_eq!(p.key(&release, &all), Outcome::Stay);
        assert_eq!(p.text, "q");
    }

    /// The action of each key of `keys` in turn on the read `v` with the first issue selected, from a closed palette.
    fn walk(v: &View, keys: &[KeyCode]) -> (Vec<Action>, Option<Palette>) {
        let mut open = None;
        let actions = keys
            .iter()
            .map(|k| {
                let a = pressed(
                    &mut open,
                    &press(*k),
                    Some(v),
                    (Selection::default(), false),
                );
                if a == Action::Palette {
                    open = Some(Palette::default());
                }
                a
            })
            .collect();
        (actions, open)
    }

    // spec: SUP-H1 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn colon_sta_enter_starts_the_selected_issue_as_its_key() {
        let v = every_key();
        let keys = [':', 's', 't', 'a'].map(KeyCode::Char);
        let (actions, open) = walk(&v, &[&keys[..], &[KeyCode::Enter]].concat());
        assert_eq!(actions[0], Action::Palette);
        assert!(actions[1..4].iter().all(|a| *a == Action::Nothing));
        assert_eq!(actions[4], Action::Start);
        assert_eq!(open, None, "Enter closes it");
        let (job, shown, stem) = crate::keys::job(&actions[4], Selection::default(), Some(&v))
            .unwrap()
            .unwrap();
        assert_eq!(
            (job, shown.as_str(), stem.as_str()),
            (crate::keys::Task::Start, "starting foo…", "foo")
        );
    }

    // spec: SUP-H2, SUP-3 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn enter_on_refresh_reads_again_and_esc_closes_doing_nothing() {
        let v = view(&[], &[]);
        let (actions, open) = walk(&v, &[KeyCode::Char(':'), KeyCode::Enter]);
        assert_eq!((actions[1].clone(), open), (Action::Refresh, None));
        let (actions, open) = walk(&v, &[KeyCode::Char(':'), KeyCode::Esc]);
        assert_eq!((actions[1].clone(), open), (Action::Nothing, None));
    }

    // spec: SUP-5 (internal: the screens are switched only in a screen that needs a real terminal)
    #[test]
    fn colon_does_nothing_on_the_charts_and_the_palette_opens_empty() {
        use crate::keys::{screen_key, Screen};
        assert_eq!(action(&press(KeyCode::Char(':'))), Action::Palette);
        for s in [Screen::Charts, Screen::Measures] {
            assert_eq!(screen_key(s, Action::Palette), (s, Action::Nothing));
        }
        assert_eq!(
            screen_key(Screen::Panels, Action::Palette),
            (Screen::Panels, Action::Palette)
        );
        let v = every_key();
        let (_, open) = walk(
            &v,
            &[
                KeyCode::Char(':'),
                KeyCode::Char('x'),
                KeyCode::Esc,
                KeyCode::Char(':'),
            ],
        );
        assert_eq!(open, Some(Palette::default()));
    }

    // spec: SUP-2 (internal: the keys typed in the palette need a real terminal)
    #[test]
    fn a_letter_held_with_ctrl_or_alt_is_not_typed() {
        let all = commands(Some(&view(&[], &[])), Selection::default(), false);
        let mut p = Palette::default();
        typed(&mut p, "st", &all);
        for m in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
            assert_eq!(
                p.key(&KeyEvent::new(KeyCode::Char('w'), m), &all),
                Outcome::Stay
            );
        }
        p.key(
            &KeyEvent::new(KeyCode::Char('R'), KeyModifiers::SHIFT),
            &all,
        );
        assert_eq!(p.text, "stR");
    }

    // spec: SUP-2 (internal: the screen is drawn only in a real terminal)
    #[test]
    fn on_a_short_screen_the_list_scrolls_to_keep_the_highlight_drawn() {
        let all = commands(Some(&every_key()), Selection::default(), false);
        let mut p = Palette::default();
        for _ in 0..7 {
            p.key(&press(KeyCode::Down), &all);
        }
        assert_eq!(p.shown(&all)[p.highlighted(&p.shown(&all))].name, "charts");
        let area = Rect::new(0, 0, 60, 7);
        let mut buf = Buffer::empty(area);
        let theme = Theme::default();
        draw(&p, &all, area, &mut buf, &theme);
        let rows: Vec<String> = (0..7)
            .map(|y| (0..60).map(|x| buf[(x, y)].symbol()).collect())
            .collect();
        let screen = rows.join("\n");
        let at = rows
            .iter()
            .position(|r| r.contains("g  charts"))
            .expect(&screen);
        assert!(
            rows[1].contains(": "),
            "the text line stays on top: {screen}"
        );
        let x = u16::try_from(rows[at].find("g  charts").unwrap()).unwrap();
        let cell = &buf[(x, u16::try_from(at).unwrap())];
        assert_eq!(
            cell.style(),
            cell.style().patch(theme.selected()),
            "{screen}"
        );
    }

    // spec: SUP-1 (internal: the screen is drawn only in a real terminal)
    #[test]
    fn the_palette_is_drawn_over_the_panels_with_its_text_and_its_highlight() {
        let all = commands(Some(&view(&[], &[])), Selection::default(), false);
        let mut p = Palette::default();
        p.key(&press(KeyCode::Char('r')), &all);
        let area = Rect::new(0, 0, 60, 12);
        let mut buf = Buffer::empty(area);
        let theme = Theme::default();
        draw(&p, &all, area, &mut buf, &theme);
        let row = |y: u16| (0..60).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let rows: Vec<String> = (0..12).map(row).collect();
        let screen = rows.join("\n");
        assert!(screen.contains(" commands "), "{screen}");
        assert!(screen.contains(": r"), "{screen}");
        let at = rows
            .iter()
            .position(|r| r.contains("r  refresh"))
            .expect(&screen);
        assert!(rows.iter().any(|r| r.contains("g  charts")), "{screen}");
        assert!(!screen.contains("q  quit"), "filtered out: {screen}");
        let x = u16::try_from(rows[at].find("r  refresh").unwrap()).unwrap();
        assert_eq!(
            buf[(x, u16::try_from(at).unwrap())].style(),
            buf[(x, u16::try_from(at).unwrap())]
                .style()
                .patch(theme.selected())
        );
    }

    // spec: SUP-4 (internal: the reads redraw the palette only in a screen that needs a real terminal)
    #[test]
    fn a_new_read_keeps_the_highlight_on_its_command_or_puts_it_first() {
        let mut v = every_key();
        let sel = Selection::default();
        let before = commands(Some(&v), sel, false);
        let mut p = Palette::default();
        p.key(&press(KeyCode::Down), &before);
        p.key(&press(KeyCode::Down), &before);
        assert_eq!(
            p.shown(&before)[p.highlighted(&p.shown(&before))].name,
            "merge"
        );
        v.keys.start = vec![false];
        let after = commands(Some(&v), sel, false);
        assert_eq!(
            p.shown(&after)[p.highlighted(&p.shown(&after))].name,
            "merge"
        );
        v.keys.merge = vec![false];
        let gone = commands(Some(&v), sel, false);
        assert_eq!(p.highlighted(&p.shown(&gone)), 0);
    }
}
