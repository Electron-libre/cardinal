//! The warnings `check-specs` prints on stderr once the specs and their citations are read: the specs over a bound
//! (CLR-13, CLR-14), the `accepted` ones lacking a part (CLR-10, CLR-12), the requirements proven only by internal
//! tests with no reason (TRC-1 to TRC-4, TRC-6).

use super::Spec;
use crate::tools::citations::{is_internal, Cite};

use std::collections::BTreeMap;

/// Warn of each `draft` spec over a bound, with its measures (CLR-13), and in one line of the `accepted` ones (CLR-14).
pub(super) fn bound_warnings(specs: &[Spec], bounds: &[u64; 3]) {
    for spec in specs.iter().filter(|s| s.status == "draft") {
        let over = spec.outgrown(bounds);
        if !over.is_empty() {
            eprintln!(
                "warning: {}: outgrows its capability: {}; split it (skill:write-spec)",
                spec.file,
                over.join(", ")
            );
        }
    }
    warn_accepted(
        specs,
        [
            "outgrows a bound of spec_max_*",
            "outgrow a bound of spec_max_*",
        ],
        |s| s.outgrown(bounds).is_empty(),
    );
}

/// Print one warning naming the `accepted` specs `has` rejects, if any, `what` saying of one or several of them what
/// they lack.
pub(super) fn warn_accepted(
    specs: &[Spec],
    [one, several]: [&str; 2],
    has: impl Fn(&Spec) -> bool,
) {
    let lacking: Vec<&str> = specs
        .iter()
        .filter(|s| s.status == "accepted" && !has(s))
        .map(|s| s.file.as_str())
        .collect();
    if !lacking.is_empty() {
        let (plural, what) = if lacking.len() == 1 {
            ("", one)
        } else {
            ("s", several)
        };
        eprintln!(
            "warning: {} accepted spec{plural} {what}: {}",
            lacking.len(),
            lacking.join(", ")
        );
    }
}

/// Warn, per spec, of the cited requirements whose every citation is internal, one at least giving no reason
/// (TRC-1 to TRC-4, TRC-6), the internal ones being under a directory named like one of `internal_dirs`.
pub(super) fn boundary_warnings(
    specs: &[Spec],
    cited: &BTreeMap<String, Vec<Cite>>,
    internal_dirs: &[String],
) {
    let dirs = internal_dirs
        .iter()
        .map(|d| format!("`{d}`"))
        .collect::<Vec<_>>()
        .join(", ");
    for spec in specs {
        let internal: Vec<&str> = spec
            .requirements
            .iter()
            .filter(|r| !r.exempt)
            .filter(|r| {
                cited.get(&r.id).is_some_and(|cites| {
                    cites.iter().all(|c| is_internal(&c.file, internal_dirs))
                        && cites.iter().any(|c| !c.reasoned)
                })
            })
            .map(|r| r.id.as_str())
            .collect();
        if !internal.is_empty() {
            let plural = if internal.len() == 1 { "" } else { "s" };
            eprintln!(
                "warning: {}: {} requirement{plural} proven only by tests under {dirs} with no reason: {} \
                 (give it as `spec: ID (internal: <reason>)` after the comment marker)",
                spec.file,
                internal.len(),
                internal.join(", ")
            );
        }
    }
}
