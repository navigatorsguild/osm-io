use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use simple_logger::SimpleLogger;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use benchmark_rs::stopwatch::StopWatch;
use osm_io::osm::pbf::reader::Reader;
use osm_io::osm::pbf::thread_local_accumulator::ThreadLocalAccumulator;

mod common;

#[test]
fn test_pbf_rw_parallel_pipe() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init().unwrap();
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/niue-230109.osm.pbf");
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf.analysis.json");

    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Started OSM PBF rw pipe test, time: {stopwatch}");

    let reader = Reader::new(&input_path)?;
    let mut file_info = reader.info().clone();
    file_info.with_writingprogram(&Some("parallel-rw-pipe-test-writer".to_string()));
    file_info.with_source(&Some("from fixture".to_string()));

    let parallel_writer = Arc::new(
        Mutex::new(
            pbf::parallel_writer::ParallelWriter::from_file_info(
                4 * 8000 * 32,
                8000,
                output_path.clone(),
                file_info,
                CompressionType::Zlib,
            )?
        )
    );
    let parallel_writer_clone = parallel_writer.clone();

    let tl_acc = ThreadLocalAccumulator::new(8000);

    {
        let mut parallel_writer_guard = parallel_writer.lock().unwrap();
        parallel_writer_guard.write_header()?;
    }

    reader.parallel_for_each(4, move |element| {
        if !element.is_sentinel() {
            tl_acc.add(element);
        } else {
            let mut parallel_writer_guard = parallel_writer.lock().unwrap();
            parallel_writer_guard.write_elements(tl_acc.elements())?;
        }
        Ok(())
    })?;

    let mut parallel_writer_guard = parallel_writer_clone.lock().unwrap();
    parallel_writer_guard.close()?;

    common::analyze_pbf_output(output_path, fixture_analysis_path);

    log::info!("Finished OSM PBF rw pipe test, time: {stopwatch}");
    Ok(())
}