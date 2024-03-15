use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};

pub fn timestamp_to_iso8601_seconds(usec: i64) -> String {
    let datetime = DateTime::<Utc>::from_naive_utc_and_offset(
        NaiveDateTime::from_timestamp_micros(usec).unwrap(),
        Utc);
    datetime.to_rfc3339_opts(SecondsFormat::Secs, true)
}
