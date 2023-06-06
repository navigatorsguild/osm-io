use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};

pub fn timestamp_to_iso8601_seconds(nsec: i64) -> String {
    let datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(nsec / (1e9 as i64), (nsec % (1e9 as i64)) as u32).unwrap(), Utc);
    datetime.to_rfc3339_opts(SecondsFormat::Secs, true)
}