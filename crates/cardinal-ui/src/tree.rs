//! The sections of the queue (SUQ-1): the open issues grouped by milestone, each section a tree of their
//! `depends_on`, with the open dependencies an issue is not drawn under (SUQ-2) and the tree drawn before its stem
//! (SUQ-3).

use crate::source::{Issue, Milestone};

/// What heads a section (SUQ-1).
#[derive(Debug, PartialEq, Eq)]
pub enum Heading {
    /// A milestone of `roadmap`, by its index in its list.
    Listed(usize),
    /// A milestone an issue names that `roadmap` does not list, or every milestone when `roadmap` was not read.
    Named(String),
    /// The issues of no milestone.
    None,
}

/// An issue row of a section: the index of its issue in the queue, the tree drawn before its stem and the open
/// dependencies it is not drawn under.
#[derive(Debug, PartialEq, Eq)]
pub struct Row {
    pub issue: usize,
    pub tree: String,
    pub also_after: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Section {
    pub heading: Heading,
    pub rows: Vec<Row>,
}

/// The sections of `issues` (in the order of `next-issue`) under `milestones`, `None` when `roadmap` was not read.
pub fn sections(milestones: Option<&[Milestone]>, issues: &[Issue]) -> Vec<Section> {
    let mut headings: Vec<(Heading, Option<&str>)> = milestones
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(n, m)| (Heading::Listed(n), Some(m.name.as_str())))
        .collect();
    for i in issues {
        let Some(name) = i.milestone.as_deref() else {
            continue;
        };
        if !headings.iter().any(|(_, m)| *m == Some(name)) {
            headings.push((Heading::Named(name.to_string()), Some(name)));
        }
    }
    if issues.iter().any(|i| i.milestone.is_none()) {
        headings.push((Heading::None, None));
    }
    // a milestone named twice in `roadmap` holds its issues in its first section only
    let mut taken: Vec<Option<&str>> = Vec::new();
    headings
        .into_iter()
        .map(|(heading, name)| {
            let members: Vec<usize> = if taken.contains(&name) {
                Vec::new()
            } else {
                taken.push(name);
                (0..issues.len())
                    .filter(|&n| issues[n].milestone.as_deref() == name)
                    .collect()
            };
            Section {
                heading,
                rows: tree(issues, &members),
            }
        })
        .collect()
}

/// The rows of the issues `members` of one section, in the order of a walk of their tree.
fn tree(issues: &[Issue], members: &[usize]) -> Vec<Row> {
    let index = |stem: &str| members.iter().position(|&n| issues[n].stem == stem);
    // the first dependency of each member in the section (as a position in `members`), before the cycles are cut
    let first: Vec<Option<usize>> = members
        .iter()
        .map(|&n| issues[n].depends_on.iter().find_map(|d| index(d)))
        .collect();
    // a member is at the root when its chain of first dependencies leads back to it; the chains are followed once,
    // each member settled when its chain ends or meets a settled one
    let mut cut = vec![None::<bool>; members.len()];
    for m in 0..members.len() {
        let mut chain = vec![m];
        let mut at = first[m];
        // where the loop the chain ends in starts: its members lead back to themselves, those before it do not
        let ends = loop {
            let Some(p) = at.filter(|&p| cut[p].is_none()) else {
                break None;
            };
            if let Some(k) = chain.iter().position(|&c| c == p) {
                break Some(k);
            }
            chain.push(p);
            at = first[p];
        };
        for (k, &c) in chain.iter().enumerate() {
            if cut[c].is_none() {
                cut[c] = Some(ends.is_some_and(|start| k >= start));
            }
        }
    }
    let parent: Vec<Option<usize>> = (0..members.len())
        .map(|m| first[m].filter(|_| cut[m] != Some(true)))
        .collect();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); members.len() + 1];
    for (m, p) in parent.iter().enumerate() {
        children[p.unwrap_or(members.len())].push(m);
    }
    let mut rows = Vec::new();
    // the levels being walked: the members of each one and the next to draw
    let mut stack: Vec<(&[usize], usize)> = vec![(children[members.len()].as_slice(), 0)];
    while let Some((level, at)) = stack.last_mut() {
        let Some(&m) = level.get(*at) else {
            stack.pop();
            continue;
        };
        *at += 1;
        let depth = stack.len();
        let mut tree = String::new();
        for (k, (level, at)) in stack.iter().enumerate().skip(1) {
            let more = *at < level.len();
            tree.push_str(match (k + 1 == depth, more) {
                (true, true) => "├─ ",
                (true, false) => "└─ ",
                (false, true) => "│  ",
                (false, false) => "   ",
            });
        }
        let n = members[m];
        let drawn_under = parent[m].map(|p| issues[members[p]].stem.as_str());
        let mut also_after: Vec<String> = Vec::new();
        for d in &issues[n].depends_on {
            let open = issues.iter().any(|i| &i.stem == d);
            if open
                && *d != issues[n].stem
                && Some(d.as_str()) != drawn_under
                && !also_after.contains(d)
            {
                also_after.push(d.clone());
            }
        }
        rows.push(Row {
            issue: n,
            tree,
            also_after,
        });
        stack.push((children[m].as_slice(), 0));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(stem: &str, milestone: Option<&str>, depends_on: &[&str]) -> Issue {
        Issue {
            stem: stem.to_string(),
            title: None,
            goal: None,
            priority: Some("low".to_string()),
            state: "ready".to_string(),
            milestone: milestone.map(str::to_string),
            depends_on: depends_on.iter().map(|d| (*d).to_string()).collect(),
            taken: None,
            tab: false,
        }
    }

    fn milestone(name: &str) -> Milestone {
        Milestone {
            name: name.to_string(),
            state: Some("in issues".to_string()),
            on_hold: false,
            open: 0,
            listed: 0,
            awaits_release: false,
            proposed: None,
        }
    }

    /// The rows of `section` as `<tree><stem>` and their `also after`.
    fn drawn<'a>(issues: &'a [Issue], section: &'a Section) -> Vec<(String, Vec<&'a str>)> {
        section
            .rows
            .iter()
            .map(|r| {
                (
                    format!("{}{}", r.tree, issues[r.issue].stem),
                    r.also_after.iter().map(String::as_str).collect(),
                )
            })
            .collect()
    }

    // spec: SUQ-1, SUQ-2 (internal: the cases of the tree, on its data rather than a repository each)
    #[test]
    fn an_issue_is_drawn_under_its_first_open_dependency_of_its_section() {
        let one = Some("One");
        let issues = [
            issue("x", Some("Two"), &["a"]),
            issue("a", one, &["gone"]),
            issue("p", one, &["q"]),
            issue("q", one, &["p"]),
            issue("ghost", Some("Ghost"), &[]),
        ];
        let ms = [milestone("One"), milestone("Two"), milestone("Done")];
        let s = sections(Some(&ms), &issues);
        let headings: Vec<&Heading> = s.iter().map(|s| &s.heading).collect();
        assert_eq!(
            headings,
            [
                &Heading::Listed(0),
                &Heading::Listed(1),
                &Heading::Listed(2),
                &Heading::Named("Ghost".to_string()),
            ],
            "the sections of roadmap, then the others; no `No milestone` when every issue has one"
        );
        assert_eq!(
            drawn(&issues, &s[0]),
            [
                ("a".to_string(), vec![]),
                ("p".to_string(), vec!["q"]),
                ("q".to_string(), vec!["p"]),
            ],
            "a closed dependency or a cycle leaves an issue at the root"
        );
        assert_eq!(drawn(&issues, &s[1]), [("x".to_string(), vec!["a"])]);
        assert!(s[2].rows.is_empty(), "a milestone with no open issue");
    }

    // spec: SUQ-1 (internal: the cases of the tree, on its data rather than a repository each)
    #[test]
    fn without_roadmap_each_milestone_an_issue_names_heads_its_section() {
        let issues = [
            issue("b", Some("Two"), &[]),
            issue("a", None, &[]),
            issue("c", Some("One"), &[]),
        ];
        let s = sections(None, &issues);
        let headings: Vec<&Heading> = s.iter().map(|s| &s.heading).collect();
        assert_eq!(
            headings,
            [
                &Heading::Named("Two".to_string()),
                &Heading::Named("One".to_string()),
                &Heading::None
            ]
        );
    }

    // spec: SUQ-3 (internal: the cases of the tree, on its data rather than a repository each)
    #[test]
    fn the_tree_draws_the_levels_above_an_issue() {
        let one = Some("One");
        let issues = [
            issue("a", one, &[]),
            issue("b", one, &["a"]),
            issue("c", one, &["a"]),
            issue("e", one, &["b"]),
        ];
        let s = sections(Some(&[milestone("One")]), &issues);
        let rows: Vec<String> = drawn(&issues, &s[0]).into_iter().map(|r| r.0).collect();
        assert_eq!(rows, ["a", "├─ b", "│  └─ e", "└─ c"]);
        let issues = [
            issue("a", one, &[]),
            issue("b", one, &["a"]),
            issue("e", one, &["b"]),
        ];
        let s = sections(Some(&[milestone("One")]), &issues);
        let rows: Vec<String> = drawn(&issues, &s[0]).into_iter().map(|r| r.0).collect();
        assert_eq!(rows, ["a", "└─ b", "   └─ e"]);
    }

    // spec: SUQ-1 (internal: the cases of the tree, on its data rather than a repository each)
    #[test]
    fn a_long_chain_of_dependencies_is_drawn_at_once() {
        let stems: Vec<String> = (0..2000).map(|n| format!("i{n}")).collect();
        let issues: Vec<Issue> = (0..2000)
            .map(|n| {
                let before: Vec<&str> = stems[..n]
                    .iter()
                    .rev()
                    .take(1)
                    .map(String::as_str)
                    .collect();
                issue(&stems[n], Some("One"), &before)
            })
            .collect();
        let s = sections(Some(&[milestone("One")]), &issues);
        assert_eq!(s[0].rows.len(), 2000);
        assert_eq!(s[0].rows[1999].tree.chars().count(), 3 * 1999);
    }

    // spec: SUQ-1, SUQ-2 (internal: the cases of the tree, on its data rather than a repository each)
    #[test]
    fn an_issue_is_drawn_once_and_never_after_itself() {
        let issues = [
            issue("a", Some("One"), &["a", "x", "x"]),
            issue("x", Some("Two"), &[]),
        ];
        let s = sections(Some(&[milestone("One"), milestone("One")]), &issues);
        assert_eq!(drawn(&issues, &s[0]), [("a".to_string(), vec!["x"])]);
        assert!(
            s[1].rows.is_empty(),
            "a milestone named twice holds its issues once"
        );
    }
}
