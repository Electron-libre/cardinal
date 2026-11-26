//! The local time of a wait, a measure or a limit: `HH:MM`, its date before it when it is not today (STU-13, STU-37).

/// The local time `HH:MM` of `seconds` since the Unix epoch, its date before it when it is not today (STU-13).
pub fn clock(seconds: i64) -> String {
    let Some(t) = chrono::DateTime::from_timestamp(seconds, 0) else {
        return "?".to_string();
    };
    let t = t.with_timezone(&chrono::Local);
    let format = if t.date_naive() == chrono::Local::now().date_naive() {
        "%H:%M"
    } else {
        "%Y-%m-%d %H:%M"
    };
    t.format(format).to_string()
}
