use std::fs::File;
use crate::osm::apidb_dump::apidb_dump_block::ApidbDumpBlock;
use crate::osm::pbf::file_block::FileBlock;
use crate::osm::pbf::file_block_metadata::FileBlockMetadata;
use crate::osm::pbf::osm_data::OsmData;

impl Into<FileBlock> for ApidbDumpBlock {
    fn into(mut self) -> FileBlock {
        FileBlock::Data {
            metadata: FileBlockMetadata::new(
                "OSMData".to_string(),
                self.index()
            ),
            data: OsmData::from_elements(
                self.index(),
                self.take_elements(),
                None
            )
        }
    }
}