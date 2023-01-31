pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(thiserror::Error, Debug)]
#[error("{message:}")]
pub struct OsmIoError {
    pub message: String,
}

impl OsmIoError {
    pub fn new(message: String) -> OsmIoError {
        OsmIoError {
            message
        }
    }

    pub fn as_generic(message: String) -> GenericError {
        GenericError::from(OsmIoError::new(message))
    }
}
