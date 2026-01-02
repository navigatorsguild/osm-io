use benchmark_rs::stopwatch::StopWatch;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::parallel_block_writer::ParallelBlockWriter;

fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new()
        .with_module_level("text_file_sort", LevelFilter::Warn)
        .with_module_level("osm_io", LevelFilter::Warn)
        .init()?;
    log::info!("Started parallel pbf reader pbf writer pipeline");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/niue-230109.osm.pbf");

    let reader = pbf::reader::Reader::new(&input_path)?;

    let mut file_info = FileInfo::default();
    file_info.with_writingprogram_str("parallel-pbf-to-pbf");
    let mut writer = ParallelBlockWriter::new(
        output_path,
        file_info,
        CompressionType::Zlib,
        6,
        1024 * 1024,
        4,
    )?;
    writer.write_header()?;

    let writer = Arc::new(Mutex::new(writer));

    {
        let writer_clone = writer.clone();
        reader.parallel_for_each_ordered_block(4, move |block_index, elements| {
            let mut w = match writer_clone.lock() {
                Ok(x) => x,
                Err(_) => todo!(),
            };
            w.write_ordered_block(block_index, elements)?;
            Ok(())
        })?;
    }

    match Arc::try_unwrap(writer) {
        Ok(mutex) => {
            let mut w = mutex.into_inner()?;
            w.close()?;
        }
        Err(_) => {
            panic!("Failed to unwrap Arc - still has references");
        }
    }

    log::info!(
        "Finished parallel pbf reader pbf writer pipeline, time: {}",
        stopwatch
    );
    Ok(())
}
