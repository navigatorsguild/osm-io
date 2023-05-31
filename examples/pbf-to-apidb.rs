use std::path::PathBuf;

use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::apidb_dump::read::reader::Reader as ApiDbDumpReader;
use osm_io::osm::apidb_dump::write::writer::Writer as ApiDbDumpWriter;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::reader::Reader as PbfReader;
use osm_io::osm::pbf::writer::Writer as PbfWriter;

fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init().unwrap();
    let input_path = PathBuf::from("./tests/fixtures/history-malta-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/history-malta-230109");

    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Started pbf reader apidb dump writer pipeline, time: {}", stopwatch);

    let pbf_reader = PbfReader::new(input_path).unwrap();

    let mut apidb_dump_writer = ApiDbDumpWriter::new(output_path.clone(), 0)?;
    for element in pbf_reader.elements()? {
        apidb_dump_writer.write_element(element)?;
    }
    apidb_dump_writer.close()?;

    log::info!("Finished pbf reader apidb dump writer pipeline, time: {}", stopwatch);
    Ok(())
}