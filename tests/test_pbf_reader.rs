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
    let test_fixture_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");

    let reader = Reader::new(test_fixture_path).unwrap();
    let info = reader.info();

    let mut header_blocks = 0;
    let mut data_blocks = 0;
    for file_block in reader.blocks().unwrap() {
        match file_block {
            FileBlock::Header { metadata, header } => {
                header_blocks.add_assign(1);
            }
            FileBlock::Data { metadata, data } => {
                data_blocks.add_assign(1);
            }
        }
    }
    assert_eq!(header_blocks, 1);
    assert!(data_blocks > 1);

    let mut nodes = 0_i64;
    let mut ways = 0_i64;
    let mut relations = 0_i64;
    for (i, element) in reader.elements().unwrap().enumerate() {
        match element {
            Element::Node { node } => {
                nodes.add_assign(1);
            }
            Element::Way { way } => {
                ways.add_assign(1);
            }
            Element::Relation { relation } => {
                relations.add_assign(1);
            }
            Element::Sentinel => {}
        }
    }

    log::info!("Finished OSM PBF reader test");
}
