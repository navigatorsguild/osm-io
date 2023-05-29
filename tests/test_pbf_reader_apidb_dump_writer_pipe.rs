use std::path::PathBuf;
use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;
use osm_io::osm::apidb_dump::write::writer::Writer;
use osm_io::osm::pbf::reader::Reader;

mod common;

// #[test]
fn test_pbf_reader_apidb_dump_writer_pipe() -> Result<(), anyhow::Error>{
    SimpleLogger::new().init().unwrap();
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/history-malta-230109.osm.pbf");
    let output_path = PathBuf::from("./tests/results/history-malta-230109");
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/history-malta-230109.osm.pbf.analysis.json");

    let reader = Reader::new(input_path).unwrap();
    let mut block_iterator = reader.blocks().unwrap();
    // skip incoming header
    let _ = block_iterator.next();

    let mut writer = Writer::new(output_path, 0)?;
    while let Some(file_block) = block_iterator.next() {
        writer.write(file_block.into()).expect("failed to write a file block");
    }
    writer.close()?;


    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Started pbf reader apidb dump writer pipeline test, time: {}", stopwatch);



    // common::analyze_pbf_output(output_path, fixture_analysis_path);

    log::info!("Finished pbf reader apidb dump writer pipeline test, time: {}", stopwatch);
    Ok(())
}
