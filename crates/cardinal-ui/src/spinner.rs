//! The mark of an issue at work that turns on the screen (SUQ-4): the frame of the time the screen has run, written
//! over the mark `--print` keeps still, so it turns at each tick with no read of the project.

use std::time::Duration;

use crate::panels::{Panel, AT_WORK};
use crate::theme::Role;

/// The frames of the turning mark, one per `STEP`.
const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// How long a frame stays on the screen.
const STEP: Duration = Duration::from_millis(100);

/// The frame of the turning mark once the screen has run for `elapsed`.
pub fn frame(elapsed: Duration) -> &'static str {
    let n = elapsed.as_millis() / STEP.as_millis();
    FRAMES[usize::try_from(n % FRAMES.len() as u128).unwrap_or(0)]
}

/// Writes `frame` over each mark at work of `panel`: a word in `accent` that is the still mark or a frame. Every
/// frame takes the bytes of the still mark, so the ranges of the marks stay where they were.
pub fn turn(panel: &mut Panel, frame: &str) {
    for m in panel.marks.iter().filter(|m| m.role == Role::Accent) {
        let Some(line) = panel.lines.get_mut(m.line) else {
            continue;
        };
        let at_work = line
            .get(m.range.clone())
            .is_some_and(|w| w == AT_WORK || FRAMES.contains(&w));
        if at_work && frame.len() == m.range.len() {
            line.replace_range(m.range.clone(), frame);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::{Mark, WAITING};

    fn panel() -> Panel {
        let mark = |line, role| Mark {
            line,
            range: 2..2 + AT_WORK.len(),
            role,
        };
        Panel {
            title: "Queue".into(),
            lines: vec![
                format!("  {AT_WORK} a"),
                format!("  {WAITING} b"),
                format!("> {AT_WORK} c"),
            ],
            marks: vec![
                mark(0, Role::Accent),
                Mark {
                    line: 1,
                    range: 2..2 + WAITING.len(),
                    role: Role::Warn,
                },
                Mark {
                    line: 2,
                    range: 0..1,
                    role: Role::Accent,
                },
                mark(2, Role::Accent),
            ],
            headings: Vec::new(),
            failed: false,
        }
    }

    // spec: SUQ-H2, SUQ-4 (internal: the screen turns the mark in a terminal, `--print` never)
    #[test]
    fn the_mark_at_work_turns_from_one_tick_to_the_next_and_the_waiting_one_stays() {
        let (first, second) = (frame(Duration::ZERO), frame(Duration::from_millis(100)));
        assert_ne!(first, second, "a frame per tick");
        assert_eq!(frame(Duration::from_millis(199)), second);
        assert_eq!(frame(Duration::from_secs(1)), first, "it turns round");
        let mut p = panel();
        turn(&mut p, first);
        assert_eq!(
            p.lines,
            [
                format!("  {first} a"),
                format!("  {WAITING} b"),
                format!("> {first} c")
            ]
        );
        turn(&mut p, second);
        assert_eq!(
            p.lines,
            [
                format!("  {second} a"),
                format!("  {WAITING} b"),
                format!("> {second} c")
            ],
            "the `>` and the waiting mark are left as they are"
        );
    }
}
