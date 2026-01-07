use std::sync::Arc;

use anyhow::Error;
use command_executor::command::Command;

use crate::osm::model::element::Element;
use crate::osm::pbf::blob_desc::BlobDesc;
use crate::osm::pbf::file_block::FileBlock;

pub(crate) struct ParallelBlockIterationCommand {
    blob_desc: BlobDesc,
    f: Arc<dyn Fn(usize, Vec<Element>) -> Result<(), Error> + Send + Sync + 'static>,
}

impl ParallelBlockIterationCommand {
    pub(crate) fn new(
        blob_desc: BlobDesc,
        f: Arc<impl Fn(usize, Vec<Element>) -> Result<(), Error> + Send + Sync + 'static>,
    ) -> ParallelBlockIterationCommand {
        ParallelBlockIterationCommand { blob_desc, f }
    }
}

impl Command for ParallelBlockIterationCommand {
    fn execute(&self) -> Result<(), Error> {
        let mut file_block = FileBlock::from_blob_desc(&self.blob_desc)?;
        if file_block.is_osm_data() {
            let elements: Vec<Element> = file_block.take_elements();
            (self.f)(self.blob_desc.index(), elements)?;
        }
        Ok(())
    }
}
