use std::sync::{Arc, Mutex};

use command_executor::command::Command;

use crate::osm::model::bounding_box::BoundingBox;
use crate::osm::model::element::Element;
use crate::osm::pbf::blob_desc::BlobDesc;
use crate::osm::pbf::file_block::FileBlock;

pub(crate) struct CalcBoundingBoxCommand {
    blob: BlobDesc,
    result: Arc<Mutex<Option<BoundingBox>>>,
}

impl CalcBoundingBoxCommand {
    pub(crate) fn new(
        blob: BlobDesc,
        result: Arc<Mutex<Option<BoundingBox>>>,
    ) -> CalcBoundingBoxCommand {
        CalcBoundingBoxCommand {
            blob,
            result,
        }
    }
}

impl Command for CalcBoundingBoxCommand {
    #[allow(clippy::unnecessary_unwrap)]
    fn execute(&self) -> Result<(), anyhow::Error> {
        let file_block = FileBlock::from_blob_desc(&self.blob)?;
        if file_block.is_osm_data() {
            let mut bounding_box = None;
            for element in file_block.elements() {
                match element {
                    Element::Node { node } => {
                        if bounding_box.is_none() {
                            bounding_box.replace(BoundingBox::from_point(node.coordinate()));
                        } else {
                            bounding_box.as_mut().unwrap().merge_point(node.coordinate());
                        }
                    }
                    _ => {
                        break;
                    }
                }
            }
            if bounding_box.is_some() {
                let mut result_guard = self.result.lock().unwrap();
                if result_guard.is_none() {
                    result_guard.replace(bounding_box.unwrap());
                } else {
                    result_guard.as_mut().unwrap().merge_bounding_box(&bounding_box.unwrap());
                }
            }
        }
        Ok(())
    }
}
