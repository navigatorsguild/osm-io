use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::ops::{AddAssign, Deref};
use std::path::PathBuf;
use std::ptr::addr_of_mut;
use std::sync::{Arc, RwLock, MutexGuard, Mutex, Condvar};
use std::vec::IntoIter;
use prost::Message;
use crate::error::GenericError;
use crate::osm::model::element::Element;
use crate::osm::pbf::{element_iterator, file_block};
use crate::osm::pbf::file_block::FileBlock;
use crate::osm::pbf::file_block_iterator::FileBlockIterator;
use rayon::{ThreadPoolBuilder, ThreadPool};
use crate::osmpbf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};
use std::alloc::handle_alloc_error;
use std::borrow::Borrow;
use std::cmp::max;
use std::collections::{BTreeMap, HashMap};
use crate::osm::model::coordinate::Coordinate;
use crate::osm::model::node::Node;
use crate::osm::pbf::osm_data::OsmData;


pub struct ParallelElementProcessor {
    path: PathBuf,
    work_buffer_size: Option<usize>,
    tasks: Option<usize>,
}

impl ParallelElementProcessor {
    pub fn new(path: &PathBuf, work_buffer_size: Option<usize>, tasks: Option<usize>) -> Result<ParallelElementProcessor, GenericError> {
        Ok(
            ParallelElementProcessor {
                path: path.clone(),
                work_buffer_size,
                tasks,
            }
        )
    }

    fn create_thread_pool(&self) -> Arc<Option<ThreadPool>> {
        let mut reading_pool = Arc::new(None);
        if let Some(tasks) = self.tasks {
            reading_pool = Arc::new(ThreadPoolBuilder::new()
                .num_threads(tasks)
                .build()
                .ok());
        }
        reading_pool
    }


    pub fn for_each<F>(self, mut handler: F) where Self: Sized, F: FnMut(Element) {
        let reading_pool = self.create_thread_pool();
        let mut file_blocks = Arc::new(RwLock::new(HashMap::<usize, OsmData>::new()));
        let buffer_full = Arc::new((Mutex::new(false), Condvar::new()));

        let mut file = File::open(&self.path).unwrap();
        // skip header
        if let Some(header_bounds) = self.next(&mut file).ok() {
            file.seek(SeekFrom::Current(header_bounds.1 as i64)).unwrap();
        }

        let mut max_buffer_size = 128_usize;
        if let Some(work_buffer_size) = self.work_buffer_size {
            max_buffer_size = work_buffer_size;
        }
        let mut file_block_index = 0;
        while let Some(bounds) = self.next(&mut file).ok() {
            let reading_pool_ref = reading_pool.clone();
            file.seek(SeekFrom::Current(bounds.1 as i64)).unwrap();
            let path = self.path.clone();
            if let Some(readers) = reading_pool_ref.deref() {
                let mut file_blocks_ref = file_blocks.clone();
                let buffer_full_ref = buffer_full.clone();
                readers.spawn_fifo(
                    move || {
                        {
                            let data = Self::decode_file_block(path, bounds.0, bounds.1);
                            let mut w = file_blocks_ref.write().unwrap();
                            w.insert(file_block_index, data);
                            let (lock, cvar) = &*buffer_full_ref;
                            let mut full = lock.lock().unwrap();
                            *full = w.len() >= max_buffer_size;
                            cvar.notify_all();
                        }
                        let (lock, cvar) = &*buffer_full_ref;
                        let mut full = lock.lock().unwrap();
                        while *full {
                            full = cvar.wait(full).unwrap();
                        }
                    }
                );
            } else {
                let data = Self::decode_file_block(path, bounds.0, bounds.1);
                Self::process_file_block(data, &mut handler);
            }
            file_block_index.add_assign(1);
        }

        if let Some(_) = reading_pool.deref() {
            Self::process_file_blocks(max_buffer_size, &buffer_full, file_block_index, &file_blocks, &mut handler);
        }
    }

    fn decode_file_block(path: PathBuf, start: u64, len: u64) -> OsmData {
        let mut file = File::open(path).expect("Failed to open the file");
        file.seek(SeekFrom::Start(start)).expect("Failed seek to data block location");

        let mut blob_buffer = vec![0; len as usize];
        file.read_exact(&mut blob_buffer).ok().expect("Failed to read data blob from file");
        let blob = osmpbf::Blob::decode(&mut Cursor::new(blob_buffer)).expect("Failed to decode blob");
        let data = FileBlockIterator::read_blob_data(blob).unwrap();
        let file_block = FileBlock::new("OSMData", data).expect("Failed to read blob data");
        if let FileBlock::Data { data } = file_block {
            data
        } else {
            panic!("Failed to read blob data")
        }
    }

    fn next(&self, file: &mut File) -> Result<(u64, u64), GenericError> {
        let mut header_len_buffer = [0_u8; 4];
        file.read_exact(&mut header_len_buffer)?;
        let blob_header_len = i32::from_be_bytes(header_len_buffer);
        let mut blob_header_buffer = vec![0; blob_header_len as usize];
        file.read_exact(&mut blob_header_buffer)?;
        let blob_header = osmpbf::BlobHeader::decode(&mut Cursor::new(blob_header_buffer))?;
        let current_offset = file.stream_position()?;
        let next_offset = blob_header.datasize as u64;
        Ok((current_offset, next_offset))
    }

    fn process_file_block<F>(data: OsmData, handler: &mut F) where F: FnMut(Element) {
        for element in data.elements {
            handler(element);
        }
    }

    fn process_file_blocks<F>(max_buffer_size: usize, buffer_full: &Arc<(Mutex<bool>, Condvar)>, count_file_blocks: usize, file_blocks: &Arc<RwLock<HashMap<usize, OsmData>>>, handler: &mut F)
        where F: FnMut(Element) {
        for i in 0..count_file_blocks {
            loop {
                {
                    let r = file_blocks.read().expect("Failed to acquire read lock");
                    if r.contains_key(&i) {
                        break;
                    }
                }
                let sleep_time = time::Duration::from_millis(1);
                println!("waiting for {i}");
                thread::sleep(sleep_time);
            }
            let mut osm_data: Option<OsmData> = None;
            {
                let mut w = file_blocks.write().expect("Failed to acquire write lock");
                osm_data = w.remove(&i);
                let (lock, cvar) = buffer_full.deref();
                let mut full = lock.lock().unwrap();
                *full = w.len() >= max_buffer_size;
                if !*full {
                    cvar.notify_one();
                }
            }
            Self::process_file_block(osm_data.unwrap(), handler);
        }
    }
}

