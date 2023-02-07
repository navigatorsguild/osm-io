use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use flate2::bufread::ZlibDecoder;
use prost::Message;
use crate::error::{GenericError, OsmIoError};
use crate::{osm, osmpbf};
use crate::osm::model::element::Element;
use crate::osm::pbf::file_block_metadata::FileBlockMetadata;
use crate::osm::pbf::osm_data::OsmData;
use crate::osm::pbf::osm_header::OsmHeader;
use crate::osmpbf::blob::Data;

#[derive(Debug)]
pub enum FileBlock {
    Header {
        metadata: FileBlockMetadata,
        header: OsmHeader,
    },
    Data {
        metadata: FileBlockMetadata,
        data: OsmData,
    },
}

impl FileBlock {
    pub fn new(index: usize, blob_type: String, data: Vec<u8>) -> Result<FileBlock, GenericError> {
        let blob_type_str = blob_type.as_str();
        match blob_type_str {
            "OSMHeader" => {
                Ok(
                    FileBlock::Header {
                        metadata: FileBlockMetadata::new(blob_type, index),
                        header: OsmHeader::new(data)?,
                    }
                )
            }
            "OSMData" => {
                Ok(
                    FileBlock::Data {
                        metadata: FileBlockMetadata::new(blob_type, index),
                        data: OsmData::new(index, data)?
                    }
                )
            }
            _ => {
                Err(OsmIoError::as_generic(format!("Failed to decode file block")))
            }
        }
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
                        FileBlock::zlib_decode(zlib_data, blob.raw_size.unwrap() as usize)
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

    pub fn from_blob(blob: &osm::pbf::blob::Blob) -> Result<FileBlock, GenericError> {
        let mut file = File::open(blob.path()).expect(
            format!("Failed to open {:?} for reading", blob.path()).as_str()
        );
        file.seek(SeekFrom::Start(blob.start())).expect(
            format!("Failed seek to {} in {:?} ", blob.start(), blob.path()).as_str()
        );
        let mut blob_buffer = vec![0; blob.length() as usize];
        file.read_exact(&mut blob_buffer).ok().expect(
            format!("Failed to read {} bytes from {:?} ", blob.length(), blob.path()).as_str()
        );
        let protobuf_blob = osmpbf::Blob::decode(&mut Cursor::new(blob_buffer)).expect(
            format!("Failed to decode a message from blob {} from {:?}", blob.index(), blob.path()).as_str()
        );
        let data = FileBlock::read_blob_data(protobuf_blob).unwrap();
        FileBlock::new(blob.index(), blob.t(), data)
    }

    pub fn metadata(&self) -> &FileBlockMetadata {
        match self {
            FileBlock::Header { metadata, header } => {
                metadata
            }
            FileBlock::Data { metadata, data } => {
                metadata
            }
        }
    }

    pub fn as_osm_header(&self) -> Result<&OsmHeader, GenericError> {
        match self {
            FileBlock::Header { header, .. } => {
                Ok(header)
            }
            FileBlock::Data { .. } => {
                Err(OsmIoError::as_generic(format!("Not an OSMHeader")))
            }
        }
    }

    pub fn as_osm_data(&self) -> Result<&OsmData, GenericError> {
        match self {
            FileBlock::Header { .. } => {
                Err(OsmIoError::as_generic(format!("Not an OSMData")))
            }
            FileBlock::Data { data, .. } => {
                Ok(data)
            }
        }
    }

    pub fn elements(&self) -> &Vec<Element> {
        &self.as_osm_data().unwrap().elements
    }
}
