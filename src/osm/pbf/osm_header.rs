use std::io::Cursor;
use prost::Message;
use crate::error::{GenericError};
use crate::osm::model::bounding_box::BoundingBox;
use crate::osm::model::file_info::FileInfo;
use crate::osmpbf;

#[derive(Clone, Debug)]
pub struct OsmHeader {
    info: FileInfo,
}

impl OsmHeader {
    pub fn new(data: Vec<u8>) -> Result<OsmHeader, GenericError> {
        let header_block = osmpbf::HeaderBlock::decode(&mut Cursor::new(data))?;
        let mut bounding_box = BoundingBox::new(-180.0,180.0,90.0,-90.0);
        if let Some(bbox) = header_block.bbox {
            let nanodeg = 1_000_000_000_f64;
            bounding_box = BoundingBox::new(
                bbox.left as f64 / nanodeg,
                bbox.right as f64 / nanodeg,
                bbox.top as f64 / nanodeg,
                bbox.bottom as f64 / nanodeg,
            )
        }

        let required_features = header_block.required_features;
        let optional_features = header_block.optional_features;
        let writingprogram = header_block.writingprogram;
        let source = header_block.source;
        let osmosis_replication_timestamp = header_block.osmosis_replication_timestamp;
        let osmosis_replication_sequence_number = header_block.osmosis_replication_sequence_number;
        let osmosis_replication_base_url = header_block.osmosis_replication_base_url;

        let info = FileInfo::new(
            bounding_box,
            required_features,
            optional_features,
            writingprogram,
            source,
            osmosis_replication_timestamp,
            osmosis_replication_sequence_number,
            osmosis_replication_base_url
        );

        Ok(
            OsmHeader {
                info,
            }
        )
    }

    pub fn info(&self) -> &FileInfo {
        &self.info
    }
}
