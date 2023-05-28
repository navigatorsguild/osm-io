use std::path::PathBuf;
use anyhow;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::element_iterator::ElementIterator;

pub fn main() -> Result<(), anyhow::Error> {
    let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/malta-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(input_path)?;
    let writer = pbf::writer::Writer::from_file_info(
        output_path,
        reader.info().clone(),
        CompressionType::Zlib,
    )?;

    let mut nodes = 0 as usize;
    let mut ways = 0 as usize;
    let mut relations = 0 as usize;

    let mut tourism_nodes = 0 as usize;
    let mut tourism_ways = 0 as usize;
    let mut tourism_relations = 0 as usize;

    for element in reader.elements()? {
        match element {
            Element::Node { node } => {
                nodes += 1;
                for tag in node.tags() {
                    if tag.k() == "tourism"{
                        tourism_nodes += 1;
                    }
                }
            }
            Element::Way { way } => {
                ways += 1;
                for tag in way.tags() {
                    if tag.k() == "tourism"{
                        tourism_ways += 1;
                    }
                }
            }
            Element::Relation { relation } => {
                relations += 1;
                for tag in relation.tags() {
                    if tag.k() == "tourism" {
                        tourism_relations += 1;
                    }
                }
            }
            Element::Sentinel => {
                println!("sentinel");
            }
        }
    }

    println!("nodes: {}", nodes);
    println!("ways: {}", ways);
    println!("relations: {}", relations);

    println!("tourism nodes: {}", tourism_nodes);
    println!("tourism ways: {}", tourism_ways);
    println!("tourism relations: {}", tourism_relations);

    Ok(())
}