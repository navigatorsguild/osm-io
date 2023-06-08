use std::path::PathBuf;
use simple_logger::SimpleLogger;
use osm_io::osm::apidb_dump::read::reader::Reader;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::parallel_writer::ParallelWriter;

mod common;

#[test]
fn test_apidb_dump_reader_parallel_pbf_writer_pipe() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init()?;
    log::info!("Started apidb dump reader parallel pbf writer pipeline test");
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/history-niue-230109");
    let output_path = PathBuf::from("./target/results/history-niue-230109.osm.pbf");
    let tmp_path = PathBuf::from("./target/results/history-niue-230109");
    // TODO: rewrite the test when history import is available
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/history-niue-230109.osm.pbf.analysis.json");

    let reader = Reader::new(input_path, tmp_path)?;
    let info = FileInfo::new(
        None,
        ["OsmSchema-V0.6", "DenseNodes"].map(|s| s.to_string()).to_vec(),
        ["Sort.Type_then_ID"].map(|s| s.to_string()).to_vec(),
        Some("rw-pipe-test-writer".to_string()),
        Some("from-apidb-dump".to_string()),
        None,
        None,
        None,
    );

    let mut writer = ParallelWriter::from_file_info(
        4 * 8000 * 64,
        8000,
        output_path.clone(),
        info,
        CompressionType::Zlib,
    )?;

    writer.write_header().expect("Failed to write the pbf header");
    for element in reader.elements()? {
        writer.write_element(element)?;
    }
    writer.close()?;
    common::analyze_pbf_output(output_path, fixture_analysis_path);

    log::info!("Finished apidb dump reader parallel pbf writer pipeline test");
    Ok(())
}