use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};

use anyhow::{anyhow, Context};
use flate2::bufread::ZlibDecoder;
use flate2::Compression;
use flate2::read::ZlibEncoder;
use prost::Message;

use crate::{osm, osmpbf};
use crate::osm::model::bounding_box::BoundingBox;
use crate::osm::model::element::Element;
use crate::osm::pbf::blob_desc::BlobDesc;
use crate::osm::pbf::compression_type::CompressionType;
use crate::osm::pbf::file_block_metadata::FileBlockMetadata;
use crate::osm::pbf::osm_data::OsmData;
use crate::osm::pbf::osm_header::OsmHeader;
use crate::osmpbf::{Blob, BlobHeader};
use crate::osmpbf::blob::Data;

/// A header or data file block in *.osm.pbf file
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
    pub(crate) fn new(index: usize, blob_type: String, data: Vec<u8>) -> Result<FileBlock, anyhow::Error> {
        let blob_type_str = blob_type.as_str();
        match blob_type_str {
            "OSMHeader" => {
                Ok(
                    FileBlock::Header {
                        metadata: FileBlockMetadata::new(blob_type, index),
                        header: OsmHeader::from_bytes(data)?,
                    }
                )
            }
            "OSMData" => {
                Ok(
                    FileBlock::Data {
                        metadata: FileBlockMetadata::new(blob_type, index),
                        data: OsmData::new(data)?,
                    }
                )
            }
            _ => {
                Err(anyhow!("Failed to decode file block"))
            }
        }
    }

    pub(crate) fn from_elements(index: usize, elements: Vec<Element>) -> FileBlock {
        FileBlock::Data {
            metadata: FileBlockMetadata::new("OSMData".to_string(), index),
            data: OsmData::from_elements( elements, None),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn compute_bounding_box(&self) -> Option<BoundingBox> {
        match self {
            FileBlock::Header { metadata: _, header } => {
                header.info().bounding_box().clone()
            }
            FileBlock::Data { metadata: _, data } => {
                data.compute_bounding_box()
            }
        }
    }

    pub(crate) fn from_header(osm_header: OsmHeader) -> FileBlock {
        FileBlock::Header {
            metadata: FileBlockMetadata::new("OSMHeader".to_string(), 0),
            header: osm_header.clone(),
        }
    }

    fn zlib_decode(data: Vec<u8>, raw_size: usize) -> Result<Vec<u8>, anyhow::Error> {
        let mut decoder = ZlibDecoder::new(data.as_slice());
        let mut decoded = vec![0_u8; raw_size];
        decoder.read_exact(&mut decoded)?;
        Ok(decoded)
    }

    fn zlib_encode(buf: Vec<u8>, compression_level: Compression) -> Result<Vec<u8>, anyhow::Error> {
        let mut encoder = ZlibEncoder::new(buf.as_slice(), compression_level);
        let mut encoded = Vec::<u8>::new();
        encoder.read_to_end(&mut encoded)?;
        Ok(encoded)
    }

    pub(crate) fn read_blob_data(blob: osmpbf::Blob) -> Result<Vec<u8>, anyhow::Error> {
        match blob.data {
            None => {
                Err(
                    anyhow!("Input file too short")
                )
            }
            Some(data) => {
                match data {
                    Data::Raw(_) => {
                        Err(
                            // TODO:
                            anyhow!("Raw data type not implemented")
                        )
                    }
                    Data::ZlibData(zlib_data) => {
                        // for now ignore that the uncompressed size is optional
                        FileBlock::zlib_decode(zlib_data, blob.raw_size.unwrap() as usize)
                    }
                    Data::LzmaData(_) => {
                        Err(
                            // TODO:
                            anyhow!("Lzma data type not implemented")
                        )
                    }
                    Data::ObsoleteBzip2Data(_) => {
                        Err(
                            anyhow!("Obsolete Bzip data type not implemented")
                        )
                    }
                    Data::Lz4Data(_) => {
                        Err(
                            // TODO:
                            anyhow!("Lz4 data type not implemented")
                        )
                    }
                    Data::ZstdData(_) => {
                        Err(
                            anyhow!("Zstd data type not implemented")
                        )
                    }
                }
            }
        }
    }

    pub(crate) fn from_blob_desc(blob_desc: &osm::pbf::blob_desc::BlobDesc) -> Result<FileBlock, anyhow::Error> {
        let mut file = File::open(blob_desc.path()).with_context(
            || anyhow!("Failed to open {:?} for reading", blob_desc.path())
        )?;
        file.seek(SeekFrom::Start(blob_desc.start())).with_context(
            || anyhow!("Failed seek to {} in {:?} ", blob_desc.start(), blob_desc.path())
        )?;
        let mut blob_buffer = vec![0; blob_desc.length() as usize];
        file.read_exact(&mut blob_buffer).ok().with_context(
            || anyhow!("Failed to read {} bytes from {:?} ", blob_desc.length(), blob_desc.path())
        )?;
        Self::deserialize(blob_desc, &mut blob_buffer)
    }

    pub(crate) fn serialize(file_block: &FileBlock, compression: CompressionType) -> Result<(Vec<u8>, Vec<u8>), anyhow::Error> {
        let (blob_type, compression_level, block_data) = match file_block {
            FileBlock::Header { metadata: _, header } => {
                ("OSMHeader".to_string(), Compression::none(), header.serialize()?)
            }
            FileBlock::Data { metadata: _, data } => {
                ("OSMData".to_string(), Compression::default(), data.serialize()?)
            }
        };

        let mut raw_size = None;
        let mut data = None;
        if block_data.len() != 0 {
            raw_size = Some(block_data.len() as i32);
            data = match compression {
                CompressionType::Uncompressed => {
                    Some(osmpbf::blob::Data::Raw(block_data))
                }
                CompressionType::Zlib => {
                    Some(osmpbf::blob::Data::ZlibData(Self::zlib_encode(block_data, compression_level)?))
                }
            };
        }

        let blob = Blob {
            raw_size,
            data,
        };

        let mut body = Vec::<u8>::with_capacity(blob.encoded_len());
        blob.encode(&mut body)?;


        let blob_header = BlobHeader {
            r#type: blob_type,
            indexdata: None,
            datasize: body.len() as i32,
        };

        let mut header = Vec::<u8>::with_capacity(blob_header.encoded_len());
        blob_header.encode(&mut header)?;

        Ok((header, body))
    }

    fn deserialize(blob_desc: &BlobDesc, blob_buffer: &mut Vec<u8>) -> Result<FileBlock, anyhow::Error> {
        // use BlobDesc rather than BlobHeader to skip reading again the blob header
        let protobuf_blob = osmpbf::Blob::decode(&mut Cursor::new(blob_buffer)).with_context(
            || anyhow!("Failed to decode a message from blob {} from {:?}", blob_desc.index(), blob_desc.path())
        )?;
        let data = FileBlock::read_blob_data(protobuf_blob)?;
        FileBlock::new(blob_desc.index(), blob_desc.t(), data)
    }

    #[allow(dead_code)]
    pub(crate) fn metadata(&self) -> &FileBlockMetadata {
        match self {
            FileBlock::Header { metadata, header: _ } => {
                metadata
            }
            FileBlock::Data { metadata, data: _ } => {
                metadata
            }
        }
    }

    pub(crate) fn as_osm_header(&self) -> Result<&OsmHeader, anyhow::Error> {
        match self {
            FileBlock::Header { header, .. } => {
                Ok(header)
            }
            FileBlock::Data { .. } => {
                Err(anyhow!("Not an OSMHeader"))
            }
        }
    }
    pub(crate) fn is_osm_header(&self) -> bool {
        match self {
            FileBlock::Header { header: _, .. } => {
                true
            }
            FileBlock::Data { .. } => {
                false
            }
        }
    }

    pub(crate) fn is_osm_data(&self) -> bool {
        !self.is_osm_header()
    }

    #[allow(dead_code)]
    pub(crate) fn as_osm_data(&self) -> Result<&OsmData, anyhow::Error> {
        match self {
            FileBlock::Header { .. } => {
                Err(anyhow!("Not an OSMData"))
            }
            FileBlock::Data { data, .. } => {
                Ok(data)
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn elements(&self) -> &Vec<Element> {
        self.as_osm_data().unwrap().elements()
    }

    pub(crate) fn take_elements(&mut self) -> Vec<Element> {
        match self {
            FileBlock::Header { .. } => {
                panic!("Not a Data variant")
            }
            FileBlock::Data { data, .. } => {
                data.take_elements()
            }
        }
    }
}

impl Default for FileBlock {
    fn default() -> Self {
        FileBlock::Data { metadata: Default::default(), data: Default::default() }
    }
}
