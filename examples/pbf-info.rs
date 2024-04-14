use std::path::PathBuf;

use osm_io::osm;
use osm_io::osm::pbf;

/// Display PBF file information
pub fn main() -> Result<(), anyhow::Error> {
    let input_path = PathBuf::from("./tests/fixtures/history-niue-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(&input_path)?;

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
            println!(
                "osmosis_replication_timestamp: {}",
                osm::converters::timestamp_to_iso8601_seconds(t * 1e6 as i64)?
            )
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

    println!("Feature queries");
    println!("Required 'HistoricalInformation': {}", info.required("HistoricalInformation"));
    println!("Required 'DenseNodes': {}", info.required("DenseNodes"));
    println!("Optional 'Sort.Type_then_ID': {}", info.optional("Sort.Type_then_ID"));

    assert!(info.required("HistoricalInformation"));
    assert!(info.required("DenseNodes"));
    assert!(info.optional("Sort.Type_then_ID"));

    Ok(())
}