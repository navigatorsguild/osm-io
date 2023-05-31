use std::path::PathBuf;

use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::apidb_dump::read::reader::Reader as ApiDbDumpReader;
use osm_io::osm::apidb_dump::write::writer::Writer as ApiDbDumpWriter;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::reader::Reader as PbfReader;
use osm_io::osm::pbf::writer::Writer as PbfWriter;

mod common;

#[test]
fn test_pbf_reader_apidb_dump_writer_pipe() -> Result<(), anyhow::Error>{
    SimpleLogger::new().init().unwrap();
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/history-malta-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/history-malta-230109");
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/history-malta-230109.osm.pbf.analysis.json");

    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Started pbf reader apidb dump writer pipeline test, time: {}", stopwatch);

    let pbf_reader = PbfReader::new(input_path).unwrap();

    let mut apidb_dump_writer = ApiDbDumpWriter::new(output_path.clone(), 0)?;
    for element in pbf_reader.elements()? {
        apidb_dump_writer.write_element(element)?;
    }
    apidb_dump_writer.close()?;

    log::info!("Convert back to pbf for test verification, time: {}", stopwatch);

    // now convert the apidb_dump back to osm.pbf
    let input_path = output_path.clone();
    let output_path = PathBuf::from("./target/results/history-malta-230109.osm.pbf");
    let tmp_path = PathBuf::from("./target/results/history-malta-230109-tmp");
    let apidb_dump_reader = ApiDbDumpReader::new(input_path, tmp_path)?;

    let file_info = FileInfo::new(
        None,
        ["OsmSchema-V0.6", "DenseNodes"].map(|s| s.to_string()).to_vec(),
        ["Sort.Type_then_ID", "HistoricalInformation"].map(|s| s.to_string()).to_vec(),
        Some("test-writer".to_string()),
        Some("from-apidb-dump".to_string()),
        None,
        None,
        None
    );

    let mut pbf_writer = PbfWriter::from_file_info(output_path.clone(), file_info, CompressionType::Zlib)?;

    pbf_writer.write_header()?;
    for element in apidb_dump_reader.elements()? {
        pbf_writer.write_element(element)?;
    }
    pbf_writer.close()?;

    common::analyze_pbf_output(output_path, fixture_analysis_path);

    log::info!("Finished pbf reader apidb dump writer pipeline test, time: {}", stopwatch);
    Ok(())
}
