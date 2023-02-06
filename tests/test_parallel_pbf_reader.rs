use std::ops::AddAssign;
use std::path::PathBuf;
use simple_logger::SimpleLogger;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf::file_block::FileBlock;
use osm_io::osm::pbf::reader::Reader;
use osm_io::osm::pbf::element_iterator::ElementIterator;
use Iterator;

mod common;

#[test]
fn test_parallel_pbf_reader() {
    log::info!("Started parallel OSM PBF reader test");
    common::setup();
    let test_fixture_path = PathBuf::from("./tests/fixtures/germany-230109.osm.pbf");

    let reader = Reader::new(test_fixture_path).unwrap();
    let info = reader.info();
    let mut nodes = 0_i64;
    let mut ways = 0_i64;
    let mut relations = 0_i64;
    let parallel_element_processor = reader.parallel_elements(
        Some(128),
        Some(6)
    );
    parallel_element_processor.for_each(
        |element: Element| {
            match element {
                Element::Node { node } => {
                    // println!("{node:?}");
                    nodes.add_assign(1);
                }
                Element::Way { way } => {
                    ways.add_assign(1);
                }
                Element::Relation { relation } => {
                    relations.add_assign(1);
                }
            }
        }
    );
    println!("nodes: {nodes}, ways: {ways}, relations: {relations}");
    log::info!("Finished OSM PBF reader test");
}
