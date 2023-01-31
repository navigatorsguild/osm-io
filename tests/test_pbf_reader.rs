use std::ops::AddAssign;
use std::path::PathBuf;
use simple_logger::SimpleLogger;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf::file_block::FileBlock;
use osm_io::osm::pbf::reader::Reader;

mod common;

#[test]
fn test_pbf_reader() {
    SimpleLogger::new().init().unwrap();
    log::info!("Started OSM PBF reader test");
    common::setup();
    let test_fixture_path = PathBuf::from("./tests/fixtures/history-malta-230109.osm.pbf");

    let reader = Reader::new(test_fixture_path).unwrap();
    let info = reader.info();

    let mut header_blocks = 0;
    let mut data_blocks = 0;
    for file_block in reader.blocks() {
        match file_block {
            FileBlock::Header { header } => {
                header_blocks.add_assign(1);
            }
            FileBlock::Data { data } => {
                data_blocks.add_assign(1);
            }
        }
    }
    assert_eq!(header_blocks, 1);
    assert!(data_blocks > 1);
    println!("headers: {header_blocks}, data blocks: {data_blocks}");

    let mut nodes = 0_i64;
    let mut ways = 0_i64;
    let mut relations = 0_i64;
    for element in reader.elements() {
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
        // println!("{i}: {:?}", element);
    }
    println!("nodes: {nodes}, ways: {ways}, relations: {relations}");

    log::info!("Finished OSM PBF reader test");
}