use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::ops::AddAssign;
use std::path::PathBuf;
use prost::Message;
use crate::error::GenericError;
use crate::{osm, osmpbf};

pub struct BlobIterator {
    path: PathBuf,
    file: File,
    jump: u64,
    index: usize,
}

impl BlobIterator {
    pub fn new(path: PathBuf) -> Result<BlobIterator, GenericError> {
        let file = File::open(path.clone())?;
        Ok(
            BlobIterator {
                path: path.clone(),
                file,
                jump: 0,
                index: 0,
            }
        )
    }
}

impl Iterator for BlobIterator {
    type Item = osm::pbf::blob_desc::BlobDesc;

    fn next(&mut self) -> Option<Self::Item> {
        self.file.seek(SeekFrom::Current(self.jump as i64)).ok()?;
        let mut header_len_buffer = [0_u8; 4];
        self.file.read_exact(&mut header_len_buffer).ok()?;
        let blob_header_len = i32::from_be_bytes(header_len_buffer);
        let mut blob_header_buffer = vec![0; blob_header_len as usize];
        self.file.read_exact(&mut blob_header_buffer).ok()?;
        let blob_header = osmpbf::BlobHeader::decode(&mut Cursor::new(blob_header_buffer)).ok()?;
        let current_offset = self.file.stream_position().ok()?;
        let length = blob_header.datasize as u64;
        self.jump = length;
        let index = self.index;
        self.index.add_assign(1);
        Some(
            osm::pbf::blob_desc::BlobDesc::new(self.path.clone(), index, current_offset, length, blob_header.r#type)
        )
    }
}