//! The worktrees of `git worktree list --porcelain` and its main one, read alike by `cardinal` and `cardinal-ui`
//! (ADR 0033): each binary runs git through its own runner and hands the output here.

use std::path::PathBuf;

/// A worktree of `git worktree list`.
#[derive(Clone, Debug, PartialEq)]
pub struct Worktree {
    pub path: PathBuf,
    /// The branch it has checked out, `None` when detached.
    pub branch: Option<String>,
    pub head: Option<String>,
    /// Locked by `git worktree lock`: git refuses to remove or prune it.
    pub locked: bool,
    /// The repository is bare: this record stands for it, not for a work tree.
    pub bare: bool,
}

/// Every worktree of the output of `git worktree list --porcelain`, the main one first.
#[must_use]
pub fn parse(porcelain: &str) -> Vec<Worktree> {
    porcelain
        .split("\n\n")
        .filter_map(|record| {
            let field = |key: &str| record.lines().find_map(|l| l.strip_prefix(key));
            Some(Worktree {
                path: PathBuf::from(field("worktree ")?),
                branch: field("branch refs/heads/").map(str::to_string),
                head: field("HEAD ").map(str::to_string),
                locked: record
                    .lines()
                    .any(|l| l == "locked" || l.starts_with("locked ")),
                bare: record.lines().any(|l| l == "bare"),
            })
        })
        .collect()
}

/// The main worktree of `list` and the branch it has checked out (`None` when detached); why there is none (RBI-4,
/// SUA-11).
///
/// # Errors
/// `no main worktree: the repository is bare`, or `git worktree list: no main worktree` for an empty list.
pub fn main(list: &[Worktree]) -> Result<(PathBuf, Option<String>), String> {
    let main = list.first().ok_or("git worktree list: no main worktree")?;
    if main.bare {
        return Err("no main worktree: the repository is bare".to_string());
    }
    Ok((main.path.clone(), main.branch.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "worktree /r\nHEAD aaa\nbranch refs/heads/main\n\n\
        worktree /r-x\nHEAD bbb\ndetached\nlocked reason\n\n\
        worktree /r-y\nHEAD ccc\nbranch refs/heads/y\nlocked\n";

    #[test]
    fn each_record_is_a_worktree_the_main_one_first() {
        let all = parse(LIST);
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].path, PathBuf::from("/r"));
        assert_eq!(all[0].branch.as_deref(), Some("main"));
        assert_eq!(all[0].head.as_deref(), Some("aaa"));
        assert!(!all[0].locked && !all[0].bare);
        assert_eq!(all[1].branch, None, "detached");
        assert!(
            all[1].locked && all[2].locked,
            "locked with or without a reason"
        );
        assert_eq!(parse(""), Vec::new());
    }

    #[test]
    fn the_main_worktree_is_the_first_unless_the_repository_is_bare() {
        assert_eq!(
            main(&parse(LIST)),
            Ok((PathBuf::from("/r"), Some("main".into())))
        );
        let bare = parse("worktree /r.git\nbare\n\nworktree /w\nHEAD a\nbranch refs/heads/w\n");
        assert!(bare[0].bare);
        assert_eq!(
            main(&bare),
            Err("no main worktree: the repository is bare".into())
        );
        assert_eq!(main(&[]), Err("git worktree list: no main worktree".into()));
    }
}
