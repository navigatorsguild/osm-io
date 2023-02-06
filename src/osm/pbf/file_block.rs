use crate::error::{GenericError, OsmIoError};
use crate::osm::pbf::osm_data::OsmData;
use crate::osm::pbf::osm_header::OsmHeader;
use crate::osmpbf::BlobHeader;

#[derive(Debug)]
pub enum FileBlock {
    Header {
        header: OsmHeader,
    },
    Data {
        data: OsmData,
    },
}

impl FileBlock {
    pub fn new(blob_type: &str, data: Vec<u8>) -> Result<FileBlock, GenericError> {
        match blob_type {
            "OSMHeader" => {
                Ok(FileBlock::Header {header: OsmHeader::new(data)?,})}
            "OSMData" => {
                Ok (FileBlock::Data {data: OsmData::new(data)?,})
            }
            _ => {
                Err(OsmIoError::as_generic(format!("Failed to decode file block")))
            }
        }
    }

    pub fn as_osm_header(&self) -> Result<&OsmHeader, GenericError> {
        match self {
            FileBlock::Header { header } => {
                Ok(header)
            }
            FileBlock::Data { .. } => {
                Err(OsmIoError::as_generic(format!("Not an OSMHeader")))
            }
        }
    }

    pub fn as_osm_data(&self) -> Result<&OsmData, GenericError> {
        match self {
            FileBlock::Header { header } => {
                Err(OsmIoError::as_generic(format!("Not an OSMData")))
            }
            FileBlock::Data { data } => {
                Ok(data)
            }
        }
    }
}
