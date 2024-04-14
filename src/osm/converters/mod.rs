use anyhow::{anyhow, Error};
use chrono::{DateTime, SecondsFormat};

pub fn timestamp_to_iso8601_seconds(usec: i64) -> Result<String, Error> {
    let datetime = DateTime::from_timestamp_micros(usec)
        .ok_or(anyhow!("Invalid timestamp {}", usec))?;
    Ok(datetime.to_rfc3339_opts(SecondsFormat::Secs, true))
}

#[cfg(test)]
mod tests {
    use crate::osm::converters::timestamp_to_iso8601_seconds;

    #[test]
    fn test_timestamp_to_iso8601_seconds() {
        let result = timestamp_to_iso8601_seconds(0).unwrap();
        assert_eq!(result, "1970-01-01T00:00:00Z");
    }

    #[test]
    #[should_panic]
    fn test_timestamp_to_iso8601_seconds_failure() {
        timestamp_to_iso8601_seconds(i64::MAX).expect("Invalid input");
    }
}
