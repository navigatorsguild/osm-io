use std::fs;
use std::fs::File;

use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use reqwest;
use reqwest::Url;


pub fn setup() {
    let fixture_link = Url::from_str("http://download.geofabrik.de/australia-oceania/niue-latest.osm.pbf").unwrap();
    let fixture_dir_path = PathBuf::from_str("./tests/fixtures/").unwrap();
    let results_dir_path = PathBuf::from_str("./tests/results/").unwrap();
    let parallel_results_dir_path = PathBuf::from_str("./tests/parallel-results/").unwrap();
    let fixture_file_path = fixture_dir_path.join("test.osm.pbf");
    if !fixture_file_path.exists() {
        println!("Downloading fixture file: {} -> {:?}", fixture_link, fixture_file_path);
        fs::create_dir_all(&fixture_dir_path).expect(
            format!("Failed to create fixtures directory: {:?}", fixture_dir_path).as_str()
        );
        let mut response = reqwest::blocking::get(fixture_link.clone()).expect(
            format!("Failed to download the fixture file from {:?}", fixture_link).as_str()
        );
        let mut body = Vec::new();
        response.read_to_end(&mut body).expect(
            format!("Failed to read the fixture file from {:?}", fixture_link).as_str()
        );
        let mut fixture_file = File::create(&fixture_file_path).expect(
            format!("Failed to create the fixture file: {:?}", fixture_file_path).as_str()
        );

        fixture_file.write(body.as_slice()).expect(
            format!("failed to write fixture content to {:?}", fixture_file_path).as_str()
        );
    } else {
        println!("Fixture file exists at {:?}, skipping download", fixture_file_path);
    }

    if !results_dir_path.exists() {
        fs::create_dir_all(&results_dir_path).expect(
            format!("Failed to create results directory: {:?}", results_dir_path).as_str()
        );
    } else {
        println!("Results directory exists at {:?}", results_dir_path);
    }

    if !parallel_results_dir_path.exists() {
        fs::create_dir_all(&parallel_results_dir_path).expect(
            format!("Failed to create parallel results directory: {:?}", parallel_results_dir_path).as_str()
        );
    } else {
        println!("Results directory exists at {:?}", parallel_results_dir_path);
    }
}
