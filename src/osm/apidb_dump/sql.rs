use std::ops::{Shl, Shr};

use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime, ParseError, SecondsFormat};

pub(crate) fn parse_sql_time(s: &str) -> Result<NaiveDateTime, ParseError> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
}

pub(crate) fn parse_sql_bool(s: &str) -> Result<bool, anyhow::Error> {
    match s {
        "t" => { Ok(true) }
        "f" => { Ok(false) }
        _ => {
            Err(anyhow!("Wrong boolean literal: {}", s))
        }
    }
}

pub(crate) fn to_sql_bool(v: bool) -> char {
    match v {
        true => { 't' }
        false => { 'f' }
    }
}

pub(crate) fn parse_sql_null_string(s: &str) -> Option<String> {
    match s {
        "\\N" => {
            None
        }
        _ => {
            Some(s.to_string())
        }
    }
}

pub(crate) fn to_sql_time_millis(t: i64) -> String {
    let datetime = DateTime::from_timestamp_millis(t).unwrap();
    let sql_time: String = datetime.to_rfc3339_opts(SecondsFormat::Secs, true);
    sql_time.replace('T', " ").replace('Z', "")
}

pub(crate) fn to_sql_time_micros(t: i64) -> String {
    let datetime = DateTime::from_timestamp_micros(t).unwrap();
    let sql_time: String = datetime.to_rfc3339_opts(SecondsFormat::Micros, true);
    sql_time.replace('T', " ").replace('Z', "")
}

pub(crate) fn calculate_tile(lat: f64, lon: f64) -> u64 {
    let x = ((lon + 180.0) * 65535.0 / 360.0).round() as u64;
    let y = ((lat + 90.0) * 65535.0 / 180.0).round() as u64;

    let mut tile = 0_u64;

    for i in (0..16).rev() {
        tile = tile.shl(1) | (x.shr(i) & 1_u64);
        tile = tile.shl(1) | (y.shr(i) & 1_u64);
    }
    tile
}
