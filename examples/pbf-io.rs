use std::path::PathBuf;

use anyhow;
use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;

pub fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init()?;
    log::info!("Started pbf io pipeline");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/malta-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(input_path)?;
    let mut file_info = FileInfo::default();
    file_info.with_writingprogram_str("pbf-io-example");
    let mut writer = pbf::writer::Writer::from_file_info(
        output_path,
        file_info,
        CompressionType::Zlib,
    )?;

    writer.write_header()?;

    for element in reader.elements()? {
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
            }
        }
        if !filter_out {
            writer.write_element(element)?;
        }
    }

    writer.close()?;

    log::info!("Finished pbf io pipeline, time: {}", stopwatch);
    Ok(())
}