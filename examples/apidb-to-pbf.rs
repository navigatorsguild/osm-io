use std::path::PathBuf;
use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;
use osm_io::osm::apidb_dump::read::reader::Reader;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::writer::Writer;

fn main() -> Result<(), anyhow::Error>{
    SimpleLogger::new().init()?;
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Started apidb dump reader pbf writer pipeline");
    let input_path = PathBuf::from("./tests/fixtures/history-malta-230109");
    let output_path = PathBuf::from("./target/results/history-malta-230109.osm.pbf");
    let tmp_path = PathBuf::from("./target/results/history-malta-230109");

    let reader = Reader::new(input_path, tmp_path)?;
    let info = FileInfo::new(
        None,
        ["OsmSchema-V0.6", "DenseNodes"].map(|s| s.to_string()).to_vec(),
        ["Sort.Type_then_ID"].map(|s| s.to_string()).to_vec(),
        Some("rw-pipe-example-writer".to_string()),
        Some("from-apidb-dump".to_string()),
        None,
        None,
        None
    );

    let mut writer = Writer::from_file_info(
        output_path.clone(),
        info,
        CompressionType::Zlib,
    )?;

    writer.write_header()?;
    for element in reader.elements()? {
        writer.write_element(element)?;
    }
    writer.close()?;

    log::info!("Finished apidb dump reader pbf writer pipeline, time: {}", stopwatch);
    Ok(())
}