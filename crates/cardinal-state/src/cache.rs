//! The cache directory of `cardinal-ui` (STU-18), which `cardinal tool status-line` writes the usage limits into
//! (STL-3).

use std::path::PathBuf;

/// `<cache>/cardinal-ui`, `<cache>` the value of `XDG_CACHE_HOME` when it is an absolute path, else `$HOME/.cache`
/// (STU-18); `None` without either.
#[must_use]
pub fn ui_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("cardinal-ui"))
}

/// The usage file of the limits of the Claude plan, one for every project (STL-3).
#[must_use]
pub fn usage_file() -> Option<PathBuf> {
    ui_dir().map(|d| d.join("usage.json"))
}
