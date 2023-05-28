use std::ops::AddAssign;
use std::path::PathBuf;
use std::sync::{Arc};
use std::sync::atomic::{AtomicI64, Ordering};
use benchmark_rs::stopwatch::StopWatch;
use command_executor::command::Command;
use command_executor::shutdown_mode::ShutdownMode;
use command_executor::thread_pool_builder::ThreadPoolBuilder;
use simple_logger::SimpleLogger;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf::file_block::FileBlock;
use osm_io::osm::pbf::reader::Reader;
use rayon::iter::ParallelIterator;
use osm_io::osm::pbf::blob_desc::BlobDesc;

pub fn main() {
    SimpleLogger::new().init().unwrap();
    log::info!("Start profiling");
    let input_path = PathBuf::from("fixtures/germany-230109.osm.pbf");
    prof_rayon_reader(input_path.clone());
    prof_reader(input_path.clone());
    prof_command_executor_reader(input_path.clone());
    log::info!("Finish profiling");
}

fn prof_rayon_reader(input_path: PathBuf) {
    log::info!("Start profiling rayon iterator, input: {:?}", input_path);
    let mut stopwatch = StopWatch::new();
    stopwatch.start();

    let reader = Reader::new(input_path).unwrap();
    let atomic_nodes = Arc::new(AtomicI64::new(0));
    let atomic_ways = Arc::new(AtomicI64::new(0));
    let atomic_relations = Arc::new(AtomicI64::new(0));
    reader.parallel_blobs().unwrap().for_each(
        |blob_desc| {
            let mut nodes: i64 = 0;
            let mut ways: i64 = 0;
            let mut relations: i64 = 0;
            for element in FileBlock::from_blob_desc(&blob_desc).unwrap().elements() {
                match element {
                    Element::Node { node: _ } => {
                        nodes.add_assign(1)
                    }
                    Element::Way { way: _ } => {
                        ways.add_assign(1);
                    }
                    Element::Relation { relation: _ } => {
                        relations.add_assign(1);
                    }
                    Element::Sentinel => {}
                }
            }
            atomic_nodes.fetch_add(nodes, Ordering::Relaxed);
            atomic_ways.fetch_add(ways, Ordering::Relaxed);
            atomic_relations.fetch_add(relations, Ordering::Relaxed);
        }
    );
    log::info!(
        "Nodes: {}, Ways: {}, Relations: {}",
        atomic_nodes.fetch_or(0, Ordering::Relaxed),
        atomic_ways.fetch_or(0, Ordering::Relaxed),
        atomic_relations.fetch_or(0, Ordering::Relaxed),
    );
    log::info!("Rayon iterator time: {stopwatch}");
}

fn prof_reader(input_path: PathBuf) {
    log::info!("Start profiling sequential reader, input: {:?}", input_path);
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    let reader = Reader::new(input_path).unwrap();
    let mut nodes = 0_i64;
    let mut ways = 0_i64;
    let mut relations = 0_i64;
    for (_i, element) in reader.elements().unwrap().enumerate() {
        match element {
            Element::Node { node: _ } => {
                nodes.add_assign(1);
            }
            Element::Way { way: _ } => {
                ways.add_assign(1);
            }
            Element::Relation { relation: _ } => {
                relations.add_assign(1);
            }
            Element::Sentinel => {}
        }
    }
    log::info!("Nodes: {nodes}, Ways: {ways}, Relations: {relations}");
    log::info!("Sequential reader time: {stopwatch}");
}

struct CountElementsCommand {
    blob: BlobDesc,
    atomic_nodes: Arc<AtomicI64>,
    atomic_ways: Arc<AtomicI64>,
    atomic_relations: Arc<AtomicI64>,
}

impl CountElementsCommand {
    pub fn new(
        blob: BlobDesc,
        atomic_nodes: Arc<AtomicI64>,
        atomic_ways: Arc<AtomicI64>,
        atomic_relations: Arc<AtomicI64>,
    ) -> CountElementsCommand {
        CountElementsCommand {
            blob,
            atomic_nodes,
            atomic_ways,
            atomic_relations,
        }
    }
}

impl Command for CountElementsCommand {
    fn execute(&self) -> Result<(), anyhow::Error> {
        // ignore the FileBlock::Header message
        let file_block = FileBlock::from_blob_desc(&self.blob).unwrap();
        if file_block.is_osm_data() {
            let mut nodes = 0_i64;
            let mut ways = 0_i64;
            let mut relations = 0_i64;
            for element in file_block.elements() {
                match element {
                    Element::Node { .. } => {
                        nodes.add_assign(1);
                    }
                    Element::Way { .. } => {
                        ways.add_assign(1);
                    }
                    Element::Relation { .. } => {
                        relations.add_assign(1);
                    }
                    Element::Sentinel => {}
                }
            }

            self.atomic_nodes.fetch_add(nodes, Ordering::Relaxed);
            self.atomic_ways.fetch_add(ways, Ordering::Relaxed);
            self.atomic_relations.fetch_add(relations, Ordering::Relaxed);
        }
        Ok(())
    }
}

fn prof_command_executor_reader(input_path: PathBuf) {
    log::info!("Start profiling command executor reader, input: {:?}", input_path);
    let mut stopwatch = StopWatch::new();
    stopwatch.start();

    let atomic_nodes = Arc::new(AtomicI64::new(0));
    let atomic_ways = Arc::new(AtomicI64::new(0));
    let atomic_relations = Arc::new(AtomicI64::new(0));

    let mut element_counter_pool = ThreadPoolBuilder::new()
        .with_tasks(8)
        .with_queue_size(64)
        .with_name("pbf-block-decoder".to_string())
        .with_shutdown_mode(ShutdownMode::CompletePending)
        .build()
        .unwrap();

    let reader = Reader::new(input_path).unwrap();
    let mut i = 0_usize;
    for blob in reader.blobs().unwrap() {
        element_counter_pool.submit(
            Box::new(
                CountElementsCommand::new(
                    blob,
                    atomic_nodes.clone(),
                    atomic_ways.clone(),
                    atomic_relations.clone(),
                )
            )
        );
        i.add_assign(1);
    }

    element_counter_pool.shutdown();
    element_counter_pool.join().expect("Failed to join element counter pool");


    log::info!(
        "Nodes: {}, Ways: {}, Relations: {}",
        atomic_nodes.fetch_or(0, Ordering::Relaxed),
        atomic_ways.fetch_or(0, Ordering::Relaxed),
        atomic_relations.fetch_or(0, Ordering::Relaxed),
    );
    log::info!("Command executor time: {stopwatch}");
}
