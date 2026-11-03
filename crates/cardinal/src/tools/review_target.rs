//! The target `skill:issue-workflow` hands to `/code-review`: the diff of a deliverable, or of a whole issue, without
//! the issue files nor the managed copies of `.claude/cardinal.lock`.
//! Usage: `cardinal run review-target [<base>]`: no argument prints the working tree against `HEAD` (RVT-2), `<base>`
//! the range `<base>..HEAD` (RVT-1), each followed by the exclusions of RVT-3 quoted for a shell (RVT-5); exits 3 on an
//! error, printing nothing (RVT-4).

use std::path::Path;
use std::process::Command;

pub fn main(args: &[String]) -> u8 {
    let base = match args {
        [] => None,
        [base] => Some(base.as_str()),
        _ => {
            eprintln!("usage: review-target [<base>]");
            return crate::NOT_A_REFUSAL;
        }
    };
    match target(base) {
        Ok(line) => {
            println!("{line}");
            0
        }
        Err(e) => {
            eprintln!("review-target: {e}");
            crate::NOT_A_REFUSAL
        }
    }
}

/// The line of the target: the revisions, then `--` and one quoted pathspec per word.
fn target(base: Option<&str>) -> Result<String, String> {
    if let Some(option) = base.filter(|b| b.starts_with('-')) {
        // git would read it as one of its options (RVT-4)
        return Err(format!("`{option}` is not a revision"));
    }
    let root = super::repo_root()?;
    let root = Path::new(&root);
    if let Some(base) = base {
        let known = Command::new("git")
            .args(["rev-parse", "--verify", "-q", &format!("{base}^{{commit}}")])
            .current_dir(root)
            .output()
            .is_ok_and(|out| out.status.success());
        if !known {
            return Err(format!("`{base}` is not a revision git resolves"));
        }
    }
    let managed = crate::managed_files(root).map_err(|e| format!("{e:#}"))?;
    // with the issue files off, `issues/` is reviewed as any other directory (FEA-39)
    let issues = (!crate::features::is_off(root, crate::features::ISSUE_FILES)).then_some("issues");
    let mut words = vec![
        base.map_or_else(|| "HEAD".to_string(), |b| shell_word(&format!("{b}..HEAD"))),
        "--".into(),
        quoted(":/"),
    ];
    words.extend(
        issues
            .into_iter()
            .chain(managed.iter().map(String::as_str))
            .map(|path| quoted(&format!(":(top,exclude){path}"))),
    );
    Ok(words.join(" "))
}

/// `word` as it is when a shell reads it unchanged (letters, digits, `._/~^-`), else quoted (RVT-5).
fn shell_word(word: &str) -> String {
    if word
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._/~^-".contains(c))
    {
        word.to_string()
    } else {
        quoted(word)
    }
}

/// `word` in single quotes, a `'` of it written `'\''` (RVT-5).
fn quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', r"'\''"))
}
