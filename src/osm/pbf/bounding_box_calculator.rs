use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use command_executor::shutdown_mode::ShutdownMode;
use command_executor::thread_pool_builder::ThreadPoolBuilder;

use crate::osm::model::bounding_box::BoundingBox;
use crate::osm::pbf::calc_bounding_box_command::CalcBoundingBoxCommand;
use crate::osm::pbf::reader::Reader;

pub struct BoundingBoxCalculator {
    path: PathBuf,
}

impl BoundingBoxCalculator {
    pub fn new(path: &Path) -> BoundingBoxCalculator {
        BoundingBoxCalculator {
            path: path.to_path_buf(),
        }
    }

    pub fn calc(&self) -> Result<BoundingBox, anyhow::Error> {
        let mut tp = ThreadPoolBuilder::new()
            .with_name_str("bounding-box-calculator")
            .with_tasks(num_cpus::get())
            .with_queue_size(1024)
            .with_shutdown_mode(ShutdownMode::CompletePending)
            .build()?;

        let result = Arc::new(
            Mutex::new(
                None
            )
        );
        let reader = Reader::new(&self.path)?;
        for blob in reader.blobs()? {
            tp.submit(
                Box::new(
                    CalcBoundingBoxCommand::new(
                        blob,
                        result.clone(),
                    )
                )
            );
        }

        tp.shutdown();
        tp.join()?;
        let mut result_guard = result.lock().unwrap();
        Ok(result_guard.take().unwrap())
    }
}
