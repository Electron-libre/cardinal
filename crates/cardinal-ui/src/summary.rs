//! The summary of the row selected in the queue (SUQ-6, SUQ-7): a panel under the queue, titled with the title of
//! the issue and holding its goal, or with the name of the milestone and holding its state, goal and why, wrapped at
//! the words, that `i` unfolds and folds; it is folded when the screen starts and only shown while the selection is
//! in the queue.

use std::borrow::Cow;

use ratatui::layout::Rect;

use crate::keys::{Focus, Row, Selection};
use crate::panels::Panel;
use crate::screen::{layout, View};
use crate::source::{Milestone, Queue};
use crate::status::Status;
use crate::status::{columns, split};

/// What the summary of a row shows: its title and its text, wrapped at the words (SUQ-6, SUQ-7).
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub title: String,
    pub text: String,
}

/// The summaries of the rows `rows` of `queue` and of the milestones `milestones` of `roadmap`, in their order.
pub fn of(queue: Option<&Queue>, milestones: &[Milestone], rows: &[Row]) -> Vec<Option<Summary>> {
    let issues = queue.map_or(&[][..], |q| &q.issues[..]);
    rows.iter()
        .map(|row| match row {
            Row::Issue(stem) => {
                let issue = issues.iter().find(|i| &i.stem == stem);
                Some(Summary {
                    title: issue
                        .and_then(|i| i.title.clone())
                        .unwrap_or_else(|| stem.clone()),
                    text: issue
                        .and_then(|i| i.goal.clone())
                        .unwrap_or_else(|| "no goal".to_string()),
                })
            }
            Row::Milestone(name) => milestones.iter().find(|m| &m.name == name).map(milestone),
        })
        .collect()
}

/// The summary of the milestone `m`: its name over its state, its goal and its why (SUQ-6).
fn milestone(m: &Milestone) -> Summary {
    let hold = match m.on_hold.as_deref() {
        Some("") => ", on hold".to_string(),
        Some(reason) => format!(", on hold: {reason}"),
        None => String::new(),
    };
    let state = format!("state: {}{hold}", m.state.as_deref().unwrap_or("(none)"));
    let field = |name, value: &Option<String>| {
        value
            .as_ref()
            .map_or_else(|| format!("no {name}"), |v| format!("{name}: {v}"))
    };
    Summary {
        title: m.name.clone(),
        text: [state, field("goal", &m.goal), field("why", &m.why)].join("\n"),
    }
}

/// The summary of the issue or the milestone `sel` selects in `v`: none while the selection is in the worktrees, or
/// the queue holds no row the selection stops on (STU-14) (an empty queue, or a `next-issue` that cannot be read, STU-6); the help line offers `i` on it (SUA-15).
pub(crate) fn selected(v: &View, sel: Selection) -> Option<&Summary> {
    if sel.focus == Focus::Queue {
        v.summaries.get(sel.row).and_then(Option::as_ref)
    } else {
        None
    }
}

/// Whether the summary is unfolded after `i` pressed with `sel` selected in the read `v`, `unfolded` before it: the
/// key acts only with the selection on a row of the queue.
pub(crate) fn toggle(unfolded: bool, sel: Selection, v: Option<&View>) -> bool {
    unfolded ^ v.and_then(|v| selected(v, sel)).is_some()
}

/// The panels of the screen over `area`, with `status` under them: the queue, the summary of the selected row when
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
    let summary = |room| Panel {
        title: Cow::Owned(s.title.clone()),
        lines: rows(&s.text, width, room),
        marks: Vec::new(),
        headings: Vec::new(),
        failed: false,
        corner: None,
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

/// `text` wrapped at the words to `width` columns, a wider word cut across the rows, each paragraph from a row of
/// its own, at most `room` rows, the last ending with `…` when it needs more (SUQ-7).
fn rows(text: &str, width: usize, room: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for paragraph in text.split('\n') {
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
        v.summaries = of(Some(&queue), &[], &v.rows);
        v
    }

    /// The read of `queue_view` with the heading of the milestone `One` above its issues, `one` its milestone.
    fn milestone_view(one: Milestone) -> View {
        let mut v = queue_view();
        v.panels[0].lines.insert(0, "■ One".to_string());
        v.rows.insert(0, Row::Milestone("One".to_string()));
        v.row_lines = vec![0, 1, 2];
        let issues = |v: &View| v.summaries.clone();
        let mut summaries = of(None, &[one], &v.rows[..1]);
        summaries.extend(issues(&v));
        v.summaries = summaries;
        v
    }

    fn one(
        state: Option<&str>,
        on_hold: Option<&str>,
        goal: Option<&str>,
        why: Option<&str>,
    ) -> Milestone {
        let text = |s: Option<&str>| s.map(str::to_string);
        Milestone {
            name: "One".to_string(),
            state: text(state),
            on_hold: text(on_hold),
            goal: text(goal),
            why: text(why),
            document: None,
            open: 2,
            listed: 2,
            awaits_release: false,
            proposed: None,
        }
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
        after.rows.reverse();
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

    // spec: SUQ-H4, SUQ-6, SUA-15 (internal: the screen needs a real terminal)
    #[test]
    fn i_on_a_milestone_unfolds_its_state_goal_and_why_and_the_fold_follows_the_selection() {
        let v = milestone_view(one(
            Some("in issues"),
            Some("after v2"),
            Some("make it work"),
            Some("it helps"),
        ));
        let mut sel = Selection::default();
        assert!(crate::help::line(&v, sel, false).contains("i summary"));
        let unfolded = toggle(false, sel, Some(&v));
        let drawn = screen(&v, sel, unfolded, 20);
        let rows: Vec<&str> = drawn.lines().collect();
        let title = rows
            .iter()
            .position(|r| r.contains("┌ One "))
            .unwrap_or_else(|| panic!("a summary titled One: {drawn}"));
        assert!(
            rows[title + 1].contains("state: in issues, on hold: after v2"),
            "{drawn}"
        );
        assert!(rows[title + 2].contains("goal: make it work"), "{drawn}");
        assert!(rows[title + 3].contains("why: it helps"), "{drawn}");
        sel.apply(&Action::Down, &v);
        let a = screen(&v, sel, unfolded, 20);
        assert!(a.contains("┌ Do a ") && !a.contains("┌ One "), "{a}");
        sel.apply(&Action::Up, &v);
        assert!(
            screen(&v, sel, unfolded, 20).contains("┌ One "),
            "back on the heading"
        );
        assert!(!toggle(unfolded, sel, Some(&v)), "i again folds it");
    }

    // spec: SUQ-6 (internal: the screen needs a real terminal)
    #[test]
    fn a_milestone_with_no_fields_reads_none_and_a_bare_hold() {
        let text = |m| {
            of(None, &[m], &[Row::Milestone("One".to_string())])[0]
                .clone()
                .unwrap()
                .text
        };
        assert_eq!(
            text(one(None, None, None, None)),
            "state: (none)\nno goal\nno why"
        );
        assert_eq!(
            text(one(Some("shaped"), Some(""), None, Some("y"))),
            "state: shaped, on hold\nno goal\nwhy: y"
        );
        assert_eq!(of(None, &[], &[Row::Milestone("One".to_string())]), [None]);
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
        v.summaries[0].as_mut().unwrap().text = (1..=10)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
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
        v.summaries[0].as_mut().unwrap().text = (1..=20)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
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
