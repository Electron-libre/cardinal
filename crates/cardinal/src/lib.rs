//! Copy the managed files of a Cardinal source into a project, and detect drift.
//!
//! A source holds `files/` (managed: overwritten on every sync while untouched locally) and `seed/`
//! (project-owned: copied once, never again), and may hold `entries/.claude/settings.json`, the entries of the
//! project's settings it manages (`entries.rs`). `.claude/cardinal.lock` records the hash of each managed file as
//! written and the managed entries, so a later sync tells a local edit from a source change.
//!
//! This root holds `init`, `sync` and `update`; `source.rs` fetches a git source, `config.rs` reads and edits the
//! configuration, `lock.rs` reads and writes the lock, `check.rs` compares the project with it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// Project configuration, relative to the project root.
pub const CONFIG: &str = ".claude/cardinal.toml";
/// The personal configuration of the person at the keyboard, relative to the project root: git ignores it and Cardinal
/// never manages it; its `[project]` values override those of `CONFIG` for the personal keys (ADR 0018).
pub const PERSONAL_CONFIG: &str = ".claude/cardinal.local.toml";
/// Hashes of the managed files as last written, relative to the project root.
pub const LOCK: &str = ".claude/cardinal.lock";
/// The table of the configuration that names the source and its pinned revision.
pub(crate) const TABLE: &str = "cardinal";
/// The lowest format this `cardinal` syncs: the first whose issues are ordered by the rank of each file alone (SYN-51,
/// ADR 0020).
const MIN_FORMAT: u32 = 5;
/// The exit code of a failure that is not the verdict of a hook: `run` that cannot get the tool or does not know it,
/// the launcher that cannot parse its arguments, an error of the other subcommands. Not 2, which Claude Code reads as
/// the refusal of a hook (BIN-7, BIN-14, BIN-17, SYN-37).
pub const NOT_A_REFUSAL: u8 = 3;
/// The format of the managed content this `cardinal` reads, and every lower one down to `MIN_FORMAT` (SYN-40): equal
/// to the `cardinal-format` of its revision (SYN-43).
pub const FORMAT: u32 = 11;
/// The file of the source that declares the format its content needs, relative to the source.
const SOURCE_FORMAT: &str = "cardinal-format";

mod budget;
mod check;
mod config;
mod entries;
mod features;
mod gate;
mod git;
mod launcher;
mod lock;
mod markdown;
mod memory;
mod migration;
mod project_table;
pub mod run;
mod settings;
mod source;
pub mod tools;

pub use budget::warnings as budget_warnings;
pub use check::{check, retention_warnings, Checked};
pub use source::upgrade;

pub(crate) use check::{config_dir, in_ci};
pub(crate) use config::{read_config, readable, set_rev, source_and_rev};
pub(crate) use lock::{hash, is_executable, managed_files};
pub(crate) use source::{
    cache_dir, checkout, clone_dir, fetch, git_output, is_full_hash, is_git_url, is_scp_like,
    resolve_in, resolved,
};

use config::format_of;
use lock::{fingerprint, fingerprint_of, is_inside, read_lock, write_lock, Lock};
use source::{cache_or_default, git};

/// The configuration of the project at `root`.
#[must_use]
pub fn config_of(root: &Path) -> PathBuf {
    root.join(CONFIG)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SyncOptions {
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Added,
    Updated,
    Removed,
    Seeded,
    Unchanged,
    /// Managed file modified locally: left as it is.
    Conflict,
    /// File dropped from the source but modified locally: left as it is, no longer managed.
    Kept,
    /// A hook of the project's own moved aside so that the managed one can chain it.
    Moved,
    /// A project file whose seeded text a migration rewrote (SYN-53).
    Rewritten,
    /// Not done, for the reason given.
    Skipped(&'static str),
}

impl Outcome {
    /// Whether the project file was written: a conflict, a kept file and a skip leave it as it was.
    fn wrote(&self) -> bool {
        matches!(
            self,
            Outcome::Added
                | Outcome::Updated
                | Outcome::Removed
                | Outcome::Seeded
                | Outcome::Moved
                | Outcome::Rewritten
        )
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Outcome::Added => "added",
            Outcome::Updated => "updated",
            Outcome::Removed => "removed",
            Outcome::Seeded => "seeded",
            Outcome::Unchanged => "unchanged",
            Outcome::Conflict => "conflict (modified locally, fix it in the source or use --force)",
            Outcome::Kept => "kept (dropped from the source but modified locally)",
            Outcome::Moved => "moved (your hook now runs after the review gate)",
            Outcome::Rewritten => "rewritten",
            Outcome::Skipped(why) => return write!(f, "skipped ({why})"),
        })
    }
}

#[derive(Debug, Default)]
pub struct Report {
    pub entries: Vec<(String, Outcome)>,
    /// What the project's settings lack or run in vain once synced (SYN-27): printed, never a failure.
    pub warnings: Vec<String>,
    /// The format the lock records once synced (as it is under `--dry-run`), `None` without a lock.
    lock_format: Option<u32>,
}

impl Report {
    #[must_use]
    pub fn conflicts(&self) -> usize {
        self.entries
            .iter()
            .filter(|(_, o)| *o == Outcome::Conflict)
            .count()
    }

    #[must_use]
    pub fn outcome(&self, path: &str) -> Option<Outcome> {
        self.entries
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, o)| *o)
    }
}

/// Relative paths (with `/`) of every file under `dir`, sorted. A missing `dir` is empty.
fn list(dir: &Path) -> Result<Vec<String>> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                walk(base, &path, out)?;
            } else if let Ok(rel) = path.strip_prefix(base) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    if dir.is_dir() {
        walk(dir, dir, &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))
}

/// `write`, then make the file executable or not as in the source.
fn write_as(path: &Path, bytes: &[u8], executable: bool) -> Result<()> {
    write(path, bytes)?;
    set_executable(path, executable)
}

#[cfg(unix)]
fn set_executable(path: &Path, executable: bool) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    let mode = permissions.mode();
    // Only the execute bits change, and only where the read bits are: the usual `chmod +x` / `-x`.
    let mode = if executable {
        mode | ((mode & 0o444) >> 2)
    } else {
        mode & !0o111
    };
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions).with_context(|| format!("chmod {}", path.display()))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path, _executable: bool) -> Result<()> {
    Ok(())
}

/// Create the configuration from the source and the project-owned defaults of the source (SYN-9).
///
/// # Errors
/// When the project already has a configuration or the source cannot be read.
pub fn init(root: &Path, source: &str, rev: Option<&str>) -> Result<()> {
    let path = root.join(CONFIG);
    if path.exists() {
        bail!("{} already exists", path.display());
    }
    let (dir, install) = if is_git_url(source) {
        let url = resolved(root, source);
        let dir = checkout(None, &url, rev)?;
        let commit = git_output(&dir, &["rev-parse", "HEAD"])?;
        let install = launcher::install(&url, Some(&commit));
        (dir, install)
    } else {
        (root.join(source), launcher::install(source, None))
    };
    readable(&dir, source, &install)?;
    let mut table = toml::Table::new();
    table.insert("source".into(), source.into());
    if let Some(rev) = rev {
        table.insert("rev".into(), rev.into());
    }
    let mut config = toml::Table::new();
    config.insert(TABLE.into(), table.into());
    let mut text = config.to_string();
    if let Ok(defaults) = fs::read_to_string(dir.join("seed").join(CONFIG)) {
        text.push('\n');
        text.push_str(&defaults);
    }
    write(&path, text.as_bytes())
}

/// Bring the project in step with its source.
///
/// # Errors
/// When the configuration or the source cannot be read, or a file cannot be written.
pub fn sync(root: &Path, options: &SyncOptions) -> Result<Report> {
    let (source, rev) = source_and_rev(root)?;
    let (dir, commit) = if is_git_url(&source) {
        let dir = checkout(None, &source, rev.as_deref())?;
        let commit = git_output(&dir, &["rev-parse", "HEAD"])?;
        (dir, Some(commit))
    } else {
        (root.join(&source), None)
    };
    readable(
        &dir,
        &source,
        &launcher::install(&source, commit.as_deref()),
    )?;
    let mut report = sync_from(root, &dir, *options)?;
    if is_git_url(&source) {
        let hint = report.lock_format.and_then(format_hint);
        report.warnings.extend(hint);
    }
    if let (Some(commit), false) = (&commit, options.dry_run) {
        report.warnings.extend(prefetch(
            &cache_dir()?,
            &source,
            commit,
            &run::Programs::default(),
        ));
    }
    Ok(report)
}

/// The hint at `cardinal update` of a lock of a lower format than this `cardinal` reads (SYN-55); the caller gives it
/// to a git source only, a local directory has no revision to update.
fn format_hint(format: u32) -> Option<String> {
    (format < FORMAT).then(|| {
        format!(
            "hint: {LOCK} is of the format {format}, this cardinal reads the format {FORMAT}: run `cardinal update` to move the project to a newer revision"
        )
    })
}

/// Fetch the tools binary of `commit` into the cache, as `cardinal run` would: a warning when it fails (BIN-13).
fn prefetch(cache: &Path, source: &str, commit: &str, programs: &run::Programs) -> Option<String> {
    run::binary_of(cache, source, commit, programs)
        .err()
        .map(|e| format!("warning: {e:#} (`cardinal run` will try again)"))
}

fn sync_from(root: &Path, source: &Path, options: SyncOptions) -> Result<Report> {
    let managed = source.join("files");
    let seeds = source.join("seed");
    if !managed.is_dir() {
        bail!(
            "{} has no files/ directory: is `source` right in {}?",
            source.display(),
            CONFIG
        );
    }
    let previous = read_lock(root)?;
    let lock_format = previous.as_ref().map(|lock| lock.format);
    let format = format_of(source)?;
    // read before anything is written: a migration that cannot be read stops the sync (SYN-52)
    let migrations = match &previous {
        Some(lock) => migration::applying(source, lock.format, format)?,
        None => Vec::new(),
    };
    let Lock {
        files: lock,
        entries: recorded,
        ..
    } = previous.unwrap_or_default();
    let wanted = entries::of_source(source)?;
    let all_files = list(&managed)?;
    // the features the project turns off are content the source does not manage (FEA-3), judged before any write
    let (files, wanted) = turned_on(root, source, &all_files, wanted)?;
    // decided first: a hook of the user that cannot be chained stops the sync before it writes anything
    let gate = gate::plan(root, gate::manages(&wanted, files.iter()))?;
    // installed before the managed files are written, so a failure leaves the project as it was
    if let (Some(gate), false) = (&gate, options.dry_run) {
        if !gate.removes() {
            gate.apply()?;
        }
    }
    let mut report = Report::default();

    let file_set: BTreeSet<&String> = files.iter().collect();
    let new_lock = write_managed(root, &managed, &files, &lock, options, &mut report)?;
    remove_dropped(root, &lock, &file_set, options, &mut report)?;
    // a managed path is never a seed, even one of a feature turned off (FEA-15)
    let all_set: BTreeSet<&String> = all_files.iter().collect();
    write_seeds(root, &seeds, &all_set, options, &mut report)?;

    // after the seeds: a first sync merges into the settings it was just seeded with, written or not
    let seeded = report.outcome(entries::SETTINGS) == Some(Outcome::Seeded);
    let base = if seeded { &seeds } else { root };
    let text = fs::read_to_string(base.join(entries::SETTINGS)).ok();
    // settings seeded again hold nothing of the project's: no recorded entry is in conflict there
    let recorded = if seeded { Vec::new() } else { recorded };
    let merge = entries::merge(text.as_deref(), &wanted, &recorded, options.force);
    if let (Some(text), false) = (&merge.write, options.dry_run) {
        write(&root.join(entries::SETTINGS), text.as_bytes())?;
    }
    report.entries.extend(merge.outcomes);

    // removed only when the lock no longer records the gate: settings left alone keep its hooks (SYN-60)
    let gate = gate.filter(|gate| !gate.removes() || !gate::manages(&merge.lock, files.iter()));
    if let Some(gate) = &gate {
        report.entries.extend(gate.outcomes());
    }

    let new_lock = Lock {
        files: new_lock,
        entries: merge.lock,
        format,
    };
    // a project file as this run leaves it: a removed one is gone, the settings are merged
    let removed: BTreeSet<String> = report
        .entries
        .iter()
        .filter(|(_, outcome)| *outcome == Outcome::Removed)
        .map(|(rel, _)| rel.clone())
        .collect();
    let merged = merge.write.clone().filter(|_| options.dry_run);
    let as_left = |rel: &str| -> Option<String> {
        if removed.contains(rel) {
            return None;
        }
        match (&merged, rel == entries::SETTINGS) {
            (Some(text), true) => Some(text.clone()),
            _ => fs::read_to_string(root.join(rel)).ok(),
        }
    };
    let rewritten = rewrite_seeded(root, &migrations, &file_set, &as_left, options, &mut report)?;
    if !options.dry_run {
        write_lock(root, &new_lock, format)?;
        // removed last: a sync that fails before leaves the gate its lock still records, the next one removes it
        if let Some(gate) = gate.filter(gate::Plan::removes) {
            gate.apply()?;
        }
    }
    // judged on the project as the sync leaves it, or would under `--dry-run`
    let done = |path: &str, outcome| report.outcome(path) == Some(outcome);
    let present = |path: &str| {
        done(path, Outcome::Added) || (root.join(path).is_file() && !done(path, Outcome::Removed))
    };
    let gated = gate::manages(&new_lock.entries, new_lock.files.keys());
    let attribution = !features::is_off(root, features::NO_AI_ATTRIBUTION);
    report.warnings = match &merge.settings {
        Some(settings) => settings::warnings_of(settings, gated, &present, attribution),
        None => settings::warnings(base, gated, &present, attribution),
    };
    report.warnings.extend(merge.warning);
    report
        .warnings
        .extend(leftovers(root, &migrations, &new_lock, &|rel| {
            rewritten.get(rel).cloned().or_else(|| as_left(rel))
        }));
    report.lock_format = if options.dry_run {
        lock_format
    } else {
        Some(format)
    };
    Ok(report)
}

/// The managed files and entries of `source` the project at `root` keeps: those of no feature it turns off.
fn turned_on(
    root: &Path,
    source: &Path,
    files: &[String],
    wanted: Vec<entries::Entry>,
) -> Result<(Vec<String>, Vec<entries::Entry>)> {
    let left = features::left_out(root, source, files, &wanted)?;
    let files = files
        .iter()
        .filter(|f| !left.files.contains(*f))
        .cloned()
        .collect();
    let wanted = wanted
        .into_iter()
        .filter(|e| !left.entries.contains(e))
        .collect();
    Ok((files, wanted))
}

/// Write the managed files of `managed` the project lacks or left as locked (all of them with `force`); returns the new
/// file lock.
fn write_managed(
    root: &Path,
    managed: &Path,
    files: &[String],
    lock: &BTreeMap<String, String>,
    options: SyncOptions,
    report: &mut Report,
) -> Result<BTreeMap<String, String>> {
    let mut new_lock = BTreeMap::new();
    for rel in files {
        let bytes = fs::read(managed.join(rel))?;
        let executable = is_executable(&managed.join(rel));
        let want = fingerprint(&bytes, executable);
        let have = fingerprint_of(&root.join(rel));
        let locked = lock.get(rel);
        let outcome = match &have {
            None => Outcome::Added,
            Some(h) if *h == want => Outcome::Unchanged,
            Some(h) if Some(h) == locked || options.force => Outcome::Updated,
            Some(_) => Outcome::Conflict,
        };
        match outcome {
            Outcome::Added | Outcome::Updated => {
                if !options.dry_run {
                    write_as(&root.join(rel), &bytes, executable)?;
                }
                new_lock.insert(rel.clone(), want);
            }
            Outcome::Unchanged => {
                new_lock.insert(rel.clone(), want);
            }
            _ => {
                if let Some(old) = locked {
                    new_lock.insert(rel.clone(), old.clone());
                }
            }
        }
        report.entries.push((rel.clone(), outcome));
    }
    Ok(new_lock)
}

/// Remove the locked files the source dropped, unless the project changed them.
fn remove_dropped(
    root: &Path,
    lock: &BTreeMap<String, String>,
    file_set: &BTreeSet<&String>,
    options: SyncOptions,
    report: &mut Report,
) -> Result<()> {
    for (rel, locked) in lock.iter().filter(|(rel, _)| !file_set.contains(rel)) {
        let path = root.join(rel);
        match fingerprint_of(&path) {
            None => {}
            Some(h) if h == *locked => {
                if !options.dry_run {
                    fs::remove_file(&path)?;
                }
                report.entries.push((rel.clone(), Outcome::Removed));
            }
            Some(_) => report.entries.push((rel.clone(), Outcome::Kept)),
        }
    }
    Ok(())
}

/// Rewrite the seeded texts of `migrations` in the project files, the managed ones and the lock excepted (SYN-53);
/// returns the new text of each file rewritten, written or not.
fn rewrite_seeded(
    root: &Path,
    migrations: &[migration::Migration],
    file_set: &BTreeSet<&String>,
    as_left: &dyn Fn(&str) -> Option<String>,
    options: SyncOptions,
    report: &mut Report,
) -> Result<BTreeMap<String, String>> {
    let mut texts: BTreeMap<&str, String> = BTreeMap::new();
    let mut changed = Vec::new();
    for rewrite in migrations.iter().flat_map(|m| &m.rewrites) {
        let rel = rewrite.path.as_str();
        if !is_inside(rel) || rel == LOCK || file_set.contains(&rewrite.path) {
            continue;
        }
        if !texts.contains_key(rel) {
            let Some(text) = as_left(rel) else {
                continue;
            };
            texts.insert(rel, text);
        }
        let text = texts.get_mut(rel).expect("read above");
        if let Some(new) = migration::rewritten(text, &rewrite.old, &rewrite.new) {
            *text = new;
            if !changed.contains(&rel) {
                changed.push(rel);
            }
        }
    }
    let mut done = BTreeMap::new();
    for rel in changed {
        if !options.dry_run {
            write(&root.join(rel), texts[rel].as_bytes())?;
        }
        report.entries.push((rel.to_string(), Outcome::Rewritten));
        done.insert(rel.to_string(), texts[rel].clone());
    }
    Ok(done)
}

/// The directories of the project that record its history: an old name stays there (SYN-54).
const HISTORY: [&str; 2] = ["issues/done/", "docs/adr/"];

/// A warning for each line of a project-owned file git tracks that still cites an old name of `migrations`, each file
/// judged as `as_left` gives it, the run done (SYN-54); none outside a git repository.
fn leftovers(
    root: &Path,
    migrations: &[migration::Migration],
    lock: &Lock,
    as_left: &dyn Fn(&str) -> Option<String>,
) -> Vec<String> {
    let names = migration::renamed(migrations);
    if names.is_empty() {
        return Vec::new();
    }
    // outside a git repository there is nothing to list
    let listed = match git::bytes(root, &["ls-files", "-z"]) {
        Ok(out) => String::from_utf8_lossy(&out).into_owned(),
        Err(_) => return Vec::new(),
    };
    let mut paths: Vec<&str> = listed
        .split('\0')
        .filter(|rel| {
            !rel.is_empty()
                && *rel != LOCK
                && !lock.files.contains_key(*rel)
                && !HISTORY.iter().any(|dir| rel.starts_with(dir))
        })
        .collect();
    paths.sort_unstable();
    let mut warnings = Vec::new();
    for rel in paths {
        let Some(text) = as_left(rel) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            for (format, name) in &names {
                if migration::cites(line, name) {
                    warnings.push(format!(
                        "warning: {rel}:{} cites {}, named {} since the format {format}",
                        number + 1,
                        name.old,
                        name.new
                    ));
                }
            }
        }
    }
    warnings
}

/// Copy the seeds of `seeds` the project does not have.
fn write_seeds(
    root: &Path,
    seeds: &Path,
    file_set: &BTreeSet<&String>,
    options: SyncOptions,
    report: &mut Report,
) -> Result<()> {
    for rel in list(seeds)? {
        let dst = root.join(&rel);
        // `symlink_metadata`: a dangling symlink is something the project has. A managed path is not a seed, nor the
        // configuration, which `init` writes.
        if file_set.contains(&rel) || rel == CONFIG || fs::symlink_metadata(&dst).is_ok() {
            continue;
        }
        if !options.dry_run {
            write_as(
                &dst,
                &fs::read(seeds.join(&rel))?,
                is_executable(&seeds.join(&rel)),
            )?;
        }
        report.entries.push((rel, Outcome::Seeded));
    }
    Ok(())
}

/// The commands of the hooks of `source` that only features the project at `root` turns off run (FEA-14); none when the
/// source or the features cannot be read, which the sync then reports.
fn turned_off(root: &Path, source: &Path) -> Vec<String> {
    let (Ok(files), Ok(wanted)) = (list(&source.join("files")), entries::of_source(source)) else {
        return Vec::new();
    };
    let Ok(left) = features::left_out(root, source, &files, &wanted) else {
        return Vec::new();
    };
    let kept: Vec<&str> = wanted
        .iter()
        .filter(|e| !left.entries.contains(e))
        .filter_map(entries::Entry::command)
        .collect();
    left.entries
        .iter()
        .filter_map(entries::Entry::command)
        .filter(|c| !kept.contains(c))
        .map(str::to_string)
        .collect()
}

/// What `update` did: the revision before and after, and the `sync` that followed.
#[derive(Debug)]
pub struct UpdateReport {
    /// The `rev` the project had, if any.
    pub previous: Option<String>,
    /// The commit the project was moved to, or would be.
    pub rev: String,
    /// False when nothing was written: `--dry-run`, or a conflict.
    pub applied: bool,
    /// True when the update left something to commit: a project file, the configuration or the lock it changed
    /// (SYN-61).
    pub changed: bool,
    pub sync: Report,
}

/// Move the project to `rev` of its git source (default: the head of the default branch): sync to it, then write
/// the resolved commit to `rev` in the configuration. A conflict, an unknown revision, an unreadable source or a hook the `cardinal` on the path refuses (BIN-16) leave the project as it was;
/// an I/O error during the write itself is not rolled back, as for `sync`.
///
/// # Errors
/// When the source is a local directory, the revision is unknown, or the configuration or sync fails.
pub fn update(root: &Path, rev: Option<&str>, options: &SyncOptions) -> Result<UpdateReport> {
    let mut done = update_in(None, root, rev, *options)?;
    if done.applied {
        let (source, _) = source_and_rev(root)?;
        done.sync.warnings.extend(prefetch(
            &cache_dir()?,
            &source,
            &done.rev,
            &run::Programs::default(),
        ));
    }
    Ok(done)
}

fn update_in(
    cache: Option<&Path>,
    root: &Path,
    rev: Option<&str>,
    options: SyncOptions,
) -> Result<UpdateReport> {
    let (source, previous) = source_and_rev(root)?;
    if !is_git_url(&source) {
        bail!("{CONFIG}: `source` is a local directory, which has no revision to update");
    }
    let dir = fetch(&cache_or_default(cache)?, &source)?;
    let commit = resolve_in(&dir, &source, rev)?;
    git(Some(&dir), &["checkout", "--quiet", "--detach", &commit])?;
    readable(
        &dir,
        &format!("{source} at {commit}"),
        &launcher::install(&source, Some(&commit)),
    )?;
    // The hooks of the target must not lock the agent out: the installed launcher has to know their subcommands. A
    // fragment that cannot be read fails the sync below, before anything is written.
    let off = turned_off(root, &dir);
    let mut refused: Vec<String> = fs::read_to_string(dir.join(entries::FRAGMENT))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .map_or_default(|fragment| launcher::refused(&fragment))
        .iter()
        .filter(|command| !off.contains(*command))
        .map(|command| {
            format!("the installed cardinal cannot run the hook `{command}` of {commit}")
        })
        .collect();
    if !refused.is_empty() {
        refused.push(format!(
            "install the cardinal of {commit} first, `{}`, then run `cardinal update` again",
            launcher::install(&source, Some(&commit))
        ));
    }
    if !refused.is_empty() && !options.dry_run {
        bail!("{}", refused.join("\n"));
    }
    let path = config_of(root);
    let current = fs::read_to_string(&path)?;
    // judged before anything is written: a configuration `update` cannot edit stops it (SYN-12)
    set_rev(&current, &commit, rev.unwrap_or("default branch"))?;
    // Nothing is written before the dry run shows a clean sync.
    let mut preview = sync_from(
        root,
        &dir,
        SyncOptions {
            dry_run: true,
            ..options
        },
    )?;
    preview
        .warnings
        .extend(refused.iter().map(|line| format!("warning: {line}")));
    let applied = !options.dry_run && preview.conflicts() == 0;
    let lock = fs::read(root.join(LOCK)).ok();
    let mut changed = false;
    let sync = if applied {
        let sync = sync_from(root, &dir, options)?;
        // the configuration as the sync left it: a migration may have rewritten it (SYN-53)
        let text = set_rev(
            &fs::read_to_string(&path)?,
            &commit,
            rev.unwrap_or("default branch"),
        )?;
        write(&path, text.as_bytes())?;
        changed = text != current
            || fs::read(root.join(LOCK)).ok() != lock
            || sync.entries.iter().any(|(_, outcome)| outcome.wrote());
        sync
    } else {
        preview
    };
    Ok(UpdateReport {
        previous,
        rev: commit,
        applied,
        changed,
        sync,
    })
}

#[cfg(test)]
mod tests;
