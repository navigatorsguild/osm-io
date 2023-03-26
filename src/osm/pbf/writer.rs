use std::fs::File;
use std::io::{Write};
use std::path::PathBuf;
use crate::osm::model::bounding_box::BoundingBox;
use crate::osm::pbf::compression_type::CompressionType;
use crate::osm::pbf::file_block::FileBlock;
use crate::osm::pbf::file_info::FileInfo;
use crate::osm::pbf::osm_header::OsmHeader;

pub struct Writer {
    path: PathBuf,
    file_info: FileInfo,
    compression_type: CompressionType,
    file: File,
}

// TODO: create with a builder or add a configuration parameter or get a Fileinfo as paramter
impl Writer {
    pub fn from_file_info(
        path: PathBuf,
        file_info: FileInfo,
        compression_type: CompressionType,
    ) -> Result<Writer, anyhow::Error> {
        let file = File::create(path.clone())?;

        Ok(
            Writer {
                path: path.clone(),
                file_info,
                compression_type,
                file,
            }
        )
    }

    pub fn new(
        path: PathBuf,
        program_name: &str,
        data_source: &str,
        osmosis_replication_timestamp: Option<i64>,
        osmosis_replication_sequence_number: Option<i64>,
        osmosis_replication_base_url: Option<String>,
        compression_type: CompressionType,
        precomputed_bounding_box: Option<BoundingBox>,
        contains_history: bool,
    ) -> Result<Writer, anyhow::Error> {
        let mut required_features = vec![
            "OsmSchema-V0.6".to_string(),
            "DenseNodes".to_string(),
        ];

        if contains_history {
            required_features.push("HistoricalInformation".to_string());
        }

        let optional_features = vec![
            "Sort.Type_then_ID".to_string(),
        ];


        let writingprogram = Some(program_name.to_string());
        let source = Some(data_source.to_string());

        let file_info = FileInfo::new(
            precomputed_bounding_box,
            required_features,
            optional_features,
            writingprogram,
            source,
            osmosis_replication_timestamp,
            osmosis_replication_sequence_number,
            osmosis_replication_base_url,
        );

        Self::from_file_info(path, file_info, compression_type)
    }

    pub fn write_header(&mut self) -> Result<(), anyhow::Error> {
        let file_block = FileBlock::from_header(
            OsmHeader::from_file_info(self.file_info.clone())
        );

        self.write(file_block)
    }

    pub fn write(&mut self, file_block: FileBlock) -> Result<(), anyhow::Error> {
        let (blob_header, blob_body) = FileBlock::serialize(&file_block, self.compression_type.clone())?;
        self.write_blob(blob_header, blob_body)
    }

    pub fn write_blob(&mut self, blob_header: Vec<u8>, blob_body: Vec<u8>) -> Result<(), anyhow::Error> {
        let blob_header_len: i32 = blob_header.len() as i32;
        self.file.write(&blob_header_len.to_be_bytes())?;
        self.file.write(&blob_header)?;
        self.file.write(&blob_body)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn add_bounding_box(&mut self, bounding_box: Option<BoundingBox>) {
        self.file_info.merge_bounding_box(bounding_box);
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
