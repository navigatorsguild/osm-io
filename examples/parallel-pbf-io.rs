use std::path::PathBuf;
use std::ptr::write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
use anyhow;
use num_format::Locale::{or, pa};
use rand::thread_rng;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::element_iterator::ElementIterator;
use osm_io::osm::pbf::parallel_writer::ParallelWriter;
use osm_io::osm::pbf::thread_local_accumulator::ThreadLocalAccumulator;

pub fn main() -> Result<(), anyhow::Error> {
    // let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    // let output_path = PathBuf::from("./target/results/parallel-malta-230109.osm.pbf");
    let input_path = PathBuf::from("./tests/fixtures/germany-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/parallel-germany-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(input_path)?;
    let mut parallel_writer = Arc::new(
        Mutex::new(
            pbf::parallel_writer::ParallelWriter::from_file_info(
                4 * 8000 * 16,
                8000,
                output_path,
                reader.info().clone(),
                CompressionType::Zlib,
            )?
        )
    );
    let mut parallel_writer_clone = parallel_writer.clone();

    let tl_acc = ThreadLocalAccumulator::new(8000);

    {
        let mut parallel_writer_guard = parallel_writer.lock().unwrap();
        parallel_writer_guard.write_header()?;
    }

    reader.parallel_for_each(4, move |element| {
        let mut filter_out = false;
        match &element {
            Element::Node { node } => {
                // for tag in node.tags() {
                //     if tag.k() == "natural" && tag.v() == "tree" {
                //         filter_out = true;
                //         break;
                //     }
                // }
            }
            Element::Way { way } => {
            }
            Element::Relation { relation } => {
            }
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
    parallel_writer_guard.flush()?;
    Ok(())
}