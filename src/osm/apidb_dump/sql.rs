use anyhow::anyhow;
use chrono::{NaiveDateTime, ParseError, ParseResult};
use num_format::Locale::fa;

pub(crate) fn parse_sql_time(s: &str) -> Result<NaiveDateTime, ParseError>{
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
}

pub(crate) fn parse_sql_bool(s: &str) -> Result<bool, anyhow::Error> {
    match s {
        "t" => {Ok(true)}
        "f" => {Ok(false)}
        _ => {
            Err(anyhow!("Wrong boolean literal: {}", s))
        }
    }
}

pub(crate) fn parse_sql_null_string(s: &str) -> Option<String> {
    match s {
       "\\N" => {
          None
       },
        _ => {
            Some(s.to_string())
        }
    }
}