//! The summary of the issue selected in the queue (SUQ-6, SUQ-7): a panel under the queue, titled with the title of
//! the issue and holding its goal wrapped at the words, that `i` unfolds and folds; it is folded when the screen
//! starts and only shown while the selection is on an issue of the queue.

use std::borrow::Cow;

use ratatui::layout::Rect;

use crate::keys::{Focus, Selection};
use crate::panels::Panel;
use crate::screen::{layout, View};
use crate::source::Queue;
use crate::status::Status;
use crate::status::{columns, split};

/// What the summary of an issue shows: its title, its stem when it has none, and its goal (NXT-19).
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub title: String,
    pub goal: Option<String>,
}

/// The summaries of the issues `stems` of `queue`, in their order.
pub fn of(queue: Option<&Queue>, stems: &[String]) -> Vec<Summary> {
    let issues = queue.map_or(&[][..], |q| &q.issues[..]);
    stems
        .iter()
        .map(|stem| {
            let issue = issues.iter().find(|i| &i.stem == stem);
            Summary {
                title: issue
                    .and_then(|i| i.title.clone())
                    .unwrap_or_else(|| stem.clone()),
                goal: issue.and_then(|i| i.goal.clone()),
            }
        })
        .collect()
}

/// The summary of the issue `sel` selects in `v`: none while the selection is in the worktrees or the queue holds no
/// issue (an empty queue, or a `next-issue` that cannot be read, STU-6); the help line offers `i` on it (SUA-15).
pub(crate) fn selected(v: &View, sel: Selection) -> Option<&Summary> {
    if sel.focus == Focus::Queue {
        v.summaries.get(sel.issue)
    } else {
        None
    }
}

/// Whether the summary is unfolded after `i` pressed with `sel` selected in the read `v`, `unfolded` before it: the
/// key acts only with the selection on an issue of the queue.
pub(crate) fn toggle(unfolded: bool, sel: Selection, v: Option<&View>) -> bool {
    unfolded ^ v.and_then(|v| selected(v, sel)).is_some()
}

/// The panels of the screen over `area`, with `status` under them: the queue, the summary of the selected issue when
/// it is `unfolded`, then the worktrees; its rows of text at most a third of the screen and no more than the layout
/// gives it, so its `…` stays shown (SUQ-7).
pub(crate) fn panels<'a>(
    v: &'a View,
    sel: Selection,
    unfolded: bool,
    (area, status): (Rect, &Status),
) -> Cow<'a, [Panel]> {
    let shown = &v.panels[..2];
    let Some(s) = selected(v, sel).filter(|_| unfolded) else {
        return Cow::Borrowed(shown);
    };
    let width = usize::from(area.width.saturating_sub(2).max(1));
    let goal = s.goal.as_deref().unwrap_or("no goal");
    let summary = |room| Panel {
        title: Cow::Owned(s.title.clone()),
        lines: rows(goal, width, room),
        marks: Vec::new(),
        headings: Vec::new(),
        failed: false,
    };
    let mut room = usize::from(area.height / 3).max(1);
    let mut panels = vec![shown[0].clone(), summary(room), shown[1].clone()];
    // the layout shares a short screen between the panels: fewer rows until the summary gets what it holds
    for _ in 0..3 {
        let given = usize::from(layout(area, &panels, status)[1].height.saturating_sub(2)).max(1);
        if given >= panels[1].lines.len() {
            break;
        }
        room = given;
        panels[1] = summary(room);
    }
    Cow::Owned(panels)
}

/// `goal` wrapped at the words to `width` columns, a wider word cut across the rows, each paragraph from a row of
/// its own, at most `room` rows, the last ending with `…` when it needs more (SUQ-7).
fn rows(goal: &str, width: usize, room: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for paragraph in goal.split('\n') {
        let mut row = String::new();
        for word in paragraph.split_whitespace() {
            if !row.is_empty() && columns(&row) + 1 + columns(word) <= width {
                row.push(' ');
                row.push_str(word);
                continue;
            }
            if !row.is_empty() {
                rows.push(std::mem::take(&mut row));
            }
            let mut rest = word;
            loop {
                let (head, tail) = split(rest, width);
                if tail.is_empty() {
                    row = head.to_string();
                    break;
                }
                rows.push(head.to_string());
                rest = tail;
            }
        }
        if !row.is_empty() {
            rows.push(row);
        }
    }
    if rows.len() > room {
        rows.truncate(room);
        if let Some(last) = rows.last_mut() {
            let kept = if columns(last) < width {
                last.as_str()
            } else if width > 1 {
                split(last, width - 1).0
            } else {
                // `split` keeps one grapheme at least: a row of one column holds the `…` alone
                ""
            };
            *last = format!("{kept}…");
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::tests::view;
    use crate::keys::{action, screen_key, Action, Screen};
    use crate::source::Issue;
    use crate::status::Status;
    use crate::theme::Theme;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// A read whose queue holds `a`, titled `Do a` with a goal, then `b` with no title nor goal.
    fn queue_view() -> View {
        let mut v = view(&["a", "b"], &[("/main", Some("trunk"))]);
        v.panels[0].lines = vec!["> a".to_string(), "  b".to_string()];
        v.panels[1].lines = vec!["/main".to_string()];
        let issue = |stem: &str, title: Option<&str>, goal: Option<&str>| Issue {
            stem: stem.to_string(),
            title: title.map(str::to_string),
            goal: goal.map(str::to_string),
            priority: None,
            state: "ready".to_string(),
            milestone: None,
            depends_on: Vec::new(),
            taken: None,
            tab: false,
        };
        let queue = Queue {
            issues: vec![
                issue("b", None, None),
                issue("a", Some("Do a"), Some("Make a work. Then test it.")),
            ],
            ..Queue::default()
        };
        v.summaries = of(Some(&queue), &v.stems);
        v
    }

    /// The rows of the panels of `v` with `sel` selected and the summary `unfolded`, drawn 80 columns wide.
    fn screen(v: &View, sel: Selection, unfolded: bool, height: u16) -> String {
        let mut terminal = ratatui::Terminal::new(TestBackend::new(80, height)).unwrap();
        terminal
            .draw(|f| {
                crate::screen::draw(
                    f,
                    Some(v),
                    (sel, Screen::Panels, false, unfolded, None),
                    &Status::default(),
                    &Theme::default(),
                );
            })
            .unwrap();
        let buf = terminal.backend().buffer();
        (0..height)
            .map(|y| (0..80).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    // spec: SUQ-H3, SUQ-6, STU-1 (internal: the screen needs a real terminal)
    #[test]
    fn i_unfolds_the_summary_of_the_selected_issue_under_the_queue_and_folds_it() {
        let i = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        assert_eq!(action(&i), Action::Summary);
        let v = queue_view();
        let mut sel = Selection::default();
        assert!(!screen(&v, sel, false, 20).contains("Do a"), "folded");
        let unfolded = toggle(false, sel, Some(&v));
        let drawn = screen(&v, sel, unfolded, 20);
        let rows: Vec<&str> = drawn.lines().collect();
        let at = |s: &str| rows.iter().position(|r| r.contains(s));
        let title = at("┌ Do a ").unwrap_or_else(|| panic!("a summary titled Do a: {drawn}"));
        assert!(
            at("> a").unwrap() < title && title < at("/main").unwrap(),
            "{drawn}"
        );
        assert!(
            rows[title + 1].contains("Make a work. Then test it."),
            "{drawn}"
        );
        sel.apply(&Action::Down, &v);
        let b = screen(&v, sel, unfolded, 20);
        assert!(
            b.contains("┌ b ") && b.contains("no goal") && !b.contains("Do a"),
            "{b}"
        );
        assert!(!toggle(unfolded, sel, Some(&v)), "i again folds it");
    }

    // spec: SUQ-6 (internal: the screen needs a real terminal)
    #[test]
    fn the_summary_waits_while_the_selection_is_in_the_worktrees_and_stays_unfolded_across_reads() {
        let v = queue_view();
        let mut sel = Selection::default();
        sel.apply(&Action::Switch, &v);
        assert!(!screen(&v, sel, true, 20).contains("Do a"), "hidden");
        assert!(toggle(true, sel, Some(&v)), "i acts on the queue only");
        sel.apply(&Action::Switch, &v);
        assert!(screen(&v, sel, true, 20).contains("Do a"), "shown again");
        let mut after = queue_view();
        after.stems.reverse();
        after.summaries.reverse();
        sel.reread(&v, &after);
        assert!(
            screen(&after, sel, true, 20).contains("Do a"),
            "after a read"
        );
        let empty = view(&[], &[]);
        assert!(
            !toggle(false, Selection::default(), Some(&empty)),
            "no issue"
        );
        assert!(!toggle(false, Selection::default(), None), "no read yet");
        assert_eq!(
            screen_key(Screen::Charts, Action::Summary),
            (Screen::Charts, Action::Nothing)
        );
    }

    // spec: SUQ-7 (internal: the screen needs a real terminal)
    #[test]
    fn the_goal_wraps_at_the_words_on_a_third_of_the_screen_at_most() {
        assert_eq!(rows("one two three", 7, 9), ["one two", "three"]);
        assert_eq!(rows("first\nsecond", 40, 9), ["first", "second"]);
        assert_eq!(rows("abcdefghij", 4, 9), ["abcd", "efgh", "ij"]);
        assert_eq!(rows("aaaa bbbb", 4, 1), ["aaa…"]);
        assert_eq!(rows("ab cd", 1, 1), ["…"]);
        let mut v = queue_view();
        v.summaries[0].goal = Some(
            (1..=10)
                .map(|n| format!("line {n}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let drawn = screen(&v, Selection::default(), true, 15);
        let rows: Vec<&str> = drawn.lines().collect();
        let title = rows.iter().position(|r| r.contains("┌ Do a ")).unwrap();
        assert!(rows[title + 5].contains("line 5…"), "{drawn}");
        assert!(
            rows[title + 6].starts_with('└') && !drawn.contains("line 6"),
            "{drawn}"
        );
    }

    // spec: SUQ-7 (internal: the screen needs a real terminal)
    #[test]
    fn on_a_short_screen_the_last_row_shown_keeps_its_ellipsis() {
        let mut v = queue_view();
        v.panels[1].lines = (1..=8).map(|n| format!("/ws{n}")).collect();
        v.summaries[0].goal = Some(
            (1..=20)
                .map(|n| format!("line {n}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let drawn = screen(&v, Selection::default(), true, 15);
        let rows: Vec<&str> = drawn.lines().collect();
        let title = rows.iter().position(|r| r.contains("┌ Do a ")).unwrap();
        let bottom = rows[title..]
            .iter()
            .position(|r| r.starts_with('└'))
            .unwrap();
        assert!(bottom > 1, "{drawn}");
        assert!(rows[title + bottom - 1].contains('…'), "{drawn}");
    }
}
