use anyhow::anyhow;
use benchmark_rs::stopwatch::StopWatch;
use clap::{arg, Parser};
use log::LevelFilter;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::parallel_block_writer::ParallelBlockWriter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    SimpleLogger::new()
        .with_module_level("text_file_sort", LevelFilter::Warn)
        .with_module_level("osm_io", LevelFilter::Warn)
        .init()?;
    log::info!("Started parallel pbf reader pbf writer pipeline");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    let input_path = &args.input;
    let output_path = args.output;

    let reader = pbf::reader::Reader::new(&input_path)?;

    let mut file_info = FileInfo::default();
    file_info.with_writingprogram_str("parallel-pbf-to-pbf");
    let mut writer = ParallelBlockWriter::new(
        output_path,
        file_info,
        CompressionType::Zlib,
        6,
        1024 * 1024,
        8,
    )?;
    writer.write_header()?;

    let writer = Arc::new(Mutex::new(writer));

    {
        let writer_clone = writer.clone();
        reader.parallel_for_each_ordered_block(4, move |block_index, elements| {
            let mut w = match writer_clone.lock() {
                Ok(x) => x,
                Err(_) => return Err(anyhow!("Writer lock poisoned")),
            };
            w.write_ordered_block(block_index, elements)?;
            Ok(())
        })?;
    }

    match Arc::try_unwrap(writer) {
        Ok(mutex) => {
            let mut w = mutex.into_inner()?;
            w.close()?;
        }
        Err(_) => {
            panic!("Failed to unwrap Arc - still has references");
        }
    }

    log::info!(
        "Finished parallel pbf reader pbf writer pipeline, time: {}",
        stopwatch
    );
    Ok(())
}
