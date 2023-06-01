use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use anyhow;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;

pub fn main() -> Result<(), anyhow::Error> {
    let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(input_path)?;

    let nodes = Arc::new(AtomicUsize::new(0));
    let ways = Arc::new(AtomicUsize::new(0));
    let relations = Arc::new(AtomicUsize::new(0));

    let nodes_clone = nodes.clone();
    let ways_clone = ways.clone();
    let relations_clone = relations.clone();

    reader.parallel_for_each(4, move |element| {
        match element {
            Element::Node { node: _ } => {
                nodes.fetch_add(1, Ordering::Relaxed);
            }
            Element::Way { .. } => {
                ways.fetch_add(1, Ordering::Relaxed);
            }
            Element::Relation { .. } => {
                relations.fetch_add(1, Ordering::Relaxed);
            }
            Element::Sentinel => {
            }
        }
        Ok(())
    },
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