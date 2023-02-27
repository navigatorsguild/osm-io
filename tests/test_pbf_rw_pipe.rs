use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use simple_logger::SimpleLogger;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_block::FileBlock;
use osm_io::osm::pbf::reader::Reader;
use osm_io::osm::pbf::writer::Writer;
use osm_io::reporting::stopwatch::StopWatch;
use crate::common::read_fixture_analysis;
use rayon::iter::ParallelIterator;

mod common;

#[test]
fn test_pbf_rw_pipe() {
    SimpleLogger::new().init().unwrap();
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    let output_path = PathBuf::from("./tests/results/malta-230109.osm.pbf");
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf.osm.pbf.analysis.json");

    let fixture_analysis = read_fixture_analysis(&fixture_analysis_path);

    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Started OSM PBF rw pipe test, time: {stopwatch}");

    let reader = Reader::new(input_path).unwrap();

    let mut info = reader.info().clone();
    info.set_writingprogram(&Some("rw-pipe-test-writer".to_string()));
    info.set_source(&Some("from fixture".to_string()));
    let mut writer = Writer::from_file_info(
        output_path.clone(),
        info,
        CompressionType::Zlib,
    ).unwrap();

    writer.write_header().unwrap();
    let mut block_iterator = reader.blocks().unwrap();
    // skip incoming header
    let _ = block_iterator.next();
    while let Some(file_block) = block_iterator.next() {
        writer.write(file_block).expect("failed to write a file block");
    }

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

    log::info!("Finished OSM PBF rw pipe test, time: {stopwatch}");
}