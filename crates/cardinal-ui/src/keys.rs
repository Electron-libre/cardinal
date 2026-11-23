//! The keys of the screen of `cardinal-ui` (STU-2): the action of each key, the selection in the queue or the
//! worktrees it moves (STU-14, SUE-1), and the task a key runs on what is selected, one at a time (SUA-1, SUM-1,
//! SUA-4 to SUA-6, SUA-16, SUE-1, SUE-2).

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::screen::View;
use crate::status::Status;

/// The panel the selection is in (SUE-1).
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub(crate) enum Focus {
    #[default]
    Queue,
    Worktrees,
}

/// A row of the queue the selection rests on (STU-14): an issue by its stem, or the heading of a milestone that
/// `roadmap` lists by its name.
#[derive(Debug, Clone, PartialEq)]
pub enum Row {
    Issue(String),
    Milestone(String),
}

/// Why `s` and `f` act on nothing with the selection on a milestone (STU-14).
const ON_A_MILESTONE: &str = "s and f act on an issue: select one";

/// Why `c` acts on nothing with the selection on an issue or a worktree (SUA-19).
const NOT_A_MILESTONE: &str = "c acts on a milestone: select its heading";

/// The selected row of the queue and line of the worktrees, and the panel the keys move it in (SUA-1, SUE-1); the
/// screen starts on the first issue of the queue ([`Selection::start`]).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Selection {
    pub(crate) focus: Focus,
    pub(crate) row: usize,
    pub(crate) worktree: usize,
}

impl Selection {
    /// Moves the selection on `action` in the view `v`: `Tab` to the other panel, the arrows within it.
    pub(crate) fn apply(&mut self, action: &Action, v: &View) {
        match (action, self.focus) {
            (Action::Switch, Focus::Queue) => self.focus = Focus::Worktrees,
            (Action::Switch, Focus::Worktrees) => self.focus = Focus::Queue,
            (_, Focus::Queue) => self.row = select(self.row, v.rows.len(), action),
            (_, Focus::Worktrees) => {
                self.worktree = select(self.worktree, v.worktrees.len(), action);
            }
        }
    }

    /// Keeps the selection on the same issue and the same worktree path from the read `before` to the read `after`
    /// (STU-14, SUE-1); after a read with no row, it starts as on the first read.
    pub(crate) fn reread(&mut self, before: &View, after: &View) {
        if before.rows.is_empty() {
            self.start(after);
        } else {
            self.row = reselect(&before.rows, self.row, &after.rows);
        }
        let paths = |v: &View| v.worktrees.iter().map(|w| w.0.clone()).collect::<Vec<_>>();
        self.worktree = reselect(&paths(before), self.worktree, &paths(after));
    }

    /// The panel and the line to highlight in `v`: none in an empty or failed panel.
    pub(crate) fn line(&self, v: &View) -> Option<(usize, usize)> {
        match self.focus {
            Focus::Queue => v.row_lines.get(self.row).map(|line| (0, *line)),
            Focus::Worktrees => (!v.worktrees.is_empty()).then_some((1, self.worktree)),
        }
    }

    /// The row the screen starts on in the first read `v`: its first issue, else its first row (STU-14).
    pub(crate) fn start(&mut self, v: &View) {
        self.row = v
            .rows
            .iter()
            .position(|r| matches!(r, Row::Issue(_)))
            .unwrap_or(0);
    }
}

impl View {
    /// The stem of the issue on the row `row` of the queue, `None` on a milestone or past its end.
    pub(crate) fn stem(&self, row: usize) -> Option<&str> {
        match self.rows.get(row) {
            Some(Row::Issue(stem)) => Some(stem),
            _ => None,
        }
    }
}

/// What a key does in the screen (STU-2).
#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    Quit,
    Refresh,
    Up,
    Down,
    Switch,
    Start,
    Merge,
    End,
    EndMerged,
    Release,
    /// `p`: starts the review of the priorities (SUA-16).
    Prioritize,
    /// `c`: starts the cut of the selected milestone into issues (SUA-19).
    Cut,
    Measure,
    Charts,
    /// `i`: folds or unfolds the summary of the selected issue (SUQ-6).
    Summary,
    /// `:`: opens the command palette on the panels (SUP-1).
    Palette,
    /// `Esc`: back to the panels from the charts and the measures (SUC-1), else quit (STU-2).
    Back,
    Nothing,
}

/// A task on the selected issue that changes the project, one at a time (SUA-6).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Task {
    Start,
    Merge,
    End,
    EndMerged,
    Release,
    Prioritize,
    Cut,
}

/// The task `action` runs on the issue `stem`, with the status line shown while it runs.
pub(crate) fn task(action: &Action, stem: &str) -> Option<(Task, String)> {
    match action {
        Action::Start => Some((Task::Start, format!("starting {stem}…"))),
        Action::Merge => Some((Task::Merge, format!("merging {stem}…"))),
        Action::End => Some((Task::End, format!("ending {stem}…"))),
        Action::EndMerged => Some((Task::EndMerged, "ending the merged worktrees…".to_string())),
        Action::Release => Some((Task::Release, "starting the release…".to_string())),
        Action::Prioritize => Some((
            Task::Prioritize,
            "starting the review of the priorities…".to_string(),
        )),
        Action::Cut => Some((Task::Cut, format!("starting the cut of {stem}…"))),
        _ => None,
    }
}

/// The task `action` runs on what `sel` selects in `v` (`None` before the first read): the task, the status line
/// while it runs and the stem; `None` for another key, an empty panel or no read yet; why it runs none when the key
/// belongs to the other panel or the worktree is detached (SUA-1, SUM-1, SUE-1). The release covers every
/// milestone awaiting it, the review of the priorities every open issue, the end of the merged worktrees every one
/// of them: they need no selection nor read (SUA-4, SUA-16, SUE-2).
pub(crate) fn job(
    action: &Action,
    sel: Selection,
    v: Option<&View>,
) -> Result<Option<(Task, String, String)>, String> {
    if matches!(
        action,
        Action::Release | Action::Prioritize | Action::EndMerged
    ) {
        return Ok(task(action, "").map(|(t, shown)| (t, shown, String::new())));
    }
    let Some(v) = v else { return Ok(None) };
    let stem = match (action, sel.focus) {
        (Action::Start | Action::Merge, Focus::Queue) => match v.rows.get(sel.row) {
            Some(Row::Milestone(_)) => return Err(ON_A_MILESTONE.to_string()),
            _ => v.stem(sel.row).map(str::to_string),
        },
        (Action::Start | Action::Merge, Focus::Worktrees) => {
            return Err("s and f act on the queue: Tab to it".to_string())
        }
        (Action::End, Focus::Worktrees) => match v.worktrees.get(sel.worktree) {
            Some((path, None)) => return Err(format!("{path}: detached, no issue to end")),
            Some((_, Some(branch))) => Some(branch.clone()),
            None => None,
        },
        (Action::End, Focus::Queue) => {
            return Err("e acts on a worktree: Tab to the worktrees".to_string())
        }
        (Action::Cut, Focus::Queue) => match v.rows.get(sel.row) {
            Some(Row::Milestone(name)) => Some(name.clone()),
            Some(Row::Issue(_)) => return Err(NOT_A_MILESTONE.to_string()),
            None => None,
        },
        (Action::Cut, Focus::Worktrees) => return Err(NOT_A_MILESTONE.to_string()),
        _ => return Ok(None),
    };
    Ok(stem.and_then(|s| task(action, &s).map(|(t, shown)| (t, shown, s))))
}

/// The action of the key `key`.
pub fn action(key: &KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::Nothing;
    }
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Esc => Action::Back,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Up | KeyCode::Char('k') => Action::Up,
        KeyCode::Down | KeyCode::Char('j') => Action::Down,
        KeyCode::Tab => Action::Switch,
        KeyCode::Char('s') => Action::Start,
        KeyCode::Char('f') => Action::Merge,
        KeyCode::Char('e') => Action::End,
        KeyCode::Char('E') => Action::EndMerged,
        KeyCode::Char('v') => Action::Release,
        KeyCode::Char('p') => Action::Prioritize,
        KeyCode::Char('c') => Action::Cut,
        KeyCode::Char('m') => Action::Measure,
        KeyCode::Char('g') => Action::Charts,
        KeyCode::Char('i') => Action::Summary,
        KeyCode::Char(':') => Action::Palette,
        _ => Action::Nothing,
    }
}

/// The screen `cardinal-ui` shows (SUC-1): its panels, the charts of the measures, or the table of the measures.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Screen {
    Panels,
    Charts,
    Measures,
}

/// The screen shown after `action` on `shown`, and what it then does (SUC-1): `g` goes from the panels to the charts,
/// to the measures and back to the panels; on the charts and the measures `Esc` goes back to the panels and only the
/// keys of the reads and of the measures act, so the selection and the tasks wait; on the panels `Esc` quits and
/// `m` does nothing.
pub(crate) fn screen_key(shown: Screen, action: Action) -> (Screen, Action) {
    match (shown, action) {
        (Screen::Panels, Action::Charts) => (Screen::Charts, Action::Nothing),
        (Screen::Charts, Action::Charts) => (Screen::Measures, Action::Nothing),
        (Screen::Panels, Action::Back) => (Screen::Panels, Action::Quit),
        // back to the panels from the last screen of `g` or on `Esc`; no measure from the panels
        (Screen::Measures, Action::Charts)
        | (Screen::Panels, Action::Measure)
        | (_, Action::Back) => (Screen::Panels, Action::Nothing),
        (s, a @ (Action::Quit | Action::Refresh | Action::Measure)) => (s, a),
        (Screen::Panels, a) => (Screen::Panels, a),
        (s, _) => (s, Action::Nothing),
    }
}

/// The selection in the rows `after` of a new read, given the one `selected` in the rows `before`: the row of the
/// same issue or milestone, else the same rank kept in the queue (STU-14, SUE-1).
pub(crate) fn reselect<T: PartialEq>(before: &[T], selected: usize, after: &[T]) -> usize {
    before
        .get(selected)
        .and_then(|row| after.iter().position(|r| r == row))
        .unwrap_or_else(|| select(selected, after.len(), &Action::Nothing))
}

/// Whether the screen quits now on `action`, and whether it quits once the task running (`busy`) ends (SUA-6).
pub(crate) fn quits(action: &Action, busy: bool, quitting: bool) -> (bool, bool) {
    match action {
        Action::Quit if busy => (false, true),
        Action::Quit => (true, quitting),
        _ => (false, quitting),
    }
}

/// What the end of a task gives (SUA-6).
#[derive(Debug, PartialEq)]
pub(crate) enum Ended {
    /// The status line it shows: its line, in the role of its result (SUS-2).
    Show(Status),
    /// The user quit during the task: the screen quits, and this line is printed once the terminal is given back.
    Quit(String),
}

/// The end of a task with `result`, the user having quit during it or not (SUA-6).
pub(crate) fn ended(result: Result<String, String>, quitting: bool) -> Ended {
    if quitting {
        Ended::Quit(result.unwrap_or_else(|e| e))
    } else {
        Ended::Show(Status::of(result))
    }
}

/// The selection after `action` in a queue of `len` issues, kept in it (SUA-1).
pub(crate) fn select(selected: usize, len: usize, action: &Action) -> usize {
    let moved = match action {
        Action::Up => selected.saturating_sub(1),
        Action::Down => selected.saturating_add(1),
        _ => selected,
    };
    moved.min(len.saturating_sub(1))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::panels::Panel;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    // spec: STU-2, STU-17 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn q_esc_and_ctrl_c_quit_and_r_refreshes() {
        assert_eq!(action(&press(KeyCode::Char('q'))), Action::Quit);
        assert_eq!(
            screen_key(Screen::Panels, action(&press(KeyCode::Esc))).1,
            Action::Quit
        );
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(action(&ctrl_c), Action::Quit);
        assert_eq!(action(&press(KeyCode::Char('c'))), Action::Cut);
        assert_eq!(action(&press(KeyCode::Char('r'))), Action::Refresh);
        assert_eq!(action(&press(KeyCode::Char('m'))), Action::Measure);
        assert_eq!(action(&press(KeyCode::Char('x'))), Action::Nothing);
        let mut release = press(KeyCode::Char('q'));
        release.kind = KeyEventKind::Release;
        assert_eq!(action(&release), Action::Nothing);
    }

    // spec: SUA-1 (internal: the keys and the selection of a screen that needs a real terminal)
    #[test]
    fn the_arrows_and_j_k_move_the_selection_and_s_starts() {
        assert_eq!(action(&press(KeyCode::Up)), Action::Up);
        assert_eq!(action(&press(KeyCode::Char('k'))), Action::Up);
        assert_eq!(action(&press(KeyCode::Down)), Action::Down);
        assert_eq!(action(&press(KeyCode::Char('j'))), Action::Down);
        assert_eq!(action(&press(KeyCode::Char('s'))), Action::Start);
        assert_eq!(select(0, 3, &Action::Down), 1);
        assert_eq!(select(2, 3, &Action::Down), 2, "kept on the last issue");
        assert_eq!(select(0, 3, &Action::Up), 0, "kept on the first issue");
        assert_eq!(
            select(5, 2, &Action::Nothing),
            1,
            "a shorter queue moves it back"
        );
        assert_eq!(select(0, 0, &Action::Down), 0);
    }

    // spec: SUM-1 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn f_merges_the_selected_issue() {
        assert_eq!(action(&press(KeyCode::Char('f'))), Action::Merge);
        assert_eq!(
            task(&Action::Merge, "foo").map(|(_, s)| s),
            Some("merging foo…".to_string())
        );
        assert_eq!(
            task(&Action::Start, "foo").map(|(_, s)| s),
            Some("starting foo…".to_string())
        );
        assert!(task(&Action::Refresh, "foo").is_none());
    }

    // spec: SUA-4, SUA-6 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn v_starts_the_release_from_either_panel_and_before_the_first_read() {
        assert_eq!(action(&press(KeyCode::Char('v'))), Action::Release);
        let v = view(&["bar"], &[("/main", Some("trunk"))]);
        for (focus, read) in [
            (Focus::Queue, Some(&v)),
            (Focus::Worktrees, Some(&v)),
            (Focus::Queue, None),
        ] {
            let sel = Selection {
                focus,
                row: 0,
                worktree: 0,
            };
            assert_eq!(
                job(&Action::Release, sel, read),
                Ok(Some((
                    Task::Release,
                    "starting the release…".to_string(),
                    String::new()
                )))
            );
        }
        assert_eq!(quits(&Action::Quit, true, false), (false, true));
    }

    // spec: SUA-16, SUA-6, SUA-15 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn p_starts_the_review_of_the_priorities_from_either_panel_and_before_the_first_read() {
        assert_eq!(action(&press(KeyCode::Char('p'))), Action::Prioritize);
        let v = view(&["bar"], &[("/main", Some("trunk"))]);
        // an empty queue still runs the task, which says why it starts nothing (SUA-15, SUA-17)
        let empty = view(&[], &[]);
        for (focus, read) in [
            (Focus::Queue, Some(&v)),
            (Focus::Worktrees, Some(&v)),
            (Focus::Queue, None),
            (Focus::Queue, Some(&empty)),
        ] {
            let sel = Selection {
                focus,
                row: 0,
                worktree: 0,
            };
            assert_eq!(
                job(&Action::Prioritize, sel, read),
                Ok(Some((
                    Task::Prioritize,
                    "starting the review of the priorities…".to_string(),
                    String::new()
                )))
            );
        }
    }

    // spec: SUA-6 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn quit_during_a_start_or_a_merge_waits_for_it() {
        assert_eq!(quits(&Action::Quit, false, false), (true, false));
        assert_eq!(quits(&Action::Quit, true, false), (false, true));
        assert_eq!(quits(&Action::Refresh, true, true), (false, true));
        assert_eq!(quits(&Action::Down, false, false), (false, false));
        assert_eq!(
            ended(Ok("merged".into()), false),
            Ended::Show(Status::of(Ok("merged".into())))
        );
        assert_eq!(
            ended(Err("merged; cardinal check: exit 1".into()), true),
            Ended::Quit("merged; cardinal check: exit 1".into()),
            "printed once the terminal is given back"
        );
    }

    pub(crate) fn view(stems: &[&str], worktrees: &[(&str, Option<&str>)]) -> View {
        let panel = |title: &'static str| Panel {
            title: title.into(),
            lines: Vec::new(),
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
        };
        View {
            panels: [panel("Queue"), panel("Worktrees"), panel("Measures")],
            rows: stems.iter().map(|s| Row::Issue((*s).to_string())).collect(),
            row_lines: (0..stems.len()).collect(),
            worktrees: worktrees
                .iter()
                .map(|(p, b)| ((*p).to_string(), b.map(str::to_string)))
                .collect(),
            waits: Some(Vec::new()),
            charts: crate::charts::charts(None),
            keys: crate::help::Keys::default(),
            summaries: stems
                .iter()
                .map(|s| {
                    Some(crate::summary::Summary {
                        title: (*s).to_string(),
                        goal: None,
                    })
                })
                .collect(),
        }
    }

    // spec: SUC-1 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn g_cycles_the_panels_the_charts_and_the_measures_where_only_the_keys_of_the_reads_act() {
        use Screen::{Charts, Measures, Panels};
        assert_eq!(action(&press(KeyCode::Char('g'))), Action::Charts);
        assert_eq!(
            screen_key(Panels, Action::Charts),
            (Charts, Action::Nothing)
        );
        assert_eq!(
            screen_key(Charts, Action::Charts),
            (Measures, Action::Nothing)
        );
        assert_eq!(
            screen_key(Measures, Action::Charts),
            (Panels, Action::Nothing)
        );
        for shown in [Charts, Measures] {
            for idle in [
                Action::Start,
                Action::Merge,
                Action::End,
                Action::EndMerged,
                Action::Release,
                Action::Prioritize,
                Action::Up,
                Action::Down,
                Action::Switch,
            ] {
                assert_eq!(
                    screen_key(shown, idle),
                    (shown, Action::Nothing),
                    "no task, the selection kept"
                );
            }
            for acts in [Action::Quit, Action::Refresh, Action::Measure] {
                assert_eq!(screen_key(shown, acts.clone()), (shown, acts));
            }
            assert_eq!(
                screen_key(shown, Action::Back),
                (Panels, Action::Nothing),
                "Esc back to the panels"
            );
        }
        assert_eq!(action(&press(KeyCode::Esc)), Action::Back);
        assert_eq!(
            screen_key(Panels, Action::Back),
            (Panels, Action::Quit),
            "Esc quits from the panels"
        );
        assert_eq!(
            screen_key(Panels, Action::Measure),
            (Panels, Action::Nothing),
            "no measure from the panels"
        );
        assert_eq!(screen_key(Panels, Action::Start), (Panels, Action::Start));
    }

    // spec: SUE-2 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn shift_e_ends_every_merged_worktree_whatever_the_selection() {
        let shift_e = KeyEvent::new(KeyCode::Char('E'), KeyModifiers::SHIFT);
        assert_eq!(action(&shift_e), Action::EndMerged);
        let ending = Ok(Some((
            Task::EndMerged,
            "ending the merged worktrees…".to_string(),
            String::new(),
        )));
        let sel = Selection {
            focus: Focus::Queue,
            row: 0,
            worktree: 0,
        };
        assert_eq!(job(&Action::EndMerged, sel, None), ending);
        let v = view(&["bar"], &[("/main", Some("trunk"))]);
        assert_eq!(job(&Action::EndMerged, sel, Some(&v)), ending);
    }

    /// A view of the queue `rows`, a milestone written `#name`, drawn on the lines `lines`.
    fn queue(rows: &[&str], lines: &[usize]) -> View {
        let mut v = view(&[], &[]);
        v.rows = rows
            .iter()
            .map(|r| match r.strip_prefix('#') {
                Some(name) => Row::Milestone(name.to_string()),
                None => Row::Issue((*r).to_string()),
            })
            .collect();
        v.row_lines = lines.to_vec();
        v
    }

    // spec: STU-14, SUA-19 (internal: the keys and the selection of a screen that needs a real terminal)
    #[test]
    fn the_selection_moves_over_the_issues_and_the_milestones() {
        // `One` on line 0 heads `a` and `b`, `Two` on line 4 heads `c`, under an empty line (SUQ-5)
        let v = queue(&["#One", "a", "b", "#Two", "c"], &[0, 1, 2, 4, 5]);
        let mut sel = Selection::default();
        sel.start(&v);
        assert_eq!(sel.line(&v), Some((0, 1)), "starts on the first issue");
        sel.apply(&Action::Down, &v);
        sel.apply(&Action::Down, &v);
        assert_eq!(sel.line(&v), Some((0, 4)), "rests on the heading of Two");
        assert_eq!(
            job(&Action::Start, sel, Some(&v)),
            Err("s and f act on an issue: select one".to_string())
        );
        assert_eq!(
            job(&Action::Merge, sel, Some(&v)),
            Err("s and f act on an issue: select one".to_string())
        );
        // another milestone read before `Two`: `Two` stays selected
        let after = queue(
            &["#Zero", "#One", "a", "b", "#Two", "c"],
            &[0, 2, 3, 4, 6, 7],
        );
        sel.reread(&v, &after);
        assert_eq!(sel.line(&after), Some((0, 6)));
        sel.apply(&Action::Down, &after);
        sel.apply(&Action::Down, &after);
        assert_eq!(sel.line(&after), Some((0, 7)), "kept on the last row");
        // `b` drawn under `a` moves to the root of its section once `a` is closed
        let before = queue(&["#One", "a", "b"], &[0, 1, 2]);
        let closed = queue(&["#One", "c", "b"], &[0, 1, 4]);
        sel.row = 2;
        sel.reread(&before, &closed);
        assert_eq!(sel.line(&closed), Some((0, 4)));
        // `c` cuts the selected milestone, and only a milestone (SUA-19)
        let mut on_two = sel;
        on_two.row = 4;
        assert_eq!(
            job(&Action::Cut, on_two, Some(&after)),
            Ok(Some((
                Task::Cut,
                "starting the cut of Two…".to_string(),
                "Two".to_string()
            )))
        );
        let not_a_milestone = Err("c acts on a milestone: select its heading".to_string());
        on_two.row = 5;
        assert_eq!(job(&Action::Cut, on_two, Some(&after)), not_a_milestone);
        on_two.focus = Focus::Worktrees;
        assert_eq!(job(&Action::Cut, on_two, Some(&after)), not_a_milestone);
        // a first read with no row (a tool that failed): the next one starts on its first issue
        let mut fresh = Selection::default();
        fresh.start(&queue(&[], &[]));
        fresh.reread(&queue(&[], &[]), &v);
        assert_eq!(fresh.line(&v), Some((0, 1)), "not on the heading of One");
        // a queue of milestones only starts on its first heading
        let empty = queue(&["#Two"], &[0]);
        sel.start(&empty);
        assert_eq!(sel.line(&empty), Some((0, 0)));
    }

    // spec: SUE-1 (internal: the keys and the selection of a screen that needs a real terminal)
    #[test]
    fn tab_moves_to_the_worktrees_where_e_ends_the_selected_one() {
        assert_eq!(action(&press(KeyCode::Tab)), Action::Switch);
        assert_eq!(action(&press(KeyCode::Char('e'))), Action::End);
        let v = view(
            &["bar"],
            &[("/main", Some("trunk")), ("/ws", Some("foo")), ("/d", None)],
        );
        let on = |focus, worktree| Selection {
            focus,
            row: 0,
            worktree,
        };
        assert_eq!(
            job(&Action::End, on(Focus::Worktrees, 1), Some(&v)),
            Ok(Some((
                Task::End,
                "ending foo…".to_string(),
                "foo".to_string()
            )))
        );
        assert_eq!(
            job(&Action::Start, on(Focus::Queue, 1), Some(&v)),
            Ok(Some((
                Task::Start,
                "starting bar…".to_string(),
                "bar".to_string()
            )))
        );
        assert!(job(&Action::End, on(Focus::Queue, 1), Some(&v)).is_err_and(|e| e.contains("Tab")));
        assert!(job(&Action::Merge, on(Focus::Worktrees, 1), Some(&v))
            .is_err_and(|e| e.contains("Tab")));
        assert!(job(&Action::End, on(Focus::Worktrees, 2), Some(&v))
            .is_err_and(|e| e.contains("/d: detached")));
        assert_eq!(
            job(&Action::Refresh, on(Focus::Queue, 0), Some(&v)),
            Ok(None)
        );
        let mut sel = on(Focus::Queue, 0);
        sel.apply(&Action::Switch, &v);
        assert_eq!(sel.focus, Focus::Worktrees);
        sel.apply(&Action::Down, &v);
        sel.apply(&Action::Down, &v);
        sel.apply(&Action::Down, &v);
        assert_eq!((sel.row, sel.worktree), (0, 2), "kept in the worktrees");
        sel.apply(&Action::Switch, &v);
        assert_eq!(sel.focus, Focus::Queue);
    }
}
