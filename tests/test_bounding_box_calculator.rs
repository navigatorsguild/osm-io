use std::path::PathBuf;
use std::str::FromStr;

use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::model::bounding_box::BoundingBox;
use osm_io::osm::pbf::bounding_box_calculator::BoundingBoxCalculator;

#[test]
fn test_bounding_box_calculator() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init()?;
    let mut stop_watch = StopWatch::new();
    stop_watch.start();
    log::info!("Started bounding box calculator test");
    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");

    let bbc = BoundingBoxCalculator::new(&input_path);
    let bb = bbc.calc()?;

    let expected = BoundingBox::from_str("-170.1595029, -19.3548665, -169.5647229, -18.7534559")?;
    assert_eq!(expected.left(), bb.left());
    assert_eq!(expected.bottom(), bb.bottom());
    assert_eq!(expected.right(), bb.right());
    assert_eq!(expected.top(), bb.top());

    log::info!("Calculated bounding box [{}] for {}", bb, input_path.display());
    log::info!("Finished bounding box calculator test, time: {}", stop_watch);
    Ok(())
}
