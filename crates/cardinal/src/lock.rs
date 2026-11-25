//! The lock of a project: the fingerprint of each managed file as written, the managed entries of its settings,
//! the format of the source it was synced from.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use crate::{entries, launcher, source_and_rev, write, FORMAT, LOCK};

/// Hash of `bytes`, followed by `+x` for an executable file: a mode change is a change.
pub(crate) fn fingerprint(bytes: &[u8], executable: bool) -> String {
    let mut print = hash(bytes);
    if executable {
        print.push_str("+x");
    }
    print
}

#[cfg(unix)]
pub(crate) fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
pub(crate) fn is_executable(_path: &Path) -> bool {
    false
}

/// Fingerprint of the file at `path`, `None` when it cannot be read.
pub(crate) fn fingerprint_of(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    Some(fingerprint(&bytes, is_executable(path)))
}

pub(crate) fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// A path of the lock must stay inside the project: relative, no `..`.
pub(crate) fn is_inside(rel: &str) -> bool {
    Path::new(rel)
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
}

/// What the lock records: the fingerprint of each managed file, the managed entries of the settings, and the format of
/// the source it was synced from (1 when it records none, SYN-41).
#[derive(Debug, Default)]
pub(crate) struct Lock {
    pub(crate) files: BTreeMap<String, String>,
    pub(crate) entries: Vec<entries::Entry>,
    pub(crate) format: u32,
}

/// The mark of a lock line that holds a managed entry of the settings instead of a file.
const ENTRY: &str = "entry";
/// The mark of the lock line that holds the format of the source it was synced from, followed by one space: a
/// `cardinal` older than SYN-41 skips a line with no two spaces.
const FORMAT_LINE: &str = "format ";

/// The lock of the project, `None` when it has none yet; an error when its source has a format this `cardinal` does
/// not read, judged before any other line (SYN-42).
pub(crate) fn read_lock(root: &Path) -> Result<Option<Lock>> {
    let path = root.join(LOCK);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    let lines = || text.lines().filter(|l| !l.starts_with('#'));
    let mut lock = Lock {
        format: 1,
        ..Lock::default()
    };
    for format in lines().filter_map(|line| line.strip_prefix(FORMAT_LINE)) {
        let format: u32 = format
            .parse()
            .with_context(|| format!("{}: not a format: {format}", path.display()))?;
        if format > FORMAT {
            bail!("{LOCK} was synced from a source of the format {format}, this one reads up to {FORMAT}: install it with {}", install_hint(root));
        }
        lock.format = format;
    }
    for line in lines().filter(|l| !l.starts_with(FORMAT_LINE)) {
        let Some((h, rel)) = line.split_once("  ") else {
            continue;
        };
        if h == ENTRY {
            let entry = entries::Entry::from_lock(rel)
                .with_context(|| format!("reading {}", path.display()))?;
            lock.entries.push(entry);
            continue;
        }
        if !is_inside(rel) {
            bail!("{}: path outside the project: {rel}", path.display());
        }
        lock.files.insert(rel.to_string(), h.to_string());
    }
    Ok(Some(lock))
}

/// The managed files the lock of the project at `root` lists, sorted by path; none when it has no lock.
pub(crate) fn managed_files(root: &Path) -> Result<Vec<String>> {
    Ok(read_lock(root)?.map_or_default(|lock| lock.files.into_keys().collect()))
}

/// The command that installs the `cardinal` of the revision the configuration pins, quoted, or what to install when it
/// cannot be read.
pub(crate) fn install_hint(root: &Path) -> String {
    match source_and_rev(root) {
        Ok((source, rev)) => format!("`{}`", launcher::install(&source, rev.as_deref())),
        Err(_) => "the cardinal of this revision".to_string(),
    }
}

/// Write the lock of a sync from a source of the format `format`.
pub(crate) fn write_lock(root: &Path, lock: &Lock, format: u32) -> Result<()> {
    let mut text = String::from(
        "# Written by `cardinal sync`: hashes of the managed files, then the managed settings entries. Do not edit.\n",
    );
    let _ = writeln!(text, "{FORMAT_LINE}{format}");
    for (path, h) in &lock.files {
        let _ = writeln!(text, "{h}  {path}");
    }
    for entry in &lock.entries {
        let _ = writeln!(text, "{ENTRY}  {}", entry.to_lock());
    }
    write(&root.join(LOCK), text.as_bytes())
}
