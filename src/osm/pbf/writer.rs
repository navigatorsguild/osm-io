use std::path::PathBuf;
use crate::error::GenericError;
use crate::osm::model::element::Element;

pub struct Writer {

}


impl Writer {
    pub fn new(path: PathBuf) -> Result<Writer, GenericError>{
        Ok(
            Writer {

            }
        )
    }

    pub fn write(&self, element: Element) -> Result<(), GenericError> {
        Ok(())
    }

    pub fn parallel_write(&self, block: usize, index: usize, element: Element) {
        todo!()
    }

}
