use std::path::PathBuf;

use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;

pub fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init()?;
    log::info!("Started count pbf elements");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(&input_path)?;

    let mut nodes = 0usize;
    let mut ways = 0usize;
    let mut relations = 0usize;

    let mut tourism_nodes = 0usize;
    let mut tourism_ways = 0usize;
    let mut tourism_relations = 0usize;

    for element in reader.elements()? {
        match element {
            Element::Node { node } => {
                nodes += 1;
                for tag in node.tags() {
                    if tag.k() == "tourism" {
                        tourism_nodes += 1;
                    }
                }
            }
            Element::Way { way } => {
                ways += 1;
                for tag in way.tags() {
                    if tag.k() == "tourism" {
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

    log::info!("nodes: {}", nodes);
    log::info!("ways: {}", ways);
    log::info!("relations: {}", relations);

    log::info!("tourism nodes: {}", tourism_nodes);
    log::info!("tourism ways: {}", tourism_ways);
    log::info!("tourism relations: {}", tourism_relations);

    log::info!("Finished count pbf elements, time: {}", stopwatch);

    Ok(())
}