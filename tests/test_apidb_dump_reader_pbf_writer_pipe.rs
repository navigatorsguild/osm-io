use std::path::PathBuf;
use simple_logger::SimpleLogger;
use osm_io::osm::apidb_dump::reader::Reader;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::writer::Writer;

mod common;

#[test]
fn test_apidb_dump_reader_pbf_writer_pipe() {
    SimpleLogger::new().init().unwrap();
    log::info!("Started apidb dump reader test");
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/malta-230109-modified-history");
    let output_path = PathBuf::from("./tests/results/malta-230109-modified-history.osm.pbf");
    let tmp_path = PathBuf::from("./tests/results/malta-230109-modified-history");
    // the malta-230109-modified-history.osm.pbf.analysis.json was created from test results because
    // for now there is no way to import into DB a complete history from PBF..
    // TODO: rewrite the test when history import is available
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/malta-230109-modified-history.osm.pbf.analysis.json");


    let reader = Reader::new(input_path, tmp_path).unwrap();
    let info = FileInfo::new(
        None,
        ["OsmSchema-V0.6", "DenseNodes"].map(|s| s.to_string()).to_vec(),
        ["Sort.Type_then_ID"].map(|s| s.to_string()).to_vec(),
        Some("rw-pipe-test-writer".to_string()),
        Some("from-apidb-dump".to_string()),
        None,
        None,
        None
    );

    let mut writer = Writer::from_file_info(
        output_path.clone(),
        info,
        CompressionType::Zlib,
    ).unwrap();

    writer.write_header().expect("Failed to write the pbf header");
    for block in reader.blocks().unwrap() {
        writer.write(block.into()).expect("failed to write a file block");
    }

    common::analyze_pbf_output(output_path, fixture_analysis_path);

    // TODO: test with history file, check elements in db relations are matched by version and ID

    log::info!("Finished apidb dump reader test");
}