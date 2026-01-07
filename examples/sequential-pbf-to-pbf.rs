use std::path::PathBuf;

use benchmark_rs::stopwatch::StopWatch;
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;

#[derive(Parser)]
#[command(name = "sequential-pbf-to-pbf")]
#[command(about = "Sequential PBF to PBF converter", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./tests/fixtures/niue-230109.osm.pbf")]
    input: PathBuf,

    #[arg(
        short,
        long,
        default_value = "./target/results/niue-230109-sequential.osm.pbf"
    )]
    output: PathBuf,
}

pub fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    SimpleLogger::new()
        .with_module_level("text_file_sort", LevelFilter::Warn)
        .with_module_level("osm_io", LevelFilter::Warn)
        .init()?;
    log::info!("Started sequential pbf reader pbf writer pipeline");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    std::fs::create_dir_all("./target/results")?;
    let input_path = &args.input;
    let output_path = args.output;
    let reader = pbf::reader::Reader::new(input_path)?;
    let mut file_info = reader.info().clone();
    file_info.with_writingprogram_str("sequential-pbf-to-pbf");
    let mut writer =
        pbf::writer::Writer::from_file_info(output_path, file_info, CompressionType::Zlib)?;

    writer.write_header()?;

    for element in reader.elements()? {
        writer.write_element(element)?;
    }

    writer.close()?;

    log::info!(
        "Finished sequential pbf reader pbf writer pipeline, time: {}",
        stopwatch
    );
    Ok(())
}
