use std::path::PathBuf;
use simple_logger::SimpleLogger;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::reporting::stopwatch::StopWatch;
use rayon::iter::ParallelIterator;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_block::FileBlock;

pub fn main() {
    SimpleLogger::new().init().unwrap();
    log::info!("Started OSM PBF rwpipe");
    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");
    let output_path = PathBuf::from("/tmp/b.osm.pbf");
    let reader = pbf::reader::Reader::new(input_path.clone()).unwrap();
    let _writer = pbf::writer::Writer::new(
        output_path.clone(),
        "rwpip-test-writer",
        "from fixture",
        None,
        None,
        None,
        CompressionType::Uncompressed,
        true,
        None,
        true,
    ).unwrap();

    let mut stopwatch = StopWatch::new();

    log::info!("start sequential iteration over blobs for {:?}", input_path);
    stopwatch.start();
    for _blob in reader.blobs().unwrap() {
    }

    log::info!("finish sequential iteration over blobs for {:?}, time: {}", input_path, stopwatch);

    log::info!("start parallel iteration over blobs for {:?}", input_path);
    stopwatch.restart();
    reader.parallel_blobs().unwrap().for_each(
       |_blob| {
       }
    );
    log::info!("finish parallel iteration over blobs for {:?}, time: {}", input_path, stopwatch);


    log::info!("start sequential iteration over file blocks for {:?}", input_path);
    stopwatch.restart();
    for block in reader.blocks().unwrap() {
        match &block {
            FileBlock::Header { metadata: _, header: _ } => {}
            FileBlock::Data { metadata: _, data: _ } => {}
        }
    }

    log::info!("finish sequential iteration over file blocks for {:?}, time: {}", input_path, stopwatch);


    stopwatch.restart();
    log::info!("start sequential iteration over elements for {:?}", input_path);
    for element in reader.elements().unwrap() {
        match element {
            Element::Node { .. } => {}
            Element::Way { .. } => {}
            Element::Relation { .. } => {}
            Element::Sentinel => {}
        }

    }
    log::info!("finish sequential iteration over elements for {:?}, time: {}", input_path, stopwatch);

    log::info!("start parallel iteration over elements for {:?}", input_path);
    stopwatch.restart();
    reader.parallel_blobs().unwrap().for_each(
        |blob_desc| {
            println!("working on {}", blob_desc.index());
            for _element in FileBlock::from_blob_desc(&blob_desc).unwrap().elements() {

            }
        }
    );
    log::info!("finish parallel iteration over elements for {:?}, time: {}", input_path, stopwatch);



    log::info!("Finished OSM PBF rwpipe");
}