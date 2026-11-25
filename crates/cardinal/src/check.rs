//! `check`: the drift of the project from its lock, the review gate and the hooks the installed `cardinal` refuses,
//! and the warnings on the settings and the retention of the transcripts.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::lock::{fingerprint_of, install_hint, read_lock};
use crate::{
    entries, features, format_hint, gate, is_git_url, launcher, settings, source_and_rev, LOCK,
};

/// What `check` found: `problems` make it fail, `warnings` do not.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Checked {
    /// Managed files modified or deleted since the last sync, and the native review gate when the project should have
    /// it and does not, and the hook commands the launcher on the path refuses (BIN-15).
    pub problems: Vec<String>,
    /// What the project's settings lack or run in vain (SYN-24 to SYN-26).
    pub warnings: Vec<String>,
}

/// Managed files modified or deleted since the last sync, one line each, and the native review gate when the project
/// should have it and does not. Under CI (the `CI` environment variable is set) the gate is not looked at: a fresh
/// clone has no git hooks, and nobody commits there. It also warns of what the project's settings lack: the deny rules of
/// the review waiver (when the review hook is among the managed files), the hooks they run, the attribution turned off.
/// A hook command whose subcommand the `cardinal` on the path refuses is a problem too
/// (BIN-15).
///
/// # Errors
/// When the project was never synced (no lock file) or the lock cannot be read.
pub fn check(root: &Path) -> Result<Checked> {
    check_with(root, in_ci())
}

/// Whether `CI` says so: set, and not empty, `0` or `false`.
pub(crate) fn in_ci() -> bool {
    std::env::var("CI")
        .is_ok_and(|v| !matches!(v.trim().to_lowercase().as_str(), "" | "0" | "false"))
}

/// The warnings on the retention of the transcripts (SYN-28), printed by `check`, `sync` and `update`; none under CI,
/// and none on the user settings when the project turns off the measure of the harness that reads them (FEA-28): its
/// own settings still sweep the transcripts of every project. Kept out of `check` and `sync`, which read the project
/// only: these read the user settings too.
#[must_use]
pub fn retention_warnings(root: &Path) -> Vec<String> {
    if in_ci() {
        return Vec::new();
    }
    if features::is_off(root, features::HARNESS_MEASURE) {
        return settings::project_retention(root);
    }
    let user = config_dir().map(|dir| dir.join("settings.json"));
    settings::retention(root, user.as_deref())
}

/// The configuration directory of Claude Code: `CLAUDE_CONFIG_DIR`, or else `~/.claude`; `None` when neither is set.
pub(crate) fn config_dir() -> Option<PathBuf> {
    config_dir_of(
        std::env::var_os("CLAUDE_CONFIG_DIR"),
        std::env::var_os("HOME"),
    )
}

fn config_dir_of(config_dir: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let set = |var: Option<OsString>| var.filter(|v| !v.is_empty()).map(PathBuf::from);
    set(config_dir).or_else(|| set(home).map(|h| h.join(".claude")))
}

fn check_with(root: &Path, ci: bool) -> Result<Checked> {
    let Some(lock) = read_lock(root)? else {
        bail!("no {LOCK}: run `cardinal sync` first");
    };
    let gated = gate::manages(&lock.entries, lock.files.keys());
    let text = fs::read_to_string(root.join(entries::SETTINGS)).ok();
    let hooks = text
        .as_deref()
        .and_then(|text| serde_json::from_str(text).ok())
        .map_or_default(|settings| launcher::refused(&settings));
    let install = install_hint(root);
    let problems = lock
        .files
        .iter()
        .filter_map(|(rel, locked)| match fingerprint_of(&root.join(rel)) {
            None => Some(format!("missing: {rel}")),
            Some(print) if print != *locked => Some(format!("modified: {rel}")),
            Some(_) => None,
        })
        .chain(entries::missing(text.as_deref(), &lock.entries))
        .chain(gate::problem(root, !ci && gated))
        .chain(hooks.iter().map(|command| {
            format!(
                "refused: the installed cardinal cannot run the hook `{command}` of {}: install {install} first",
                entries::SETTINGS
            )
        }))
        .collect();
    let attribution = !features::is_off(root, features::NO_AI_ATTRIBUTION);
    let mut warnings =
        settings::warnings(root, gated, &|path| root.join(path).is_file(), attribution);
    if source_and_rev(root).is_ok_and(|(source, _)| is_git_url(&source)) {
        warnings.extend(format_hint(lock.format));
    }
    Ok(Checked { problems, warnings })
}
