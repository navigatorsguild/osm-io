use std::path::PathBuf;

use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::apidb_dump::write::writer::Writer as ApiDbDumpWriter;
use osm_io::osm::pbf::reader::Reader as PbfReader;

fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init()?;
    let input_path = PathBuf::from("./tests/fixtures/history-niue-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/history-niue-230109");
    // let input_path = PathBuf::from("./tests/fixtures/germany-230109.osm.pbf");
    // let output_path = PathBuf::from("./target/results/germany-230109");

    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Started pbf reader apidb dump writer pipeline");

    let pbf_reader = PbfReader::new(&input_path)?;

    let mut apidb_dump_writer = ApiDbDumpWriter::new(output_path.clone(), 0)?;
    for element in pbf_reader.elements()? {
        apidb_dump_writer.write_element(element)?;
    }
    apidb_dump_writer.close()?;

    log::info!("Finished pbf reader apidb dump writer pipeline, time: {}", stopwatch);
    Ok(())
}