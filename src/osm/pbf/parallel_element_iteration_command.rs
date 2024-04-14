use std::sync::Arc;

use anyhow::Error;
use command_executor::command::Command;

use crate::osm::model::element::Element;
use crate::osm::pbf::blob_desc::BlobDesc;
use crate::osm::pbf::file_block::FileBlock;

pub(crate) struct ParallelElementIterationCommand {
    blob_desc: BlobDesc,
    f: Arc<dyn Fn(Element) -> Result<(), Error> + Send + Sync + 'static>,
}

impl ParallelElementIterationCommand {
    pub(crate) fn new(blob_desc: BlobDesc, f: Arc<impl Fn(Element) -> Result<(), Error> + Send + Sync + 'static>) -> ParallelElementIterationCommand {
        ParallelElementIterationCommand {
            blob_desc,
            f,
        }
    }
}

impl Command for ParallelElementIterationCommand {
    fn execute(&self) -> Result<(), Error> {
        let mut file_block = FileBlock::from_blob_desc(&self.blob_desc)?;
        if file_block.is_osm_data() {
            for element in file_block.take_elements() {
                (self.f)(element)?;
            }
            (self.f)(Element::Sentinel)?;
        }
        Ok(())
    }
}