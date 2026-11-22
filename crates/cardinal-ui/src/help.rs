//! The help line under the panels (SUA-15): the keys that always work, then the keys of the tasks that would act on
//! what is selected in the read shown, so the user never reads a key that would only be refused. The keys still
//! refuse on their own when pressed out of context (a read may have changed the project since): the line is a hint,
//! never the guard.

use crate::keys::{Focus, Selection};
use crate::screen::View;
use crate::source::{State, Taken};

/// The keys of the help line that always work.
const ALWAYS: &str = "q quit  r refresh  ↑↓ select  tab panel";

/// The last key of the help line, which always works.
const LAST: &str = "g charts";

/// The keys of the tasks that would act in a read (SUA-1, SUA-2, SUA-4, SUE-1, SUE-2).
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
        }
    }
}

/// The help line of the panels of `v` with `sel` selected (SUA-15).
pub(crate) fn line(v: &View, sel: Selection) -> String {
    let at = |flags: &[bool], i: usize| flags.get(i).copied().unwrap_or(false);
    let keys = &v.keys;
    let queue = sel.focus == Focus::Queue;
    let offered = [
        (queue && at(&keys.start, sel.issue), "s start"),
        (queue && at(&keys.merge, sel.issue), "f merge"),
        (!queue && at(&keys.merged, sel.worktree), "e end"),
        (keys.merged.contains(&true), "E end merged"),
        (keys.release, "v release"),
    ];
    std::iter::once(ALWAYS)
        .chain(offered.iter().filter(|(on, _)| *on).map(|(_, key)| *key))
        .chain(std::iter::once(LAST))
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
        };
        let always = "q quit  r refresh  ↑↓ select  tab panel";
        assert_eq!(
            line(&v, on(Focus::Queue, 0, 0)),
            format!("{always}  s start  E end merged  g charts")
        );
        assert_eq!(
            line(&v, on(Focus::Queue, 1, 0)),
            format!("{always}  f merge  E end merged  g charts")
        );
        assert_eq!(
            line(&v, on(Focus::Worktrees, 0, 1)),
            format!("{always}  e end  E end merged  g charts")
        );
        assert_eq!(
            line(&v, on(Focus::Worktrees, 0, 2)),
            format!("{always}  E end merged  g charts")
        );
        v.keys.release = true;
        v.keys.merged = vec![false; 3];
        assert_eq!(
            line(&v, on(Focus::Worktrees, 0, 0)),
            format!("{always}  v release  g charts")
        );
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
        };
        let mut sel = Selection::default();
        assert_eq!(
            line(&v, sel),
            "q quit  r refresh  ↑↓ select  tab panel  f merge  E end merged  g charts"
        );
        sel.apply(&Action::Switch, &v);
        sel.apply(&Action::Down, &v);
        assert_eq!(
            line(&v, sel),
            "q quit  r refresh  ↑↓ select  tab panel  e end  E end merged  g charts"
        );
    }

    // spec: SUA-15 (internal: an empty or failed panel, which a read past the selection leaves)
    #[test]
    fn an_empty_read_offers_only_the_keys_that_always_work() {
        let v = view(&[], &[]);
        let always = "q quit  r refresh  ↑↓ select  tab panel  g charts";
        assert_eq!(line(&v, on(Focus::Queue, 3, 0)), always);
        assert_eq!(line(&v, on(Focus::Worktrees, 0, 5)), always);
    }
}
