use std::fs;

use std::path::PathBuf;
use std::str::FromStr;
use json::JsonValue;
use osm_io::osm::pbf::reader::Reader;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use anyhow::Context;
use osm_io::osm::model::element::Element;

pub fn setup() {
    let results_dir_path = PathBuf::from_str("./target/results/").unwrap();

    if !results_dir_path.exists() {
        fs::create_dir_all(&results_dir_path).unwrap_or_else(|_| panic!("Failed to create results directory: {:?}", results_dir_path));
    } else {
        println!("Results directory exists at {:?}", results_dir_path);
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
    let test_reader = Reader::new(&output_path).unwrap();
    let atomic_nodes = Arc::new(AtomicI64::new(0));
    let atomic_nodes_clone = atomic_nodes.clone();
    let atomic_ways = Arc::new(AtomicI64::new(0));
    let atomic_ways_clone = atomic_ways.clone();
    let atomic_relations = Arc::new(AtomicI64::new(0));
    let atomic_relations_clone = atomic_relations.clone();
    test_reader.parallel_for_each(
        4,
        move |element| {
                match element {
                    Element::Node { node: _ } => {
                        atomic_nodes.fetch_add(1, Ordering::SeqCst);
                    }
                    Element::Way { way: _ } => {
                        atomic_ways.fetch_add(1, Ordering::SeqCst);
                    }
                    Element::Relation { relation: _ } => {
                        atomic_relations.fetch_add(1, Ordering::SeqCst);
                    }
                    Element::Sentinel => {}
                }
            Ok(())
        }
    ).unwrap();
    assert_eq!(atomic_nodes_clone.fetch_or(0, Ordering::SeqCst), fixture_analysis["data"]["count"]["nodes"].as_i64().unwrap());
    assert_eq!(atomic_ways_clone.fetch_or(0, Ordering::SeqCst), fixture_analysis["data"]["count"]["ways"].as_i64().unwrap());
    assert_eq!(atomic_relations_clone.fetch_or(0, Ordering::SeqCst), fixture_analysis["data"]["count"]["relations"].as_i64().unwrap());
}
