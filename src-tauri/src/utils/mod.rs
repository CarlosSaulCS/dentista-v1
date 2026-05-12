use chrono::{DateTime, Local, Utc};
use uuid::Uuid;

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn now_utc() -> String {
    Utc::now().to_rfc3339()
}

pub fn today_prefix() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub fn parse_rfc3339(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|date| date.with_timezone(&Utc))
}

pub fn normalize_search(value: &str) -> String {
    format!("%{}%", value.trim())
}
