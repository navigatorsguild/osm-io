use std::ops::AddAssign;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use simple_logger::SimpleLogger;

use osm_io::osm::model::element::Element;
use osm_io::osm::pbf::file_block::FileBlock;
use osm_io::osm::pbf::reader::Reader;

use crate::common::read_fixture_analysis;

#[allow(dead_code)]
mod common;

#[test]
fn test_pbf_reader() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init()?;
    log::info!("Started OSM PBF reader test");
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf.analysis.json");

    let fixture_analysis = read_fixture_analysis(&fixture_analysis_path);

    let reader = Reader::new(&input_path)?;

    let mut header_blocks = 0;
    let mut data_blocks = 0;
    for file_block in reader.blocks()? {
        match file_block {
            FileBlock::Header { metadata: _, header: _ } => {
                header_blocks.add_assign(1);
            }
            FileBlock::Data { metadata: _, data: _ } => {
                data_blocks.add_assign(1);
            }
        }
    }
    assert_eq!(header_blocks, 1);
    assert!(data_blocks > 1);

    let mut nodes = 0_i64;
    let mut ways = 0_i64;
    let mut relations = 0_i64;
    for (_i, element) in reader.elements()?.enumerate() {
        match element {
            Element::Node { node: _ } => {
                nodes.add_assign(1);
            }
            Element::Way { way: _ } => {
                ways.add_assign(1);
            }
            Element::Relation { relation: _ } => {
                relations.add_assign(1);
            }
            Element::Sentinel => {}
        }
    }

    assert_eq!(nodes, fixture_analysis["data"]["count"]["nodes"].as_i64().unwrap());
    assert_eq!(ways, fixture_analysis["data"]["count"]["ways"].as_i64().unwrap());
    assert_eq!(relations, fixture_analysis["data"]["count"]["relations"].as_i64().unwrap());

    let atomic_nodes = Arc::new(AtomicI64::new(0));
    let atomic_nodes_clone = atomic_nodes.clone();
    let atomic_ways = Arc::new(AtomicI64::new(0));
    let atomic_ways_clone = atomic_ways.clone();
    let atomic_relations = Arc::new(AtomicI64::new(0));
    let atomic_relations_clone = atomic_relations.clone();
    reader.parallel_for_each(
        4,
        move |element| {
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
            Ok(())
        },
    )?;
    assert_eq!(atomic_nodes_clone.fetch_or(0, Ordering::Relaxed), fixture_analysis["data"]["count"]["nodes"].as_i64().unwrap());
    assert_eq!(atomic_ways_clone.fetch_or(0, Ordering::Relaxed), fixture_analysis["data"]["count"]["ways"].as_i64().unwrap());
    assert_eq!(atomic_relations_clone.fetch_or(0, Ordering::Relaxed), fixture_analysis["data"]["count"]["relations"].as_i64().unwrap());


    log::info!("Finished OSM PBF reader test");
    Ok(())
}
