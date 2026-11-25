//! The configuration of the project: read, its `rev` rewritten in place, and the format of a source it names.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::source::resolved;
use crate::{config_of, CONFIG, FORMAT, MIN_FORMAT, SOURCE_FORMAT, TABLE};

pub(crate) fn read_config(root: &Path) -> Result<toml::Table> {
    let path = config_of(root);
    let text = fs::read_to_string(&path).with_context(|| {
        format!(
            "no {CONFIG} in {}: run `cardinal init <source>`",
            root.display()
        )
    })?;
    text.parse()
        .with_context(|| format!("parsing {}", path.display()))
}

/// The text of the configuration with `rev` set in its table, every other line left as it is.
///
/// The line becomes `rev = "<commit>"  # <name>`; a missing `rev` is inserted right after `source`.
pub(crate) fn set_rev(text: &str, commit: &str, name: &str) -> Result<String> {
    if name.contains(char::is_control) {
        bail!("invalid revision name {name:?}");
    }
    let line = if name == commit {
        format!("rev = \"{commit}\"")
    } else {
        format!("rev = \"{commit}\"  # {name}")
    };
    let key_is = |l: &str, key: &str| {
        l.trim_start()
            .strip_prefix(key)
            .is_some_and(|rest| rest.trim_start().starts_with('='))
    };
    let (config, table) = (CONFIG, TABLE);
    let header = format!("[{table}]");
    let mut in_table = false;
    let mut source_at = None;
    let mut rev_at = None;
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    for (i, l) in lines.iter().enumerate() {
        let trimmed = l.trim();
        if is_table_header(trimmed) {
            in_table = trimmed.starts_with(&header);
        } else if in_table && key_is(l, "rev") {
            rev_at = Some(i);
        } else if in_table && key_is(l, "source") {
            source_at = Some(i);
        }
    }
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    if let Some(i) = rev_at {
        let ending = &lines[i][lines[i].trim_end_matches(['\r', '\n']).len()..];
        out[i] = format!("{line}{ending}");
    } else if let Some(i) = source_at {
        if !out[i].ends_with('\n') {
            out[i].push_str(eol);
        }
        out.insert(i + 1, format!("{line}{eol}"));
    } else {
        bail!("{config}: missing `source` in the {header} table");
    }
    let out = out.concat();
    // Quoted or dotted keys are valid TOML this line-based edit does not see: refuse rather than corrupt.
    let written = out
        .parse::<toml::Table>()
        .ok()
        .and_then(|t| t.get(table)?.get("rev")?.as_str().map(|r| r == commit));
    if written != Some(true) {
        bail!("{config}: cannot rewrite `rev` automatically, set `rev = \"{commit}\"` in {header} by hand");
    }
    Ok(out)
}

/// A `[table]` line, as opposed to a continuation line of a multi-line array such as `["a"],`.
fn is_table_header(line: &str) -> bool {
    let Some(rest) = line.strip_prefix('[') else {
        return false;
    };
    let Some((name, after)) = rest.split_once(']') else {
        return false;
    };
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ' '))
        && (after.trim().is_empty() || after.trim_start().starts_with('#'))
}

/// The `source`, resolved (SYN-50), and the pinned `rev` of the `[cardinal]` table of the configuration.
pub(crate) fn source_and_rev(root: &Path) -> Result<(String, Option<String>)> {
    let config = read_config(root)?;
    let table = config.get(TABLE).and_then(toml::Value::as_table);
    let text = |key: &str| {
        table
            .and_then(|t| t.get(key))
            .and_then(toml::Value::as_str)
            .map(str::to_string)
    };
    let source = text("source")
        .with_context(|| format!("{CONFIG}: missing `source` in the [{TABLE}] table"))?;
    Ok((resolved(root, &source), text("rev")))
}

/// The format the source in `dir` declares, 1 when it declares none.
pub(crate) fn format_of(dir: &Path) -> Result<u32> {
    let path = dir.join(SOURCE_FORMAT);
    match fs::read_to_string(&path) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(1),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        Ok(text) => text
            .trim()
            .parse()
            .with_context(|| format!("{}: not a whole number", path.display())),
    }
}

/// An error when the source `name` in `dir` needs a newer `cardinal`, which `install` installs (SYN-40), or an older
/// one (SYN-51); a directory without `files/` is no source, which the sync reports (SYN-10).
pub(crate) fn readable(dir: &Path, name: &str, install: &str) -> Result<()> {
    if !dir.join("files").is_dir() {
        return Ok(());
    }
    let format = format_of(dir)?;
    if format > FORMAT {
        bail!("{name} needs a cardinal that reads the format {format}, this one reads up to {FORMAT}: install it with `{install}`");
    }
    if format < MIN_FORMAT {
        bail!("{name} has the format {format}, this cardinal syncs the format {MIN_FORMAT} and higher");
    }
    Ok(())
}
