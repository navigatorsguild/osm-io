use std::ops::AddAssign;
use std::path::PathBuf;
use simple_logger::SimpleLogger;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_block::FileBlock;
use osm_io::osm::pbf::reader::Reader;
use osm_io::osm::pbf::writer::Writer;
use osm_io::reporting::stopwatch::StopWatch;

mod common;

#[test]
fn test_pbf_rw_pipe() {
    SimpleLogger::new().init().unwrap();
    common::setup();
    let test_fixture_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    let test_output_path = PathBuf::from("./tests/results/malta-230109.osm.pbf");


    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Started OSM PBF rw pipe test, time: {stopwatch}");

    let reader = Reader::new(test_fixture_path).unwrap();

    let mut info = reader.info().clone();
    info.writingprogram = Some("rw-pipe-test-writer".to_string());
    info.source = Some("from fixture".to_string());
    let mut writer = Writer::from_file_info(
        test_output_path,
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

    log::info!("Finished OSM PBF rw pipe test, time: {stopwatch}");
}