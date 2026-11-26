//! The band `by issue` of the charts screen (SUC-7 to SUC-9): five series read from the `issues.tsv` of the last
//! measure (QUA-23), each a line of its name, its largest and its last value, then three rows of one column per done
//! issue, the newest last, in eighths of a cell, under them the days of the first and the last issue drawn.

use std::path::Path;

use crate::panels::{Mark, Panel};
use crate::theme::Role;

/// The rows of the columns of a series (SUC-7).
const ROWS: usize = 3;

/// The rows a series takes: its line and its columns.
const SERIES_ROWS: usize = ROWS + 1;

/// The partial cells of a column, from one eighth to seven.
const EIGHTHS: [char; 7] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇'];

/// How a series writes its values (SUC-8).
#[derive(Clone, Copy)]
enum Unit {
    Duration,
    Tokens,
    Count,
}

/// The series of the band, in their order: name, column of `issues.tsv`, unit (SUC-7).
const SERIES: [(&str, &str, Unit); 5] = [
    ("time", "duration_s", Unit::Duration),
    ("input tokens", "input_tokens", Unit::Tokens),
    ("tool calls", "tool_calls", Unit::Count),
    ("fixes after", "fixes_after", Unit::Count),
    ("findings", "review_findings", Unit::Count),
];

/// The number of series of the band.
pub const COUNT: usize = SERIES.len();

/// A done issue: the day it closed and its value in each series, `None` for an empty cell.
#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub day: String,
    pub values: [Option<f64>; COUNT],
}

/// The done issues of the last measure, oldest first; `None` when the measure has no `issues.tsv` (SUC-7).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Timeline(pub Option<Vec<Issue>>);

/// The issues of the `issues.tsv` of the measure of `dir`, in the order of the file (SUC-7).
pub fn read(dir: Option<&Path>) -> Timeline {
    let Some(text) = dir.and_then(|d| std::fs::read_to_string(d.join("issues.tsv")).ok()) else {
        return Timeline(None);
    };
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    let at = |column: &str| header.iter().position(|h| *h == column);
    let closed = at("closed");
    let columns = SERIES.map(|(_, column, _)| at(column));
    let issues = lines
        .filter(|l| !l.is_empty())
        .map(|l| {
            let cells: Vec<&str> = l.split('\t').collect();
            let cell = |i: Option<usize>| i.and_then(|i| cells.get(i)).copied().unwrap_or("");
            Issue {
                day: cell(closed).chars().take(10).collect(),
                values: columns.map(|c| cell(c).parse().ok()),
            }
        })
        .collect();
    Timeline(Some(issues))
}

/// The rows the band takes with `series` series, borders included (SUC-9).
pub fn height(timeline: &Timeline, series: usize) -> u16 {
    let lines = if timeline.0.is_some() {
        series * SERIES_ROWS + 1
    } else {
        1
    };
    u16::try_from(lines + 2).unwrap_or(u16::MAX)
}

/// How many series fit in `rows` rows, borders included: at least one, at most all (SUC-9).
pub fn fitting(rows: u16) -> usize {
    (usize::from(rows).saturating_sub(3) / SERIES_ROWS).clamp(1, COUNT)
}

/// The series picked after `first`: after the last one, the first (SUC-9).
pub fn next(first: usize) -> usize {
    (first + 1) % COUNT
}

/// `value` as its series writes it (SUC-8).
fn shown(value: Option<f64>, unit: Unit) -> String {
    let Some(v) = value else {
        return "-".to_string();
    };
    // the values of `issues.tsv` are whole numbers of seconds, tokens or counts
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole = v.max(0.0).round() as u64;
    match unit {
        Unit::Duration if whole < 60 => format!("{whole}s"),
        Unit::Duration if whole < 3600 => format!("{}m", whole / 60),
        Unit::Duration => format!("{}h{:02}m", whole / 3600, whole % 3600 / 60),
        Unit::Tokens if whole < 1000 => whole.to_string(),
        // 999,500 and over rounds to 1000k: written in millions
        Unit::Tokens if whole < 999_500 => format!("{}k", (v / 1000.0).round()),
        Unit::Tokens => format!("{:.1}M", v / 1_000_000.0),
        Unit::Count => whole.to_string(),
    }
}

/// The cell of row `row` (0 the top one) of a column of `value` over `max`, from zero, in eighths (SUC-8).
fn cell(value: f64, max: f64, row: usize) -> char {
    if max <= 0.0 || value <= 0.0 {
        return ' ';
    }
    let cells = u32::try_from(ROWS * 8).unwrap_or(u32::MAX);
    // within 0..=ROWS*8 by the ratio, so the cast neither truncates nor loses a sign
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let eighths = ((value / max).min(1.0) * f64::from(cells)).round() as usize;
    // a value above zero takes at least an eighth, so it is never drawn as a zero (SUC-8)
    let eighths = eighths.max(1);
    let below = (ROWS - 1 - row) * 8;
    match eighths.saturating_sub(below).min(8) {
        0 => ' ',
        8 => '█',
        part => EIGHTHS[part - 1],
    }
}

/// The panel of the band laid out in `width` columns inside its borders (SUC-7 to SUC-9): `count` series from the
/// series `first`, those after it following, after the last the first one; all of them in their order when `count`
/// holds them.
pub fn panel(timeline: &Timeline, width: usize, first: usize, count: usize) -> Panel {
    let mut lines = Vec::new();
    let mut marks = Vec::new();
    match &timeline.0 {
        None => lines.push("no rows per issue: m measures again".to_string()),
        Some(issues) => {
            let drawn = &issues[issues.len().saturating_sub(width)..];
            // with room for all of them, the series keep their order (SUC-7)
            let first = if count >= COUNT { 0 } else { first };
            for k in 0..count.min(COUNT) {
                let (name, _, unit) = SERIES[(first + k) % COUNT];
                let i = (first + k) % COUNT;
                let values: Vec<Option<f64>> = drawn.iter().map(|d| d.values[i]).collect();
                let max = values
                    .iter()
                    .flatten()
                    .copied()
                    .fold(None, |m: Option<f64>, v| Some(m.map_or(v, |m| m.max(v))));
                let last = values.last().copied().flatten();
                lines.push(format!(
                    "{name}  max {}  last {}",
                    shown(max, unit),
                    shown(last, unit)
                ));
                for row in 0..ROWS {
                    let n = lines.len();
                    let mut line = String::new();
                    for v in &values {
                        let start = line.len();
                        let (c, role) = match v {
                            // an empty cell (no session, or no counted review) is never drawn as a zero (SUC-8)
                            None if row == ROWS - 1 => ('·', Role::Muted),
                            None => (' ', Role::Accent),
                            Some(v) => (cell(*v, max.unwrap_or(0.0), row), Role::Accent),
                        };
                        line.push(c);
                        if c != ' ' {
                            marks.push(Mark {
                                line: n,
                                range: start..line.len(),
                                role,
                            });
                        }
                    }
                    lines.push(line);
                }
            }
            lines.push(days(drawn));
        }
    }
    Panel {
        title: "by issue".into(),
        lines,
        marks,
        headings: Vec::new(),
        failed: false,
        corner: None,
    }
}

/// The days of the first and the last issue of `drawn`, the last ending under its column when there is room (SUC-7).
fn days(drawn: &[Issue]) -> String {
    match drawn {
        [] => String::new(),
        [one] => one.day.clone(),
        [first, .., last] => {
            let gap = drawn
                .len()
                .saturating_sub(first.day.len() + last.day.len())
                .max(1);
            format!("{}{:gap$}{}", first.day, "", last.day)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str =
        "issue\tclosed\tfixes_after\treview_findings\tinput_tokens\tduration_s\ttool_calls";

    fn measure(rows: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let text = std::iter::once(HEADER)
            .chain(rows.iter().copied())
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(dir.path().join("issues.tsv"), text + "\n").unwrap();
        dir
    }

    /// Three issues that took an hour, half an hour and a quarter of an hour.
    fn three() -> Timeline {
        let dir = measure(&[
            "a\t2026-11-24T10:00:00+01:00\t1\t2\t13600000\t3600\t70",
            "b\t2026-11-25T10:00:00+01:00\t0\t\t950000\t1800\t40",
            "c\t2026-11-26T10:00:00+01:00\t0\t3\t\t900\t",
        ]);
        read(Some(dir.path()))
    }

    // spec: SUC-7 (internal: the charts screen is drawn only in the terminal, never by `--print`)
    #[test]
    fn each_series_draws_a_column_per_issue_the_newest_last_over_the_days() {
        let p = panel(&three(), 40, 0, COUNT);
        let names: Vec<&str> = p
            .lines
            .iter()
            .step_by(SERIES_ROWS)
            .take(COUNT)
            .map(|l| l.split("  ").next().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "time",
                "input tokens",
                "tool calls",
                "fixes after",
                "findings"
            ]
        );
        assert_eq!(p.lines[0], "time  max 1h00m  last 15m");
        assert_eq!(&p.lines[1..4], ["█  ", "█▄ ", "██▆"]);
        assert_eq!(p.lines[4], "input tokens  max 13.6M  last -");
        assert_eq!(p.lines.last().unwrap(), "2026-11-24 2026-11-26");
        assert_eq!(p.title, "by issue");
    }

    // spec: SUC-7 (internal: the charts screen is drawn only in the terminal, never by `--print`)
    #[test]
    fn a_band_narrower_than_the_issues_draws_the_newest() {
        let rows: Vec<String> = (0..300)
            .map(|n| {
                format!(
                    "i{n}\t2026-{:02}-01T00:00:00Z\t0\t0\t1\t{}\t1",
                    n % 12 + 1,
                    n + 1
                )
            })
            .collect();
        let dir = measure(&rows.iter().map(String::as_str).collect::<Vec<_>>());
        let p = panel(&read(Some(dir.path())), 100, 0, 1);
        assert!(p.lines[1..4].iter().all(|l| l.chars().count() == 100));
        // the 100th newest is the 201st issue, n = 200, its month 200 % 12 + 1
        assert!(p.lines[4].starts_with("2026-09-01"), "{:?}", p.lines[4]);
        assert_eq!(p.lines[0], "time  max 5m  last 5m");
    }

    // spec: SUC-7 (internal: the charts screen is drawn only in the terminal, never by `--print`)
    #[test]
    fn a_measure_without_rows_per_issue_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let t = read(Some(dir.path()));
        assert_eq!(t, Timeline(None));
        assert_eq!(
            panel(&t, 40, 0, 5).lines,
            ["no rows per issue: m measures again"]
        );
        assert_eq!(height(&t, 5), 3);
    }

    // spec: SUC-8 (internal: the charts screen is drawn only in the terminal, never by `--print`)
    #[test]
    fn a_column_is_drawn_from_zero_in_eighths() {
        let dir = measure(&[
            "a\t2026-11-24\t0\t0\t1\t1800\t1",
            "b\t2026-11-25\t0\t0\t1\t3600\t1",
        ]);
        let p = panel(&read(Some(dir.path())), 40, 0, 1);
        assert_eq!(&p.lines[1..4], [" █", "▄█", "██"]);
        assert_eq!(cell(1.0, 24.0, 2), '▁');
        assert_eq!(cell(0.0, 24.0, 2), ' ');
        // a value above zero is never drawn as a zero
        assert_eq!(cell(5.0, 300.0, 2), '▁');
        assert_eq!(cell(5.0, 300.0, 1), ' ');
    }

    // spec: SUC-8 (internal: the charts screen is drawn only in the terminal, never by `--print`)
    #[test]
    fn an_empty_cell_is_a_muted_dot_never_a_zero() {
        let p = panel(&three(), 40, 1, 1);
        assert_eq!(p.lines[0], "input tokens  max 13.6M  last -");
        assert_eq!(p.lines[3], "█▂·");
        let dot = p
            .marks
            .iter()
            .find(|m| m.line == 3 && m.range.start == "█▂".len())
            .unwrap();
        assert_eq!(dot.role, Role::Muted);
        assert!(p
            .marks
            .iter()
            .filter(|m| m.line == 3 && m.range.start == 0)
            .all(|m| m.role == Role::Accent));
    }

    // spec: SUC-8 (internal: the charts screen is drawn only in the terminal, never by `--print`)
    #[test]
    fn each_series_writes_its_values_in_its_unit() {
        assert_eq!(shown(Some(8369.0), Unit::Duration), "2h19m");
        assert_eq!(shown(Some(722.0), Unit::Duration), "12m");
        assert_eq!(shown(Some(40.0), Unit::Duration), "40s");
        assert_eq!(shown(Some(13_624_519.0), Unit::Tokens), "13.6M");
        assert_eq!(shown(Some(950_000.0), Unit::Tokens), "950k");
        assert_eq!(shown(Some(800.0), Unit::Tokens), "800");
        assert_eq!(shown(Some(999_600.0), Unit::Tokens), "1.0M");
        assert_eq!(shown(Some(77.0), Unit::Count), "77");
        assert_eq!(shown(None, Unit::Count), "-");
    }

    // spec: SUC-9 (internal: the charts screen is drawn only in the terminal, never by `--print`)
    #[test]
    fn the_band_shows_the_picked_series_then_the_following_ones() {
        let names = |first, count| {
            let p = panel(&three(), 40, first, count);
            p.lines
                .iter()
                .step_by(SERIES_ROWS)
                .take(count)
                .map(|l| l.split("  ").next().unwrap().to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(3, 2), ["fixes after", "findings"]);
        assert_eq!(names(4, 2), ["findings", "time"]);
        // with room for all five, the order of SUC-7 whatever the pick
        assert_eq!(
            names(2, COUNT),
            [
                "time",
                "input tokens",
                "tool calls",
                "fixes after",
                "findings"
            ]
        );
        assert_eq!(fitting(0), 1);
        assert_eq!(fitting(height(&three(), 2)), 2);
        assert_eq!(fitting(200), COUNT);
    }
}
