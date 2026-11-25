//! The metric of each axis of `cardinal-report`: efficiency, quality, rules, skills and fixed context, one table per
//! axis with its cells per group.

use std::collections::{BTreeMap, HashSet};

use super::{
    median, mix, number, short, sorted_pairs, table, value, By, Group, Row, Table, NO_MILESTONE,
    QUALITY,
};

pub(super) fn efficiency(groups: &[Group], anonymous: bool) -> Table {
    let mut columns = vec![
        "issues",
        "projects",
        "models",
        "versions",
        "turns",
        "tool_calls",
        "output_tokens",
        "input_tokens",
        "cache_read_share",
        "main_context_peak",
        "peak_sessions",
        "advisor_models",
        "advisor_sessions",
        "advisor_calls",
        "advisor_input_tokens",
        "advisor_output_tokens",
        "validations",
        "other_messages",
    ];
    if groups.iter().any(|g| g.by == By::Setup) {
        columns.insert(4, "revisions");
    }
    table("Efficiency", &columns, groups, |g| {
        let issues: HashSet<(&str, &str)> = g
            .sessions
            .iter()
            .flat_map(|r| {
                value(r, "issues")
                    .split(',')
                    .filter(|i| !i.is_empty())
                    .map(|i| (value(r, "project"), i))
            })
            .collect();
        let projects = if anonymous {
            g.sessions
                .iter()
                .map(|r| value(r, "project"))
                .collect::<HashSet<_>>()
                .len()
                .to_string()
        } else {
            mix(&g.sessions, "project")
        };
        let per_session =
            |f: &dyn Fn(&Row) -> u64| median(g.sessions.iter().map(|r| f(r)).collect()).to_string();
        let column = |c: &'static str| per_session(&move |r: &Row| number(r, c));
        let peaks = main_context_peaks(&g.sessions);
        let peak_sessions = peaks.len().to_string();
        let calls = filled(&g.sessions, "advisor_calls");
        let advised = |c: &str| median_or_empty(filled(&g.sessions, c));
        let mut row = vec![
            issues.len().to_string(),
            projects,
            mix(&g.sessions, "models"),
            mix(&g.sessions, "versions"),
            column("turns"),
            column("tool_calls"),
            column("output_tokens"),
            per_session(&input_sum),
            cache_read_share(&g.sessions),
            median_or_empty(peaks),
            peak_sessions,
            mix(&g.sessions, "advisor_models"),
            calls.len().to_string(),
            median_or_empty(calls),
            advised("advisor_input_tokens"),
            advised("advisor_output_tokens"),
            column("validations"),
            column("other_messages"),
        ];
        if g.by == By::Setup {
            row.insert(4, revision_mix(&g.sessions));
        }
        vec![row]
    })
}

/// `revision=n` for the revisions of `sessions`, named as REP-4 names them (REP-16).
fn revision_mix(sessions: &[&Row]) -> String {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for row in sessions {
        *counts.entry(short(value(row, "revision"))).or_default() += 1;
    }
    sorted_pairs(counts.iter().map(|(name, n)| (name.as_str(), *n)).collect())
}

/// The input of a session: its uncached input, cache read and cache creation tokens (REP-5).
fn input_sum(row: &Row) -> u64 {
    number(row, "input_tokens")
        + number(row, "cache_read_tokens")
        + number(row, "cache_creation_tokens")
}

/// The median over the sessions with input of their share read from the cache, as a whole percent rounded half up;
/// empty when no session has input (REP-12). The median of the ratios, not the ratio of the medians.
#[allow(clippy::cast_precision_loss)] // counts far below 2^52
fn cache_read_share(sessions: &[&Row]) -> String {
    let mut shares: Vec<f64> = sessions
        .iter()
        .filter_map(|r| match input_sum(r) {
            0 => None,
            sum => Some(number(r, "cache_read_tokens") as f64 * 100.0 / sum as f64),
        })
        .collect();
    shares.sort_by(f64::total_cmp);
    let n = shares.len();
    let middle = match n {
        0 => return String::new(),
        _ if n % 2 == 1 => shares[n / 2],
        _ => f64::midpoint(shares[n / 2 - 1], shares[n / 2]),
    };
    format!("{}%", (middle + 0.5).floor())
}

/// The `main_context_peak` of the sessions that have one (REP-9, REP-10): an archived row written before the column
/// is left out rather than counted as 0.
pub(super) fn main_context_peaks(sessions: &[&Row]) -> Vec<u64> {
    filled(sessions, "main_context_peak")
}

/// The sessions that set an advisor (REP-11): those whose `advisor_calls` is not empty.
pub(super) fn advisor_sessions(sessions: &[&Row]) -> usize {
    filled(sessions, "advisor_calls").len()
}

/// The values of `column` in the sessions whose cell is not empty.
fn filled(sessions: &[&Row], column: &str) -> Vec<u64> {
    sessions
        .iter()
        .filter_map(|r| value(r, column).parse().ok())
        .collect()
}

/// The median of `values`, empty when there is none.
fn median_or_empty(values: Vec<u64>) -> String {
    if values.is_empty() {
        String::new()
    } else {
        median(values).to_string()
    }
}

/// The `name=n;name=n` pairs of `column` summed over `sessions`, as `name=n, …`, most frequent first, then by name.
fn summed_pairs(sessions: &[&Row], column: &str) -> String {
    let mut counts: BTreeMap<&str, u64> = BTreeMap::new();
    for row in sessions {
        for (name, n) in value(row, column)
            .split(';')
            .filter_map(|p| p.split_once('='))
        {
            *counts.entry(name).or_default() += n.parse().unwrap_or(0);
        }
    }
    sorted_pairs(counts)
}

/// The sum of a column over `rows`, `None` when there is no row or one value is empty or not a number.
fn sum(rows: &[&Row], column: &str) -> Option<u64> {
    if rows.is_empty() {
        return None;
    }
    rows.iter()
        .map(|r| value(r, column).parse::<u64>().ok())
        .sum()
}

/// `a / b` with two decimals, empty when a term is missing or `b` is 0.
#[allow(clippy::cast_precision_loss)] // counts far below 2^52
fn ratio(a: Option<u64>, b: Option<u64>) -> String {
    match (a, b) {
        (Some(a), Some(b)) if b > 0 => format!("{:.2}", a as f64 / b as f64),
        _ => String::new(),
    }
}

/// `a / b` rounded half up to the unit, empty when a term is missing or `b` is 0.
fn per_unit(a: Option<u64>, b: Option<u64>) -> String {
    match (a, b) {
        (Some(a), Some(b)) if b > 0 => ((2 * a + b) / (2 * b)).to_string(),
        _ => String::new(),
    }
}

/// The rows of `inputs` whose `revision` belongs to `group`.
fn of_group<'a>(inputs: impl IntoIterator<Item = &'a Row>, group: &Group) -> Vec<&'a Row> {
    inputs
        .into_iter()
        .filter(|r| group.revisions.contains(value(r, "revision")))
        .collect()
}

/// The quality rows of `group`: those of its revisions, or, by setup, those of its setup over every non-empty revision,
/// as its sessions (REP-16), a `mixed` row matching no setup; or, by milestone, those of its milestone (REP-20).
fn quality_of_group<'a>(inputs: impl IntoIterator<Item = &'a Row>, group: &Group) -> Vec<&'a Row> {
    match group.by {
        By::Setup => inputs
            .into_iter()
            .filter(|r| !value(r, "revision").is_empty() && value(r, "setup") == group.name)
            .collect(),
        By::Milestone => {
            let name = Some(group.name.as_str()).filter(|n| *n != NO_MILESTONE);
            inputs
                .into_iter()
                .filter(|r| value(r, "milestone") == name.unwrap_or_default())
                .collect()
        }
        By::Revision => of_group(inputs, group),
    }
}

/// A ratio of the quality table (REP-6): its column, the summed columns it divides, how it divides them (`ratio` to two
/// decimals, `per_unit` to the unit), and the summed columns that must reach `--min-issues` for a verdict on it
/// (VRD-11, VRD-12).
pub(super) struct QualityRatio {
    metric: &'static str,
    numerator: &'static str,
    divisor: &'static str,
    divide: fn(Option<u64>, Option<u64>) -> String,
    pub(super) floored: &'static [&'static str],
}

const QUALITY_RATIOS: [QualityRatio; 9] = [
    QualityRatio {
        metric: "fixes_after_per_issue",
        numerator: "fixes_after",
        divisor: "issues",
        divide: ratio,
        floored: &["issues"],
    },
    QualityRatio {
        metric: "findings_per_review",
        numerator: "review_findings",
        divisor: "counted_reviews",
        divide: ratio,
        floored: &["issues", "counted_reviews"],
    },
    QualityRatio {
        metric: "doc_lines_per_spec_line",
        numerator: "doc_lines",
        divisor: "spec_lines",
        divide: ratio,
        floored: &["issues"],
    },
    QualityRatio {
        metric: "input_tokens_per_issue",
        numerator: "input_tokens",
        divisor: "token_issues",
        divide: per_unit,
        floored: &["token_issues"],
    },
    QualityRatio {
        metric: "output_tokens_per_issue",
        numerator: "output_tokens",
        divisor: "token_issues",
        divide: per_unit,
        floored: &["token_issues"],
    },
    // the shape of the specs the issues changed (REP-19, VRD-14)
    QualityRatio {
        metric: "bytes_per_spec",
        numerator: "spec_bytes",
        divisor: "specs",
        divide: per_unit,
        floored: &["issues", "specs"],
    },
    QualityRatio {
        metric: "intent_chars_per_spec",
        numerator: "spec_intent_chars",
        divisor: "specs",
        divide: per_unit,
        floored: &["issues", "specs"],
    },
    QualityRatio {
        metric: "requirements_per_spec",
        numerator: "spec_requirements",
        divisor: "specs",
        divide: ratio,
        floored: &["issues", "specs"],
    },
    QualityRatio {
        metric: "happy_paths_per_spec",
        numerator: "spec_happy_paths",
        divisor: "specs",
        divide: ratio,
        floored: &["issues", "specs"],
    },
];

pub(super) fn quality_ratio(metric: &str) -> Option<&'static QualityRatio> {
    QUALITY_RATIOS.iter().find(|r| r.metric == metric)
}

pub(super) fn quality(groups: &[Group], files: &[Vec<Row>]) -> Table {
    // the columns of the `delivered-quality` summary after `revision`, `setup` and `milestone`, in its order
    let summed: Vec<&str> = QUALITY.split('\t').skip(3).collect();
    let mut columns: Vec<&str> = summed
        .iter()
        .map(|c| {
            if *c == "sessions" {
                "issue_sessions"
            } else {
                c
            }
        })
        .collect();
    columns.extend(QUALITY_RATIOS.iter().map(|r| r.metric));
    table("Quality", &columns, groups, |g| {
        let rows = quality_of_group(files.iter().flatten(), g);
        let total = |c: &str| sum(&rows, c);
        let mut cells: Vec<String> = summed
            .iter()
            .map(|c| total(c).map_or_default(|n| n.to_string()))
            .collect();
        cells.extend(
            QUALITY_RATIOS
                .iter()
                .map(|r| (r.divide)(total(r.numerator), total(r.divisor))),
        );
        vec![cells]
    })
}

pub(super) fn rules(groups: &[Group], breach_rows: &[Row]) -> Table {
    let columns = [
        "rule",
        "applied",
        "breached",
        "breaches",
        "blocked",
        "breached_share",
        "bytes",
        "bytes_per_session",
    ];
    table("Rules", &columns, groups, |g| {
        let mut by_rule: BTreeMap<&str, Vec<&Row>> = BTreeMap::new();
        for row in of_group(breach_rows, g) {
            by_rule.entry(value(row, "rule")).or_default().push(row);
        }
        if by_rule.is_empty() {
            return vec![vec![String::new(); columns.len()]];
        }
        by_rule
            .into_iter()
            .map(|(rule, rows)| {
                let total = |c: &str| rows.iter().map(|r| number(r, c)).sum::<u64>();
                let bytes = sum(&rows, "bytes");
                let (applied, breached) = (total("applied"), total("breached"));
                let share = if applied > 0 {
                    format!("{}%", (breached * 200 + applied).div_euclid(applied * 2))
                } else {
                    String::new()
                };
                vec![
                    rule.to_string(),
                    applied.to_string(),
                    breached.to_string(),
                    total("breaches").to_string(),
                    total("blocked").to_string(),
                    share,
                    bytes.map_or_default(|b| b.to_string()),
                    per_unit(bytes, Some(applied)),
                ]
            })
            .collect()
    })
}

pub(super) fn skills(groups: &[Group], context: Option<&[Row]>) -> Table {
    let mut columns = vec!["calls", "hook_refusals"];
    if context.is_some() {
        columns.push("unused");
    }
    table("Skills", &columns, groups, |g| {
        let mut cells = vec![
            summed_pairs(&g.sessions, "skills"),
            summed_pairs(&g.sessions, "hook_refusals"),
        ];
        if let Some(context) = context {
            let called: HashSet<&str> = g
                .sessions
                .iter()
                .flat_map(|r| value(r, "skills").split(';'))
                .filter_map(|p| p.split_once('='))
                .map(|(n, _)| n.rsplit(':').next().unwrap_or(n))
                .collect();
            let shipped: std::collections::BTreeSet<&str> = of_group(context, g)
                .iter()
                .flat_map(|r| value(r, "skills").split(','))
                .filter(|s| !s.is_empty())
                .collect();
            cells.push(
                shipped
                    .into_iter()
                    .filter(|s| !called.contains(s))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        vec![cells]
    })
}

pub(super) fn fixed_context(groups: &[Group], context: &[Row]) -> Table {
    let bytes = [
        "claude_md_bytes",
        "agents_md_bytes",
        "skill_description_bytes",
    ];
    let mut columns = bytes.to_vec();
    columns.push("total_bytes");
    table("Fixed context", &columns, groups, |g| {
        let rows: Vec<&Row> = of_group(context, g)
            .into_iter()
            .filter(|r| bytes.iter().all(|c| value(r, c).parse::<u64>().is_ok()))
            .collect();
        let n = rows.len() as u64;
        let mean =
            |f: &dyn Fn(&Row) -> u64| per_unit(Some(rows.iter().map(|r| f(r)).sum()), Some(n));
        let mut cells: Vec<String> = bytes
            .iter()
            .map(|c| mean(&|r: &Row| number(r, c)))
            .collect();
        cells.push(mean(&|r: &Row| bytes.iter().map(|c| number(r, c)).sum()));
        vec![cells]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: VRD-1 (internal: the two lists are private to their tools)
    #[test]
    fn every_quality_ratio_is_a_metric_of_an_expectation() {
        for r in &QUALITY_RATIOS {
            assert!(
                super::super::super::cardinal_changes::METRICS.contains(&r.metric),
                "{}",
                r.metric
            );
        }
    }
}
