use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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

    let mut nodes = Arc::new(AtomicUsize::new(0));
    let mut ways = Arc::new(AtomicUsize::new(0));
    let mut relations = Arc::new(AtomicUsize::new(0));

    let nodes_clone = nodes.clone();
    let ways_clone = ways.clone();
    let relations_clone = relations.clone();

    reader.parallel_for_each(4, move |element| {
        match element {
            Element::Node { node } => {
                nodes.fetch_add(1, Ordering::Relaxed);
            }
            Element::Way { .. } => {
                ways.fetch_add(1, Ordering::Relaxed);
            }
            Element::Relation { .. } => {
                relations.fetch_add(1, Ordering::Relaxed);
            }
            Element::Sentinel => {
                println!("Sentinel");
            }
        }
        OK(())
    }
    )?;

    println!("nodes: {}", nodes_clone.load(Ordering::Relaxed));
    println!("ways: {}", ways_clone.load(Ordering::Relaxed));
    println!("relations: {}", relations_clone.load(Ordering::Relaxed));


    // println!("nodes: {}", nodes);
    // println!("ways: {}", ways);
    // println!("relations: {}", relations);
    //
    // println!("tourism nodes: {}", tourism_nodes);
    // println!("tourism ways: {}", tourism_ways);
    // println!("tourism relations: {}", tourism_relations);

    Ok(())
}