use chrono::prelude::*;
use chrono::{Duration, Local};

pub fn yyyymmdd_today_plus_n(n: i64) -> String {
    let today: DateTime<Local> = Local::now() + Duration::days(n);
    today.format("%Y-%m-%d").to_string()
}

pub fn days_range(start: i32, end: i32) -> Vec<String> {
    let today: DateTime<Local> = Local::now();
    let mut vec = Vec::new();
    for day in start..end {
        let dt = today + Duration::days(day.into());
        vec.push(dt.format("%Y-%m-%d").to_string());
    }
    vec
}

pub fn get_duration(log_start: &str, log_end: &str) -> String {
    let parse_from_str = NaiveTime::parse_from_str;

    let duration_string =
        parse_from_str(&log_end, "%H:%M").unwrap() - parse_from_str(&log_start, "%H:%M").unwrap();
    Duration::num_minutes(&duration_string).to_string()
}
