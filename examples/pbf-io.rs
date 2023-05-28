use std::path::PathBuf;
use std::ptr::write;
use std::sync::{Arc, Mutex};
use std::thread;
use anyhow;
use num_format::Locale::or;
use rand::thread_rng;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::element_iterator::ElementIterator;
use osm_io::osm::pbf::parallel_writer::ParallelWriter;
use osm_io::osm::pbf::thread_local_accumulator::ThreadLocalAccumulator;

pub fn main() -> Result<(), anyhow::Error> {
    // let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    // let output_path = PathBuf::from("./target/results/malta-230109.osm.pbf");
    let input_path = PathBuf::from("./tests/fixtures/germany-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/germany-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(input_path)?;
    let mut writer = pbf::writer::Writer::from_file_info(
        output_path,
        reader.info().clone(),
        CompressionType::Zlib,
    )?;

    writer.write_header()?;

    for element in reader.elements()? {
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
            }
        }
        if !filter_out {
            writer.write_element(element)?;
        }
    }

    writer.flush()?;

    Ok(())
}