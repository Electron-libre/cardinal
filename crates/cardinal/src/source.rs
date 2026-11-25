//! The fetch of a git source: its URL as git reads it, its clone in the user cache, the checkout of a revision,
//! and `upgrade`.

use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::git::{self};
use crate::lock::{hash, is_executable};
use crate::{launcher, run};

// A plain suffix test, not `Path::extension`: `/x/.git` is a repository and `X.GIT` or `x.git/` a
// directory (SYN-49).
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub(crate) fn is_git_url(source: &str) -> bool {
    source.contains("://") || source.starts_with("git@") || source.ends_with(".git")
}

/// A scp-like `[user@]host:path` git source: no `://`, and a `:` before its first `/`, as git reads it.
pub(crate) fn is_scp_like(source: &str) -> bool {
    !source.contains("://")
        && source
            .split('/')
            .next()
            .is_some_and(|head| head.contains(':'))
}

/// The source as git and cargo read it (SYN-50): a git source that is a path becomes the absolute `file://` URL of
/// `root.join(source)`, `root` with its symlinks resolved, so the clone, its cache key and `cargo install --git` do not
/// depend on the working directory nor on the path the root was reached by; a URL, a scp-like `host:path` and a local
/// directory are returned as written.
pub(crate) fn resolved(root: &Path, source: &str) -> String {
    if !is_git_url(source) || source.contains("://") || is_scp_like(source) {
        return source.to_string();
    }
    let Ok(root) = fs::canonicalize(root).or_else(|_| std::path::absolute(root)) else {
        return source.to_string();
    };
    let mut path = PathBuf::from("/");
    for part in root.join(source).components() {
        match part {
            Component::Normal(name) => path.push(name),
            Component::ParentDir => {
                path.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
        }
    }
    let mut url = String::from("file://");
    for &byte in path_bytes(&path).iter() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            url.push(char::from(byte));
        } else {
            let _ = write!(url, "%{byte:02X}");
        }
    }
    url
}

/// The bytes of `path` as the system names it: its raw bytes on unix, its UTF-8 elsewhere.
#[cfg(unix)]
fn path_bytes(path: &Path) -> std::borrow::Cow<'_, [u8]> {
    use std::os::unix::ffi::OsStrExt;
    path.as_os_str().as_bytes().into()
}

#[cfg(not(unix))]
fn path_bytes(path: &Path) -> std::borrow::Cow<'_, [u8]> {
    path.to_string_lossy().into_owned().into_bytes().into()
}

/// Runs `git args` in `dir`, else in the current directory.
pub(crate) fn git(dir: Option<&Path>, args: &[&str]) -> Result<()> {
    git::bytes(dir.unwrap_or(Path::new(".")), args)
        .map(drop)
        .map_err(anyhow::Error::msg)
}

fn resolves(dir: &Path, rev: &str) -> bool {
    git::run_ok(
        dir,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{rev}^{{commit}}"),
        ],
    )
}

pub(crate) fn git_output(dir: &Path, args: &[&str]) -> Result<String> {
    git::run(dir, args).map_err(anyhow::Error::msg)
}

pub(crate) fn cache_dir() -> Result<PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .context("no cache directory (set HOME or XDG_CACHE_HOME)")
}

pub(crate) fn cache_or_default(cache: Option<&Path>) -> Result<PathBuf> {
    cache.map_or_else(cache_dir, |c| Ok(c.to_path_buf()))
}

/// Where the clone of the git source `url` lives in `cache`.
pub(crate) fn clone_dir(cache: &Path, url: &str) -> PathBuf {
    cache.join("cardinal").join(&hash(url.as_bytes())[..12])
}

/// Clone (first time) or fetch the git source in `cache`, and return its local directory.
pub(crate) fn fetch(cache: &Path, url: &str) -> Result<PathBuf> {
    let dir = clone_dir(cache, url);
    if dir.join(".git").exists() {
        git(Some(&dir), &["fetch", "--quiet", "--tags", "origin"])?;
        // `fetch` leaves `origin/HEAD` alone: follow a default branch the remote renamed (best effort).
        let _ = git_output(&dir, &["remote", "set-head", "origin", "--auto"]);
    } else {
        fs::create_dir_all(&dir)?;
        git(None, &["clone", "--quiet", url, &dir.to_string_lossy()])?;
    }
    Ok(dir)
}

/// Full commit hash of `rev` (default: the remote's default branch) in a fetched source.
pub(crate) fn resolve_in(dir: &Path, url: &str, rev: Option<&str>) -> Result<String> {
    if rev.is_some_and(|r| r.starts_with('-')) {
        bail!("{url}: invalid revision {}", rev.unwrap_or_default());
    }
    // A branch name must resolve to the remote branch the fetch just moved, not to a stale local one.
    let candidates = match rev {
        Some(rev) => vec![format!("origin/{rev}"), rev.to_string()],
        None => vec!["origin/HEAD".to_string()],
    };
    if let Some(target) = candidates.iter().find(|c| resolves(dir, c)) {
        return git_output(dir, &["rev-parse", &format!("{target}^{{commit}}")]);
    }
    let rev = rev.unwrap_or("origin/HEAD");
    if !is_full_hash(rev) {
        bail!("{url}: unknown revision {rev}");
    }
    // No branch or tag points to the pinned commit any more: ask the source for it by hash (hosts differ
    // on whether they serve it; a refusal is reported below).
    let fetched = git(Some(dir), &["fetch", "--quiet", "origin", rev]);
    if !resolves(dir, rev) {
        let why = fetched.map_or_else(|e| e.to_string(), |()| "it served no such commit".into());
        bail!("{url}: the pinned commit {rev} is gone from the source (or the source is unreachable: {why}); run `cardinal update` to pin a current revision");
    }
    git_output(dir, &["rev-parse", &format!("{rev}^{{commit}}")])
}

/// Replace the `cardinal` of the `PATH` (else the running one) with the binary of `commit` of the git source `source`, fetched as `run` fetches it
/// (BIN-19, BIN-21); `source` is resolved against `root` as the configuration would be. Returns the path replaced.
///
/// # Errors
/// When `commit` is not a full hash, `source` is not a git source, the binary cannot be fetched or the running one
/// cannot be replaced.
pub fn upgrade(root: &Path, source: &str, commit: &str) -> Result<PathBuf> {
    if !is_full_hash(commit) {
        bail!("`{commit}` is not a full commit hash: pass the `rev` that `cardinal update` writes");
    }
    if !is_git_url(source) {
        bail!(
            "`{source}` is not a git source: install the cardinal of a local source with `{}`",
            launcher::install(source, None)
        );
    }
    let binary = run::binary_of(
        &cache_dir()?,
        &resolved(root, source),
        commit,
        &run::Programs::default(),
    )?;
    let exe = launcher_on_path().map_or_else(
        || std::env::current_exe().context("finding the running cardinal"),
        Ok,
    )?;
    run::replace(&binary, &exe)?;
    Ok(exe)
}

/// The first executable `cardinal` of an absolute directory of the `PATH`, as the shell finds it, its symlinks left as they are: the launcher the hooks run, which `upgrade`
/// replaces rather than a link target of the cache or the binary it was started from (BIN-19).
fn launcher_on_path() -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .filter(|dir| dir.is_absolute())
        .map(|dir| dir.join("cardinal"))
        .find(|path| path.is_file() && is_executable(path))
}

/// A 40 (SHA-1) or 64 (SHA-256) hexadecimal digit commit name, as `update` writes it to `rev`.
pub(crate) fn is_full_hash(rev: &str) -> bool {
    matches!(rev.len(), 40 | 64) && rev.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Local checkout of a git source in the user cache, at `rev` (default: the remote's default branch).
pub(crate) fn checkout(cache: Option<&Path>, url: &str, rev: Option<&str>) -> Result<PathBuf> {
    let dir = fetch(&cache_or_default(cache)?, url)?;
    let commit = resolve_in(&dir, url, rev)?;
    git(Some(&dir), &["checkout", "--quiet", "--detach", &commit])?;
    Ok(dir)
}
