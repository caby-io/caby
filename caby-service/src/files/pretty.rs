#![feature(duration_constructors)]
use std::time::{Duration, SystemTime};

const DURATION_MINUTE: u64 = 60;
const DURATION_HOUR: u64 = 3600;
const DURATION_DAY: u64 = 86400;
const DURATION_MONTH: u64 = 2628000;
const DURATION_YEAR: u64 = 31540000;

pub fn date(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return "Unknown".to_owned();
    };

    // TODO: Handle this error
    let duration = SystemTime::now().duration_since(time).unwrap().as_secs();

    if duration < DURATION_MINUTE {
        return format!("Just now");
    }

    if duration < DURATION_HOUR {
        return format!("{:.0} minutes ago", duration / DURATION_MINUTE);
    }

    if duration < 24 * DURATION_HOUR {
        return format!("{:.0} hours ago", duration / DURATION_HOUR);
    }

    if duration < 30 * DURATION_DAY {
        return format!("{:.0} days ago", duration / DURATION_DAY);
    }

    if duration < DURATION_YEAR {
        return format!("{:.0} months ago", duration / DURATION_MONTH);
    }

    return format!("{:.0} years ago", duration / DURATION_YEAR);
}

// Credit: https://gist.github.com/maxmcd
pub fn bytes(bytes: Option<u64>) -> String {
    let Some(bytes) = bytes else {
        return "Unknown".to_owned();
    };

    let mut bf = bytes as f64;
    for suffix in vec!["", "K", "M", "G", "T", "P", "E", "Z"] {
        if bf.abs() < 1024.0 {
            return format!("{:.1} {}B", bf, suffix.to_string());
        }
        bf /= 1024.0
    }
    return return format!("{:.1} YB", bf);
}
