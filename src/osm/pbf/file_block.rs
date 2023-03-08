use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use flate2::bufread::ZlibDecoder;
use flate2::Compression;
use flate2::read::ZlibEncoder;
use prost::Message;
use crate::error::{GenericError, OsmIoError};
use crate::{osm, osmpbf};
use crate::osm::model::bounding_box::BoundingBox;
use crate::osm::model::element::Element;
use crate::osm::pbf::blob_desc::BlobDesc;
use crate::osm::pbf::compression_type::CompressionType;
use crate::osm::pbf::file_block_metadata::FileBlockMetadata;
use crate::osm::pbf::osm_data::OsmData;
use crate::osm::pbf::osm_header::OsmHeader;
use crate::osmpbf::blob::Data;
use crate::osmpbf::{Blob, BlobHeader};

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
                        data: OsmData::new(index, data)?,
                    }
                )
            }
            _ => {
                Err(OsmIoError::as_generic(format!("Failed to decode file block")))
            }
        }
    }

    pub fn compute_bounding_box(&self) -> Option<BoundingBox> {
        match self {
            FileBlock::Header { metadata: _, header } => {
                header.info().bounding_box().clone()
            }
            FileBlock::Data { metadata: _, data } => {
                data.compute_bounding_box()
            }
        }
    }

    pub fn from_header(osm_header: OsmHeader) -> FileBlock {
        FileBlock::Header {
            metadata: FileBlockMetadata::new("OSMHeader".to_string(), 0),
            header: osm_header.clone(),
        }
    }

    fn zlib_decode(data: Vec<u8>, raw_size: usize) -> Result<Vec<u8>, GenericError> {
        let mut decoder = ZlibDecoder::new(data.as_slice());
        let mut decoded = vec![0_u8; raw_size];
        decoder.read_exact(&mut decoded)?;
        Ok(decoded)
    }

    fn zlib_encode(buf: Vec<u8>, compression_level: Compression) -> Result<Vec<u8>, GenericError> {
        let mut encoder = ZlibEncoder::new(buf.as_slice(), compression_level);
        let mut encoded = Vec::<u8>::new();
        encoder.read_to_end(&mut encoded)?;
        Ok(encoded)
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
                            // TODO:
                            OsmIoError::as_generic(format!("Raw data type not implemented"))
                        )
                    }
                    Data::ZlibData(zlib_data) => {
                        // for now ignore that the uncompressed size is optional
                        FileBlock::zlib_decode(zlib_data, blob.raw_size.unwrap() as usize)
                    }
                    Data::LzmaData(_) => {
                        Err(
                            // TODO:
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
                            // TODO:
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


    pub fn from_blob_desc(blob_desc: &osm::pbf::blob_desc::BlobDesc) -> Result<FileBlock, GenericError> {
        let mut file = File::open(blob_desc.path()).expect(
            format!("Failed to open {:?} for reading", blob_desc.path()).as_str()
        );
        file.seek(SeekFrom::Start(blob_desc.start())).expect(
            format!("Failed seek to {} in {:?} ", blob_desc.start(), blob_desc.path()).as_str()
        );
        let mut blob_buffer = vec![0; blob_desc.length() as usize];
        file.read_exact(&mut blob_buffer).ok().expect(
            format!("Failed to read {} bytes from {:?} ", blob_desc.length(), blob_desc.path()).as_str()
        );
        Self::deserialize(blob_desc, &mut blob_buffer)
    }

    pub fn serialize(file_block: &FileBlock, compression: CompressionType) -> Result<(Vec<u8>, Vec<u8>), GenericError> {
        let (blob_type, compression_level, block_data) = match file_block {
            FileBlock::Header { metadata: _, header } => {
                ("OSMHeader".to_string(), Compression::none(), header.serialize().unwrap())
            }
            FileBlock::Data { metadata: _, data } => {
                ("OSMData".to_string(), Compression::default(), data.serialize().unwrap())
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

    fn deserialize(blob_desc: &BlobDesc, blob_buffer: &mut Vec<u8>) -> Result<FileBlock, GenericError> {
        // use BlobDesc rather than BlobHeader to skip reading again the blob header
        let protobuf_blob = osmpbf::Blob::decode(&mut Cursor::new(blob_buffer)).expect(
            format!("Failed to decode a message from blob {} from {:?}", blob_desc.index(), blob_desc.path()).as_str()
        );
        let data = FileBlock::read_blob_data(protobuf_blob).unwrap();
        FileBlock::new(blob_desc.index(), blob_desc.t(), data)
    }

    pub fn metadata(&self) -> &FileBlockMetadata {
        match self {
            FileBlock::Header { metadata, header: _ } => {
                metadata
            }
            FileBlock::Data { metadata, data: _ } => {
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
pub fn is_osm_header(&self) -> bool {
        match self {
            FileBlock::Header { header: _, .. } => {
                true
            }
            FileBlock::Data { .. } => {
                false
            }
        }
    }

    pub fn is_osm_data(&self) -> bool {
        !self.is_osm_header()
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
        self.as_osm_data().unwrap().elements()
    }

    pub fn release_elements(&mut self) -> Vec<Element> {
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
