use std::path::PathBuf;
use anyhow;
use num_format::Locale::ur;
use osm_io::osm;
use osm_io::osm::model::bounding_box::BoundingBox;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::element_iterator::ElementIterator;
use osm_io::osm::pbf::file_info::FileInfo;

/// Display PBF file informationn
pub fn main() -> Result<(), anyhow::Error> {
    let input_path = PathBuf::from("./tests/fixtures/niue-geofabric.osm.pbf");
    let reader = pbf::reader::Reader::new(input_path.clone())?;

    let info = reader.info();
    println!("PBF file: {}", input_path.to_string_lossy());

    match info.bounding_box() {
        None => {
            println!("Bounding box: None");
        }
        Some(bb) => {
            println!("Bounding box: {}", bb);
        }
    }

    for feature in info.required_features() {
        println!("Required feature: {}", feature);
    }

    for feature in info.optional_features() {
        println!("Optional feature: {}", feature);
    }

    match info.writingprogram() {
        None => {}
        Some(program) => {
            println!("Writing program: {}", program)
        }
    }

    match info.source() {
        None => {}
        Some(source) => {
            println!("Source: {}", source)
        }
    }

    match info.osmosis_replication_timestamp() {
        None => {}
        Some(t) => {
            println!("osmosis_replication_timestamp: {}", osm::converters::timestamp_to_iso8601_seconds(t * 1e9 as i64))
        }
    }

    match info.osmosis_replication_sequence_number() {
        None => {}
        Some(n) => {
            println!("osmosis_replication_sequence_number: {}", n)
        }
    }

    match info.osmosis_replication_base_url() {
        None => {}
        Some(url) => {
            println!("osmosis_replication_base_url: {}", url)
        }
    }

    Ok(())
}