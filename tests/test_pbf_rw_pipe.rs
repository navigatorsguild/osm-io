use std::path::PathBuf;
use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::reader::Reader;
use osm_io::osm::pbf::writer::Writer;

mod common;

#[test]
fn test_pbf_rw_pipe() {
    SimpleLogger::new().init().unwrap();
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    let output_path = PathBuf::from("./tests/results/malta-230109.osm.pbf");
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf.osm.pbf.analysis.json");


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
        writer.write_file_block(file_block).expect("failed to write a file block");
    }

    common::analyze_pbf_output(output_path, fixture_analysis_path);

    log::info!("Finished OSM PBF rw pipe test, time: {stopwatch}");
}