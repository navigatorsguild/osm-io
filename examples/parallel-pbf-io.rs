use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow;
use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::thread_local_accumulator::ThreadLocalAccumulator;

pub fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init()?;
    log::info!("Started parallel pbf io pipeline");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/parallel-malta-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(input_path)?;
    let mut file_info = reader.info().clone();
    file_info.with_writingprogram_str("parallel-pbf-io-example");
    let parallel_writer = Arc::new(
        Mutex::new(
            pbf::parallel_writer::ParallelWriter::from_file_info(
                4 * 8000 * 32,
                8000,
                output_path,
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
        let mut filter_out = false;
        match &element {
            Element::Node { node } => {
                for tag in node.tags() {
                    if tag.k() == "natural" && tag.v() == "tree" {
                        filter_out = true;
                        break;
                    }
                }
            }
            Element::Way { way: _ } => {}
            Element::Relation { relation: _ } => {}
            Element::Sentinel => {
                filter_out = true;
                let mut parallel_writer_guard = parallel_writer.lock().unwrap();
                parallel_writer_guard.write_elements(tl_acc.elements())?;
            }
        }

        if !filter_out {
            tl_acc.add(element);
        }
        Ok(())
    })?;

    let mut parallel_writer_guard = parallel_writer_clone.lock().unwrap();
    parallel_writer_guard.close()?;
    log::info!("Finished parallel pbf io pipeline, time: {}", stopwatch);
    Ok(())
}