use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow;
use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;

pub fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init()?;
    log::info!("Started parallel count pbf elements");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(&input_path)?;

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
            Element::Sentinel => {}
        }
        Ok(())
    },
    )?;

    log::info!("nodes: {}", nodes_clone.load(Ordering::Relaxed));
    log::info!("ways: {}", ways_clone.load(Ordering::Relaxed));
    log::info!("relations: {}", relations_clone.load(Ordering::Relaxed));

    log::info!("Finished parallel count pbf elements, time: {}", stopwatch);
    Ok(())
}