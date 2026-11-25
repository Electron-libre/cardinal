//! The verdicts of `cardinal-report` (`specs/cardinal-verdicts.md`): each expectation of a recorded change judged
//! from the groups before and after it, under the floors of sessions and issues.

use std::collections::HashSet;

use super::super::cardinal_changes::{RULE_METRIC, WASTE_METRIC};
use super::axes::{advisor_sessions, main_context_peaks, quality_ratio};
use super::{name_counts, value, Group, Printed, Row, Table};

/// The value of `metric` for group `g` in the table VRD-1 takes it from, as printed and as a number
/// (a percent, as `breached_share` of the rule's row or `cache_read_share`, with its `%` dropped); `None` when that
/// table is not printed or the cell is not a finite number.
fn metric_value(tables: &[Table], g: usize, metric: &str) -> Option<(String, f64)> {
    let (title, column, key) = match metric {
        _ if quality_ratio(metric).is_some() => ("Quality", metric, None),
        "total_bytes" => ("Fixed context", metric, None),
        _ => match (
            metric.strip_prefix(RULE_METRIC),
            metric.strip_prefix(WASTE_METRIC),
        ) {
            (Some(rule), _) => ("Rules", "breached_share", Some(("rule", rule))),
            (_, Some(rule)) => ("Rules", "bytes_per_session", Some(("rule", rule))),
            _ => ("Efficiency", metric, None),
        },
    };
    let cell = tables
        .iter()
        .find(|t| t.title == title)?
        .cell(g, column, key)?
        .trim_end_matches('%');
    let number = cell.parse::<f64>().ok().filter(|n| n.is_finite())?;
    Some((cell.to_string(), number))
}

/// The most frequent name of `column` over the sessions of a group, the first by name among equals.
fn dominant<'a>(group: &Group<'a>, column: &str) -> Option<&'a str> {
    name_counts(&group.sessions, column)
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0)))
        .map(|(name, _)| name)
}

/// The metrics read from the sessions that set an advisor (REP-11).
const ADVISOR_METRICS: [&str; 3] = [
    "advisor_calls",
    "advisor_input_tokens",
    "advisor_output_tokens",
];

/// The floors of the verdicts: the sessions of a group (VRD-9, VRD-10) and its issues (VRD-11, VRD-12).
#[derive(Clone, Copy)]
pub(super) struct Floors {
    pub(super) sessions: usize,
    pub(super) issues: usize,
}

/// Whether group `g` holds fewer values of `metric` than its floor: sessions with a value (VRD-9, VRD-10), as only
/// `main_context_peak` and the advisor columns can be missing from a session row, or issues (and counted reviews) in the
/// quality table for one of its ratios (VRD-11, VRD-12).
fn too_few_values(
    tables: &[Table],
    groups: &[Group],
    g: usize,
    metric: &str,
    floors: Floors,
) -> bool {
    let too_few = |column: &str| {
        tables
            .iter()
            .find(|t| t.title == "Quality")
            .and_then(|t| t.cell(g, column, None))
            .and_then(|c| c.parse::<usize>().ok())
            .is_none_or(|n| n < floors.issues)
    };
    let group = &groups[g];
    if let Some(ratio) = quality_ratio(metric) {
        return ratio.floored.iter().any(|c| too_few(c));
    }
    match metric {
        "main_context_peak" => main_context_peaks(&group.sessions).len() < floors.sessions,
        _ if ADVISOR_METRICS.contains(&metric) => {
            advisor_sessions(&group.sessions) < floors.sessions
        }
        _ => false,
    }
}

/// The verdict of each expectation of `changes` (VRD-5, VRD-6, VRD-9, VRD-10, VRD-11, VRD-12), from the groups before
/// and after its change.
pub(super) fn verdicts(
    groups: &[Group],
    tables: &[Table],
    changes: &[Row],
    floors: Floors,
) -> Printed {
    // Per row: the revisions holding its change, and the index of its after group.
    let placed: Vec<(HashSet<&str>, Option<usize>)> = changes
        .iter()
        .map(|row| {
            let held: HashSet<&str> = value(row, "revisions")
                .split(',')
                .filter(|r| !r.is_empty())
                .collect();
            let after = groups
                .iter()
                .position(|g| g.revisions.iter().any(|r| held.contains(r)));
            (held, after)
        })
        .collect();
    let columns = [
        "change",
        "metric",
        "expected",
        "before",
        "after",
        "before_value",
        "after_value",
        "verdict",
    ];
    let mut rows = Vec::new();
    for (row, (held, after_index)) in changes.iter().zip(&placed) {
        let change = value(row, "change");
        let (metric, direction) = (value(row, "metric"), value(row, "direction"));
        let before_index = after_index.and_then(|i| i.checked_sub(1));
        let name = |i: Option<usize>| i.map_or_default(|i| groups[i].name.clone());
        let at = |i: Option<usize>| i.and_then(|i| metric_value(tables, i, metric));
        let (before_value, after_value) = (at(before_index), at(*after_index));
        let verdict = match (before_index, after_index, &before_value, &after_value) {
            (Some(b), Some(a), Some((_, bv)), Some((_, av))) => {
                let (before, after) = (&groups[b], &groups[*a]);
                // an advisor metric also moves with the advisor set (VRD-10)
                let advisor = ADVISOR_METRICS
                    .contains(&metric)
                    .then_some("advisor_models");
                let mix_changed = ["models", "versions", "project"]
                    .into_iter()
                    .chain(advisor)
                    .any(|c| dominant(before, c) != dominant(after, c));
                let other_change = changes
                    .iter()
                    .zip(&placed)
                    .any(|(other, (_, i))| value(other, "change") != change && i == after_index);
                // Only the last group can be too few in sessions, and the before group comes earlier; either can
                // be too few in values of the metric.
                if after.too_few
                    || too_few_values(tables, groups, b, metric, floors)
                    || too_few_values(tables, groups, *a, metric, floors)
                    || after.revisions.iter().any(|r| !held.contains(r))
                    || mix_changed
                    || other_change
                {
                    "confounded"
                } else {
                    moved(*bv, *av, direction)
                }
            }
            _ => "no data",
        };
        rows.push(vec![
            change.chars().take(7).collect(),
            metric.to_string(),
            direction.to_string(),
            name(before_index),
            name(*after_index),
            before_value.map_or_default(|(printed, _)| printed),
            after_value.map_or_default(|(printed, _)| printed),
            verdict.to_string(),
        ]);
    }
    Printed {
        title: "Verdicts".to_string(),
        columns: columns.iter().map(|c| (*c).to_string()).collect(),
        rows,
    }
}

/// The verdict of a value that went from `before` to `after` when the change expected it to go `direction` (VRD-6).
#[allow(clippy::float_cmp)] // `-0.0` equals `0.0`, as VRD-6 reads two printed values; `total_cmp` would order them
fn moved(before: f64, after: f64, direction: &str) -> &'static str {
    if after == before {
        "no observed effect"
    } else if (after > before) == (direction == "up") {
        "helped"
    } else {
        "hurt"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: VRD-6 (internal: the tables of the binary never print a value that is not finite)
    #[test]
    fn a_value_that_is_not_finite_is_missing() {
        let table = |cell: &str| Table {
            title: "Efficiency",
            columns: vec!["turns".to_string()],
            rows: vec![vec![vec![cell.to_string()]]],
            printed: Printed {
                title: String::new(),
                columns: Vec::new(),
                rows: Vec::new(),
            },
        };
        let at = |cell: &str| metric_value(&[table(cell)], 0, "turns");
        assert_eq!(at("NaN"), None);
        assert_eq!(at("inf"), None);
        assert_eq!(at("12.50%"), Some(("12.50".to_string(), 12.5)));
    }

    // spec: VRD-6 (internal: a metric of counts never prints `-0.00`, the binary cannot reach this edge)
    #[test]
    fn equal_values_are_no_observed_effect_whatever_their_sign() {
        assert_eq!(moved(0.0, -0.0, "up"), "no observed effect");
        assert_eq!(moved(-0.0, 0.0, "down"), "no observed effect");
        assert_eq!(moved(1.0, 2.0, "up"), "helped");
        assert_eq!(moved(1.0, 2.0, "down"), "hurt");
    }
}
