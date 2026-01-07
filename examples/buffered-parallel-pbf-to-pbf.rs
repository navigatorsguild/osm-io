use benchmark_rs::stopwatch::StopWatch;
use clap::Parser;
use log::LevelFilter;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::thread_local_accumulator::ThreadLocalAccumulator;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Parser)]
#[command(name = "parallel-pbf-to-pbf")]
#[command(about = "Sequential PBF to PBF converter", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./tests/fixtures/niue-230109.osm.pbf")]
    input: PathBuf,

    #[arg(
        short,
        long,
        default_value = "./target/results/niue-230109-parallel.osm.pbf"
    )]
    output: PathBuf,
}

pub fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    SimpleLogger::new()
        .with_module_level("text_file_sort", LevelFilter::Warn)
        .with_module_level("osm_io", LevelFilter::Warn)
        .init()?;
    log::info!("Started parallel pbf reader pbf writer pipeline");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    std::fs::create_dir_all("./target/results")?;
    let input_path = &args.input;
    let output_path = args.output;
    let reader = pbf::reader::Reader::new(input_path)?;
    let mut file_info = reader.info().clone();
    file_info.with_writingprogram_str("parallel-pbf-to-pbf");
    let parallel_writer = Arc::new(RwLock::new(
        pbf::parallel_writer::ParallelWriter::from_file_info(
            8000 * 128,
            8000,
            output_path,
            file_info,
            CompressionType::Zlib,
        )?,
    ));

    let parallel_writer_clone = parallel_writer.clone();
    let tl_acc = ThreadLocalAccumulator::new(8000);

    {
        let parallel_writer_guard = parallel_writer
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to lock parallel writer: {}", e))?;
        parallel_writer_guard.write_header()?;
    }

    reader.parallel_for_each(4, move |element| {
        let mut filter_out = false;
        match &element {
            Element::Node { .. } => {}
            Element::Way { .. } => {}
            Element::Relation { .. } => {}
            Element::Sentinel => {
                filter_out = true;
                let parallel_writer_guard = parallel_writer
                    .read()
                    .map_err(|e| anyhow::anyhow!("Failed to lock parallel writer: {}", e))?;
                parallel_writer_guard.write_elements(tl_acc.elements())?;
            }
        }
        if !filter_out {
            tl_acc.add(element);
        }
        Ok(())
    })?;

    let mut parallel_writer_guard = parallel_writer_clone
        .write()
        .map_err(|e| anyhow::anyhow!("Failed to lock parallel writer: {}", e))?;
    parallel_writer_guard.close()?;

    log::info!(
        "Finished parallel pbf reader pbf writer pipeline, time: {}",
        stopwatch
    );
    Ok(())
}
