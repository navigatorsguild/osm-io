use std::path::PathBuf;

use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::reader::Reader;
use osm_io::osm::pbf::writer::Writer;

mod common;

#[test]
fn test_pbf_rw_pipe() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init().unwrap();
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    let output_path = PathBuf::from("./tests/results/malta-230109.osm.pbf");
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf.osm.pbf.analysis.json");

    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Started OSM PBF rw pipe test, time: {stopwatch}");

    let reader = Reader::new(input_path)?;
    let mut file_info = reader.info().clone();
    file_info.with_writingprogram(&Some("rw-pipe-test-writer".to_string()));
    file_info.with_source(&Some("from fixture".to_string()));
    let mut writer = Writer::from_file_info(
        output_path.clone(),
        file_info,
        CompressionType::Zlib,
    )?;

    writer.write_header()?;
    for element in reader.elements()? {
        writer.write_element(element)?;
    }
    writer.flush()?;

    common::analyze_pbf_output(output_path, fixture_analysis_path);

    log::info!("Finished OSM PBF rw pipe test, time: {stopwatch}");
    Ok(())
}