//! The help line under the panels (SUA-15): `: commands`, the moves and `q`, `i` on an issue of the queue, then the
//! keys of the tasks that would act on what is selected in the read shown, so the user never reads a key that would
//! only be refused; the keys that act whatever the selection are in the palette (SUP-1), which lists them all. The keys still
//! refuse on their own when pressed out of context (a read may have changed the project since): the line is a hint,
//! never the guard.

use crate::keys::{Action, Focus, Selection};
use crate::screen::View;
use crate::source::{State, Taken};

/// The keys of the help line that always work.
const ALWAYS: &str = ": commands  ↑↓ select  tab panel  q quit";

/// The actions of the keys that act on what is selected, the only ones the help line lists besides `ALWAYS`.
const OF_THE_SELECTION: [Action; 4] = [Action::Summary, Action::Start, Action::Merge, Action::End];

/// The keys of the tasks that would act in a read (SUA-1, SUA-2, SUA-4, SUA-16, SUE-1, SUE-2).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Keys {
    /// For each issue of the queue, in the order of its stems: whether `s` would start it (SUA-1).
    pub start: Vec<bool>,
    /// For each issue of the queue, in the order of its stems: whether `f` would merge it (SUA-2).
    pub merge: Vec<bool>,
    /// For each worktree, in the order of its panel: whether its row reads `merged`, so `e` would end it (SUE-1).
    pub merged: Vec<bool>,
    /// Whether `v` would start the release: in zellij, a milestone awaits its release (SUA-4).
    pub release: bool,
    /// Whether `p` would start the review of the priorities: in zellij, the queue lists an open issue (SUA-16).
    pub prioritize: bool,
}

impl Keys {
    /// The keys of `state` for the issues `stems` of its queue, in a zellij session when `zellij` (SUA-15).
    pub fn of(state: &State, stems: &[String], zellij: bool) -> Self {
        let issues = state.queue.as_ref().map_or(&[][..], |q| &q.issues[..]);
        let issue = |stem: &String| issues.iter().find(|i| &i.stem == stem);
        let start = |stem| {
            issue(stem).is_some_and(|i| {
                let free = match &i.taken {
                    None => true,
                    Some(Taken::Claim { state, alive, .. }) => state == "working" && !alive,
                    Some(Taken::Branch { .. }) => false,
                };
                zellij && i.state == "ready" && free
            })
        };
        let merge = |stem| {
            issue(stem).is_some_and(|i| {
                matches!(&i.taken, Some(Taken::Claim { state, .. }) if state == "merge pending")
            })
        };
        Self {
            start: stems.iter().map(start).collect(),
            merge: stems.iter().map(merge).collect(),
            merged: state
                .worktrees
                .as_ref()
                .map_or_else(|_| Vec::new(), |ws| ws.iter().map(|w| w.merged).collect()),
            release: zellij
                && state
                    .milestones
                    .as_ref()
                    .is_ok_and(|m| m.list.iter().any(|m| m.awaits_release)),
            prioritize: zellij && !issues.is_empty(),
        }
    }
}

/// The commands that would act in `v` with `sel` selected, each its label (its key, a space and its name) and its
/// action, in the order of SUA-15: `i summary` with the selection on an issue of the queue (SUQ-6), then the keys of
/// the tasks that would act, none while one runs (`busy`, SUA-6); the help line and the palette (SUP-1) read them.
pub(crate) fn offered(v: &View, sel: Selection, busy: bool) -> Vec<(&'static str, Action)> {
    let at = |flags: &[bool], i: usize| flags.get(i).copied().unwrap_or(false);
    let keys = &v.keys;
    let queue = sel.focus == Focus::Queue;
    let summary = crate::summary::selected(v, sel).is_some();
    let tasks = [
        (
            queue && at(&keys.start, sel.issue),
            "s start",
            Action::Start,
        ),
        (
            queue && at(&keys.merge, sel.issue),
            "f merge",
            Action::Merge,
        ),
        (
            !queue && at(&keys.merged, sel.worktree),
            "e end",
            Action::End,
        ),
        (
            keys.merged.contains(&true),
            "E end merged",
            Action::EndMerged,
        ),
        (keys.release, "v release", Action::Release),
        (keys.prioritize, "p prioritize", Action::Prioritize),
    ];
    std::iter::once((summary, "i summary", Action::Summary))
        .chain(
            tasks
                .into_iter()
                .map(|(on, label, a)| (on && !busy, label, a)),
        )
        .filter(|(on, _, _)| *on)
        .map(|(_, label, a)| (label, a))
        .collect()
}

/// The help line of the panels of `v` with `sel` selected, a task running when `busy` (SUA-15).
pub(crate) fn line(v: &View, sel: Selection, busy: bool) -> String {
    std::iter::once(ALWAYS)
        .chain(
            offered(v, sel, busy)
                .into_iter()
                .filter(|(_, a)| OF_THE_SELECTION.contains(a))
                .map(|(label, _)| label),
        )
        .collect::<Vec<_>>()
        .join("  ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::tests::view;

    fn on(focus: Focus, issue: usize, worktree: usize) -> Selection {
        Selection {
            focus,
            issue,
            worktree,
        }
    }

    // spec: SUA-15 (internal: the selection moves only in a screen that needs a real terminal)
    #[test]
    fn the_line_follows_the_selection_across_the_panels() {
        let mut v = view(
            &["a", "b"],
            &[
                ("/main", Some("trunk")),
                ("/ws", Some("b")),
                ("/x", Some("x")),
            ],
        );
        v.keys = Keys {
            start: vec![true, false],
            merge: vec![false, true],
            merged: vec![false, true, false],
            release: false,
            prioritize: false,
        };
        let always = ": commands  ↑↓ select  tab panel  q quit";
        assert_eq!(
            line(&v, on(Focus::Queue, 0, 0), false),
            format!("{always}  i summary  s start")
        );
        assert_eq!(
            line(&v, on(Focus::Queue, 1, 0), false),
            format!("{always}  i summary  f merge")
        );
        assert_eq!(
            line(&v, on(Focus::Worktrees, 0, 1), false),
            format!("{always}  e end")
        );
        assert_eq!(line(&v, on(Focus::Worktrees, 0, 2), false), always);
        v.keys.release = true;
        v.keys.merged = vec![false; 3];
        assert_eq!(line(&v, on(Focus::Worktrees, 0, 0), false), always);
        v.keys.prioritize = true;
        assert_eq!(line(&v, on(Focus::Worktrees, 0, 0), false), always);
    }

    // spec: SUA-H7 (internal: `Tab` and the arrows move the selection only in a screen that needs a real terminal)
    #[test]
    fn tab_to_the_merged_worktree_offers_its_end() {
        use crate::keys::Action;
        let mut v = view(&["foo"], &[("/main", Some("trunk")), ("/bar", Some("bar"))]);
        v.keys = Keys {
            start: vec![false],
            merge: vec![true],
            merged: vec![false, true],
            release: false,
            prioritize: false,
        };
        let mut sel = Selection::default();
        assert_eq!(
            line(&v, sel, false),
            ": commands  ↑↓ select  tab panel  q quit  i summary  f merge"
        );
        sel.apply(&Action::Switch, &v);
        sel.apply(&Action::Down, &v);
        assert_eq!(
            line(&v, sel, false),
            ": commands  ↑↓ select  tab panel  q quit  e end"
        );
    }

    // spec: SUA-15 (internal: a task runs only in a screen that needs a real terminal)
    #[test]
    fn no_key_of_a_task_while_one_runs() {
        let mut v = view(&["foo"], &[("/main", Some("trunk")), ("/bar", Some("bar"))]);
        v.keys = Keys {
            start: vec![true],
            merge: vec![true],
            merged: vec![false, true],
            release: true,
            prioritize: true,
        };
        assert_eq!(
            line(&v, Selection::default(), true),
            ": commands  ↑↓ select  tab panel  q quit  i summary"
        );
    }

    // spec: SUA-15 (internal: an empty or failed panel, which a read past the selection leaves)
    #[test]
    fn an_empty_read_offers_only_the_keys_that_always_work() {
        let v = view(&[], &[]);
        let always = ": commands  ↑↓ select  tab panel  q quit";
        assert_eq!(line(&v, on(Focus::Queue, 3, 0), false), always);
        assert_eq!(line(&v, on(Focus::Worktrees, 0, 5), false), always);
    }

    // spec: SUA-15 (internal: the keys that act whatever the selection are in the palette, which `--print` never shows)
    #[test]
    fn the_keys_of_the_whole_project_follow_the_read_and_zellij() {
        use crate::source::{Milestone, Milestones, Queue, State, Worktree};
        let worktree = |merged| Worktree {
            path: "/w".into(),
            branch: Some("w".into()),
            session: None,
            merged,
            locked: false,
        };
        let milestone = || Milestone {
            name: "New".into(),
            state: Some("done".into()),
            on_hold: false,
            open: 0,
            listed: 1,
            awaits_release: true,
            proposed: None,
        };
        let state = |issues: Vec<crate::source::Issue>| State {
            milestones: Ok(Milestones {
                list: vec![milestone()],
                release_error: None,
            }),
            queue: Ok(Queue {
                next: None,
                issues,
                problems: Vec::new(),
            }),
            worktrees: Ok(vec![worktree(false), worktree(true)]),
            measure: Vec::new(),
            measuring: false,
            measure_error: None,
            charts: crate::charts::charts(None),
        };
        let issue = crate::source::Issue {
            stem: "a".into(),
            title: None,
            goal: None,
            priority: None,
            state: "ready".into(),
            milestone: None,
            depends_on: Vec::new(),
            taken: None,
            tab: false,
        };
        let stems = ["a".to_string()];
        let k = Keys::of(&state(vec![issue]), &stems, true);
        assert_eq!(
            (k.release, k.prioritize, k.merged),
            (true, true, vec![false, true])
        );
        assert_eq!(k.start, [true]);
        let k = Keys::of(&state(Vec::new()), &[], true);
        assert_eq!(
            (k.release, k.prioritize),
            (true, false),
            "no open issue to prioritize"
        );
        let k = Keys::of(&state(Vec::new()), &[], false);
        assert_eq!((k.release, k.prioritize), (false, false), "outside zellij");
    }
}
