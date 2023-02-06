use std::path::PathBuf;
use std::fs::File;
use flate2::read::ZlibDecoder;
use std::io::{Cursor, Read};
use prost::Message;
use crate::error::{GenericError, OsmIoError};
use crate::osm::model::element::Element;
use crate::osm::pbf::element_iterator::ElementIterator;
use crate::osm::pbf::file_block::FileBlock;
use crate::osmpbf;
use crate::osmpbf::blob::Data;

pub struct FileBlockIterator {
    path: PathBuf,
    file: File,
}

impl FileBlockIterator {
    pub fn new(path: &PathBuf) -> Result<FileBlockIterator, GenericError> {
        let file = File::open(path)?;

        Ok(
            FileBlockIterator {
                path: path.clone(),
                file,
            }
        )
    }

    fn zlib_decode(data: Vec<u8>, raw_size: usize) -> Result<Vec<u8>, GenericError> {
        let mut decoder = ZlibDecoder::new(data.as_slice());
        let mut decoded = vec![0_u8; raw_size];
        decoder.read_exact(&mut decoded)?;
        Ok(decoded)
    }

    pub fn read_blob_data(blob: osmpbf::Blob) -> Result<Vec<u8>, GenericError> {
        match blob.data {
            None => {
                Err(
                    OsmIoError::as_generic(format!("Input file too short"))
                )
            }
            Some(data) => {
                match data {
                    Data::Raw(_) => {
                        Err(
                            OsmIoError::as_generic(format!("Raw data type not implemented"))
                        )
                    }
                    Data::ZlibData(zlib_data) => {
                        FileBlockIterator::zlib_decode(zlib_data, blob.raw_size.unwrap() as usize)
                    }
                    Data::LzmaData(_) => {
                        Err(
                            OsmIoError::as_generic(format!("Lzma data type not implemented"))
                        )
                    }
                    Data::ObsoleteBzip2Data(_) => {
                        Err(
                            OsmIoError::as_generic(format!("Obsolete Bzip data type not implemented"))
                        )
                    }
                    Data::Lz4Data(_) => {
                        Err(
                            OsmIoError::as_generic(format!("Lz4 data type not implemented"))
                        )
                    }
                    Data::ZstdData(_) => {
                        Err(
                            OsmIoError::as_generic(format!("Zstd data type not implemented"))
                        )
                    }
                }
            }
        }
    }
}

impl Iterator for FileBlockIterator {
    type Item = FileBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let mut header_len_buffer = [0_u8; 4];
        self.file.read_exact(&mut header_len_buffer).ok()?;
        let blob_header_len = i32::from_be_bytes(header_len_buffer);
        let mut blob_header_buffer = vec![0; blob_header_len as usize];
        self.file.read_exact(&mut blob_header_buffer).ok()?;
        let blob_header = osmpbf::BlobHeader::decode(&mut Cursor::new(blob_header_buffer)).ok()?;
        let mut blob_buffer = vec![0; blob_header.datasize as usize];
        self.file.read_exact(&mut blob_buffer).ok()?;
        let blob = osmpbf::Blob::decode(&mut Cursor::new(blob_buffer)).ok()?;
        let data = FileBlockIterator::read_blob_data(blob).unwrap();
        FileBlock::new(blob_header.r#type.as_str(), data).ok()
    }
}
