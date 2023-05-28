use std::fs::File;

use anyhow::Error;
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};

use crate::osm::apidb_dump::apidb_dump_block::ApidbDumpBlock;
use crate::osm::pbf::file_block::FileBlock;
use crate::osm::pbf::file_block_metadata::FileBlockMetadata;
use crate::osm::pbf::osm_data::OsmData;

impl Into<FileBlock> for ApidbDumpBlock {
    fn into(mut self) -> FileBlock {
        FileBlock::Data {
            metadata: FileBlockMetadata::new(
                "OSMData".to_string(),
                self.index(),
            ),
            data: OsmData::from_elements(
                self.index(),
                self.take_elements(),
                None,
            ),
        }
    }
}

impl Into<ApidbDumpBlock> for FileBlock {
    fn into(mut self) -> ApidbDumpBlock {
        ApidbDumpBlock::new(self.metadata().index(), self.take_elements())
    }
}

pub fn timestamp_to_iso8601_seconds(nsec: i64) -> String {
    let datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(nsec / (1e9 as i64), (nsec % (1e9 as i64)) as u32).unwrap(), Utc);
    datetime.to_rfc3339_opts(SecondsFormat::Secs, true)
}