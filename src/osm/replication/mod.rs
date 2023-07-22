use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use chrono::{DateTime, Duration, Utc};
use chrono::SecondsFormat;
use num_format::{Locale, ToFormattedString};

use crate::osm::converters::date_time_to_iso8601_seconds;

pub fn init_replication(
    minute_dir_path: &PathBuf,
    osmosis_replication_timestamp: DateTime<Utc>,
    osmosis_replication_base_url: reqwest::Url,
    _var_lib_path: &PathBuf,
    _var_log_path: &PathBuf,
    _verbose: bool,
) -> Result<(), anyhow::Error> {
    let minute_url = osmosis_replication_base_url.join("/replication/minute/")?;
    let starting_point_sequence_number = find_starting_point(osmosis_replication_timestamp, minute_url.clone())?;
    let (a, b, c) = split_name_components(starting_point_sequence_number);
    create_minute_dirs(&a, &b, &minute_dir_path)?;
    let mut global_state_path = minute_dir_path.clone();
    global_state_path.push("state.txt");
    let mut minute_state_path = minute_dir_path.clone();
    minute_state_path.push(&a);
    minute_state_path.push(&b);
    minute_state_path.push(&c);
    let mut minute_change_path = minute_state_path.clone();
    minute_state_path.set_extension("state.txt");
    minute_change_path.set_extension("osc.gz");

    let minute_state_url = minute_url.clone().join(format!("{}/{}/{}.state.txt", a, b, c).as_str())?;
    let minute_change_url = minute_url.clone().join(format!("{}/{}/{}.osc.gz", a, b, c).as_str())?;

    log::info!("Fetch starting point state from: {}", minute_state_url);
    let starting_point_state = fetch_text(&minute_state_url)?;
    let (starting_point_sequence_number_2, starting_point_timestamp) = parse_replication_state(&starting_point_state)?;
    log::info!("Fetched starting point sequence number: {}, timestamp: {}", starting_point_sequence_number_2, starting_point_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true));
    log::info!("Write starting point state to: {}", global_state_path.display());
    log::info!("Write starting point state to: {}", minute_state_path.display());
    fs::write(global_state_path, &starting_point_state)?;
    fs::write(minute_state_path, &starting_point_state)?;

    log::info!("Fetch starting point change: {}", minute_change_url);
    let starting_point_change = fetch_bytes(&minute_change_url)?;
    log::info!("Write starting point change to: {}", minute_change_path.display());
    fs::write(minute_change_path, starting_point_change)?;

    Ok(())
}

fn find_starting_point(osmosis_replication_timestamp: DateTime<Utc>, minute_url: reqwest::Url) -> Result<i64, anyhow::Error> {
    log::info!("Looking for replication starting point for timestamp: {}", osmosis_replication_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true));
    let current_state_url = minute_url.join("state.txt")?;
    log::info!("About to fetch current state: {}", current_state_url);
    let text = fetch_text(&current_state_url)?;
    let (mut current_sequence_number, mut current_timestamp) = parse_replication_state(&text)?;
    log::info!("Current sequence number: {}, current timestamp: {}", current_sequence_number, current_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true));

    if current_timestamp < osmosis_replication_timestamp {
        Err(anyhow!("Requested replication timestamp ({}) is greater than the replication source ({})",
            osmosis_replication_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
            current_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
        ))?;
    }

    let starting_point_timestamp = osmosis_replication_timestamp - Duration::hours(24);

    while current_timestamp > starting_point_timestamp {
        current_sequence_number -= Duration::hours(48).num_minutes();
        let (a, b, c) = split_name_components(current_sequence_number);
        let minute_state_url = minute_url.clone().join(format!("{}/{}/{}.state.txt", a, b, c).as_str())?;
        let text = fetch_text(&minute_state_url)?;
        (current_sequence_number, current_timestamp) = parse_replication_state(&text)?;
        log::info!("Seaching for starting point, sequence number: {}, timestamp: {}", current_sequence_number, current_timestamp);
    }
    log::info!("Found starting point, sequence number: {}, timestamp: {}", current_sequence_number, current_timestamp);
    Ok(current_sequence_number)
}

pub fn clean_replication(
    minute_dir_path: &PathBuf,
    _var_lib_path: &PathBuf,
    _var_log_path: &PathBuf,
    _verbose: bool,
) -> Result<(), anyhow::Error> {
    log::info!("Remove minute dir: {}", minute_dir_path.display());
    fs::remove_dir_all(&minute_dir_path)
        .or_else(|e| Err(anyhow!("Failed to remove minutes directory: {}, error: {}", minute_dir_path.display(), e)))?;
    log::info!("Removed minute dir: {}", minute_dir_path.display());
    Ok(())
}

pub fn update_replication(
    minute_dir_path: &PathBuf,
    osmosis_replication_base_url: reqwest::Url,
    _var_lib_path: &PathBuf,
    _var_log_path: &PathBuf,
    _verbose: bool,
) -> Result<(), anyhow::Error> {
    let mut global_state_path = minute_dir_path.clone();
    global_state_path.push("state.txt");
    let global_state_text = fs::read_to_string(&global_state_path)
        .or_else(|e| Err(anyhow!("Failed to read global state: {}, error: {}", global_state_path.display(), e)))?;
    let (current_sequence_number, current_timestamp) = parse_replication_state(&global_state_text)?;
    log::info!("Current global sequence number: {}, current global timestamp: {}", current_sequence_number, date_time_to_iso8601_seconds(current_timestamp));

    let minute_url = osmosis_replication_base_url.join("/replication/minute/")?;
    let remote_global_state_url = minute_url.clone().join("state.txt")?;
    let remote_global_state_text = fetch_text(&remote_global_state_url)?;
    let (remote_current_sequence_number, remote_current_timestamp) = parse_replication_state(&remote_global_state_text)?;
    log::info!("Current remote global sequence number: {}, current remote global timestamp: {}", remote_current_sequence_number, date_time_to_iso8601_seconds(remote_current_timestamp));

    let mut count = 0 as i64;
    let mut next_sequence_number = current_sequence_number + 1;
    while next_sequence_number <= remote_current_sequence_number {
        let (a, b, c) = split_name_components(next_sequence_number);
        create_minute_dirs(&a, &b, &minute_dir_path)?;
        let minute_state_url = minute_url.clone().join(format!("{}/{}/{}.state.txt", a, b, c).as_str())?;
        let minute_change_url = minute_url.clone().join(format!("{}/{}/{}.osc.gz", a, b, c).as_str())?;

        log::info!("Fetch minute state from: {}", minute_state_url);
        let minute_state_text = fetch_text(&minute_state_url)?;
        let (minute_state_sequence_number, minute_state_timestamp) = parse_replication_state(&minute_state_text)?;
        log::info!("Fetched minute state, sequence number: {}, timestamp: {}", minute_state_sequence_number, date_time_to_iso8601_seconds(minute_state_timestamp));
        let mut minute_state_path = minute_dir_path.clone();
        minute_state_path.push(&a);
        minute_state_path.push(&b);
        minute_state_path.push(&c);
        let mut minute_change_path = minute_state_path.clone();
        minute_state_path.set_extension("state.txt");
        minute_change_path.set_extension("osc.gz");
        log::info!("Write minute state to: {}", minute_state_path.display());
        fs::write(minute_state_path, &minute_state_text)?;

        log::info!("Fetch minute change: {}", minute_change_url);
        let minute_change = fetch_bytes(&minute_change_url)?;
        log::info!("Write minute change to: {}", minute_change_path.display());
        fs::write(minute_change_path, minute_change)?;

        let tmp_global_state_path = tempfile::NamedTempFile::new_in(&minute_dir_path)?;
        fs::write(&tmp_global_state_path, &minute_state_text)?;
        log::info!("Write global state to: {}", global_state_path.display());
        fs::rename(tmp_global_state_path, &global_state_path)?;

        next_sequence_number += 1;
        count += 1;
    }
    if count > 0 {
        log::info!("Downloaded {} minute updates", count.to_formatted_string(&Locale::en));
    } else {
        log::info!("Replication is up to date");
    }
    Ok(())
}

fn fetch_text(url: &reqwest::Url) -> Result<String, anyhow::Error> {
    let response = reqwest::blocking::get(url.clone())?;
    let content = match response.status().as_u16() {
        200 => {
            Ok(response.text()?)
        }
        code => {
            Err(anyhow!("URL: {}, code: {}", url, code))
        }
    }?;
    Ok(content)
}

fn fetch_bytes(url: &reqwest::Url) -> Result<Vec<u8>, anyhow::Error> {
    let response = reqwest::blocking::get(url.clone())?;
    let content = match response.status().as_u16() {
        200 => {
            Ok(response.bytes()?.to_vec())
        }
        code => {
            Err(anyhow!("URL: {}, code: {}", url, code))
        }
    }?;
    Ok(content)
}

fn split_name_components(sequence_number: i64) -> (String, String, String) {
    let mut n = sequence_number;
    let c = n - n / 1000 * 1000;
    n = n / 1000;
    let b = n - n / 1000 * 1000;
    n = n / 1000;
    let a = n - n / 1000 * 1000;
    (
        format!("{:03}", a),
        format!("{:03}", b),
        format!("{:03}", c),
    )
}

fn create_minute_dirs(a: &String, b: &String, minute_dir_path: &PathBuf) -> Result<(), anyhow::Error> {
    let mut path = minute_dir_path.clone();
    path.push(a);
    path.push(b);
    fs::create_dir_all(&path)
        .or_else(|e| Err(anyhow!("Failed to create minutes directory: {}, error: {}", path.display(), e)))?;
    Ok(())
}

fn parse_replication_state(text: &String) -> Result<(i64, DateTime<Utc>), anyhow::Error> {
    let mut sequence_number = None;
    let mut timestamp = None;

    for line in text.split("\n") {
        if line.starts_with("sequenceNumber") {
            let parts: Vec<&str> = line.split("=").map(|s| s).collect();
            sequence_number.replace(i64::from_str(parts[1])?);
        } else if line.starts_with("timestamp") {
            let parts: Vec<&str> = line.split("=").map(|s| s).collect();
            timestamp.replace(DateTime::<Utc>::from_str(parts[1].replace("\\", "").as_str())?);
        }
    }
    if sequence_number.is_some() && timestamp.is_some() {
        Ok((sequence_number.unwrap(), timestamp.unwrap()))
    } else {
        Err(anyhow!("Invalid state file format: {}", text))
    }
}
