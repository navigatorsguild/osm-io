use std::fs;
use std::fs::File;

use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use json::JsonValue;
use reqwest;
use reqwest::Url;
use osm_io::osm::pbf::reader::Reader;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use anyhow::Context;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf::file_block::FileBlock;
use rayon::iter::ParallelIterator;

pub fn setup() {
    let fixture_link = Url::from_str("http://download.geofabrik.de/australia-oceania/niue-230225.osm.pbf").unwrap();
    let fixture_dir_path = PathBuf::from_str("./tests/fixtures/").unwrap();
    let results_dir_path = PathBuf::from_str("./target/results/").unwrap();
    let parallel_results_dir_path = PathBuf::from_str("./target/results/").unwrap();
    let fixture_file_path = fixture_dir_path.join("niue-230225-geofabrik.osm.pbf");
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

pub fn read_fixture_analysis(path: &PathBuf) -> JsonValue {
    let fixture_analysis_string = fs::read_to_string(path)
        .with_context(|| format!("path: {}", path.to_string_lossy()))
        .expect("Failed to read fixture analysis file");

    let fixture_analysis = json::parse(fixture_analysis_string.as_str()).unwrap();
    fixture_analysis
}

pub fn analyze_pbf_output(output_path: PathBuf, fixture_analysis_path: PathBuf) {
    let fixture_analysis = read_fixture_analysis(&fixture_analysis_path);
    let test_reader = Reader::new(output_path).unwrap();
    let atomic_nodes = Arc::new(AtomicI64::new(0));
    let atomic_ways = Arc::new(AtomicI64::new(0));
    let atomic_relations = Arc::new(AtomicI64::new(0));
    test_reader.parallel_blobs().unwrap().for_each(
        |blob_desc| {
            for element in FileBlock::from_blob_desc(&blob_desc).unwrap().elements() {
                match element {
                    Element::Node { node: _ } => {
                        atomic_nodes.fetch_add(1, Ordering::Relaxed);
                    }
                    Element::Way { way: _ } => {
                        atomic_ways.fetch_add(1, Ordering::Relaxed);
                    }
                    Element::Relation { relation: _ } => {
                        atomic_relations.fetch_add(1, Ordering::Relaxed);
                    }
                    Element::Sentinel => {}
                }
            }
        }
    );
    assert_eq!(atomic_nodes.fetch_or(0, Ordering::Relaxed), fixture_analysis["data"]["count"]["nodes"].as_i64().unwrap());
    assert_eq!(atomic_ways.fetch_or(0, Ordering::Relaxed), fixture_analysis["data"]["count"]["ways"].as_i64().unwrap());
    assert_eq!(atomic_relations.fetch_or(0, Ordering::Relaxed), fixture_analysis["data"]["count"]["relations"].as_i64().unwrap());
}
