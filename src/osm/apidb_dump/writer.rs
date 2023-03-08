use crate::error::GenericError;

pub struct Writer {}

impl Writer {
    pub fn new() -> Result<Writer, GenericError> {
        Ok(
            Writer {

            }
        )
    }
}