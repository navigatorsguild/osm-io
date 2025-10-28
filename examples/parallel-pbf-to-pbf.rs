use benchmark_rs::stopwatch::StopWatch;
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::parallel_writer::ParallelWriter;

#[derive(Parser)]
#[command(name = "parallel-pbf-to-pbf")]
#[command(about = "Parallel PBF to PBF converter", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./tests/fixtures/niue-230109.osm.pbf")]
    input: PathBuf,

    #[arg(short, long, default_value = "./target/results/niue-230109.osm.pbf")]
    output: PathBuf,

    #[arg(short = 'r', long, default_value_t = 4)]
    reader_threads: usize,

    #[arg(short = 'w', long, default_value_t = 8)]
    encoding_threads: usize,

    #[arg(short = 'c', long, default_value = "zlib")]
    compression_type: CompressionType,

    #[arg(short = 'l', long, default_value_t = 6)]
    compression_level: u32,

    #[arg(short = 'b', long, default_value_t = 1024 * 1024)]
    compression_buffer_size: usize,

    #[arg(long, default_value_t = 1024 * 1024)]
    element_ordering_buffer_size: usize,

    #[arg(short = 'f', long, default_value_t = 8000)]
    file_block_size: usize,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    SimpleLogger::new()
        .with_module_level("text_file_sort", LevelFilter::Warn)
        .with_module_level("osm_io", LevelFilter::Warn)
        .init()?;
    log::info!("Started parallel PBF reader PBF writer pipeline");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();

    let input_path = &args.input;
    let output_path = args.output;
    let reader_threads = args.reader_threads;
    let encoding_threads = args.encoding_threads;
    let compression_type = args.compression_type;
    let compression_level = args.compression_level;
    let compression_buffer_size = args.compression_buffer_size;
    let element_ordering_buffer_size = args.element_ordering_buffer_size;
    let file_block_size = args.file_block_size;

    let reader = pbf::reader::Reader::new(input_path)?;

    let mut file_info = FileInfo::default();
    file_info.with_writingprogram_str("parallel-pbf-to-pbf");
    let mut writer = ParallelWriter::builder()
        .path(&output_path)
        .file_info(file_info)
        .compression(compression_type)
        .compression_level(compression_level)
        .compression_buffer_size(compression_buffer_size)
        .element_ordering_buffer_size(element_ordering_buffer_size)
        .file_block_size(file_block_size)
        .encoding_threads(encoding_threads)
        .build()?;
    writer.write_header()?;

    let writer = Arc::new(Mutex::new(writer));

    {
        let writer_clone = writer.clone();
        reader.parallel_for_each_ordered_block(reader_threads, move |block_index, elements| {
            let mut w = writer_clone.lock().unwrap();
            w.write_ordered_block(block_index, elements)?;
            Ok(())
        })?;
    }

    match Arc::try_unwrap(writer) {
        Ok(mutex) => {
            let mut w = mutex.into_inner().unwrap();
            w.close()?;
        }
        Err(_) => {
            panic!("Failed to unwrap Arc - still has references");
        }
    }

    log::info!(
        "Finished parallel PBF reader PBF writer pipeline, time: {}",
        stopwatch
    );
    Ok(())
}
