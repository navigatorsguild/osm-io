use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::LocalKey;

use anyhow::{anyhow, Error};
use command_executor::command::Command;
use command_executor::shutdown_mode::ShutdownMode;
use command_executor::thread_pool::ThreadPool;
use command_executor::thread_pool_builder::ThreadPoolBuilder;

use crate::osm::model::element::Element;
use crate::osm::pbf::compression_type::CompressionType;
use crate::osm::pbf::file_block::FileBlock;
use crate::osm::pbf::file_info::FileInfo;
use crate::osm::pbf::writer::Writer;

thread_local! {
    // Block-based pipeline thread locals
    #[allow(clippy::type_complexity)]
    static BLOCK_ORDERING_BUFFER: RefCell<BTreeMap<usize, Vec<Element>>> = const { RefCell::new(BTreeMap::new()) };
    static BLOCK_CONSOLIDATION_ACCUMULATOR: RefCell<Vec<Element>> = const { RefCell::new(Vec::new()) };
    static OUTPUT_BLOCK_INDEX: RefCell<usize> = const { RefCell::new(1) };
    static BLOCK_ENCODING_POOL: RefCell<Option<Arc<RwLock<ThreadPool>>>> = const { RefCell::new(None) };
    static BLOCK_COMPRESSION_TYPE: RefCell<Option<CompressionType>> = const { RefCell::new(None) };
    static BLOCK_COMPRESSION_LEVEL: RefCell<Option<u32>> = const { RefCell::new(None) };
    static BLOCK_COMPRESSION_BUFFER_SIZE: RefCell<Option<usize>> = const { RefCell::new(None) };
    static BLOCK_WRITING_POOL: RefCell<Option<Arc<RwLock<ThreadPool>>>> = const { RefCell::new(None) };
    #[allow(clippy::type_complexity)]
    pub static BLOCK_BLOB_ORDERING_BUFFER: RefCell<HashMap<usize, (Vec<u8>, Vec<u8>)>> = RefCell::new(HashMap::new());
    pub static BLOCK_NEXT_TO_WRITE: RefCell<usize> = const { RefCell::new(1) };
    pub static BLOCK_PBF_WRITER: RefCell<Option<Writer>> = const { RefCell::new(None) };
}

struct AddBlockCommand {
    block_index: usize,
    elements: Mutex<Option<Vec<Element>>>,
}

impl AddBlockCommand {
    fn new(block_index: usize, elements: Vec<Element>) -> AddBlockCommand {
        AddBlockCommand {
            block_index,
            elements: Mutex::new(Some(elements)),
        }
    }
}

// noinspection DuplicatedCode
impl Command for AddBlockCommand {
    fn execute(&self) -> Result<(), Error> {
        // Unwrap the payload
        let mut elements_guard = self
            .elements
            .lock()
            .map_err(|e| anyhow!("Failed to lock elements: {}", e))?;
        let elements = elements_guard
            .take()
            .ok_or_else(|| anyhow!("Elements already taken"))?;

        // Insert into buffer
        BLOCK_ORDERING_BUFFER.with(|block_ordering_buffer| {
            block_ordering_buffer
                .borrow_mut()
                .insert(self.block_index, elements);
        });

        // Consolidate -> get top block if ready
        while let Some((output_block_id, output_elements)) = consolidate_blocks() {
            // Dispatch the block to encoding pool
            BLOCK_ENCODING_POOL.with(|encoding_pool| -> Result<(), Error> {
                let pool = encoding_pool.borrow();
                let pool_guard = pool
                    .as_ref()
                    .ok_or_else(|| anyhow!("Block encoding pool not initialized"))?
                    .read()
                    .map_err(|e| anyhow!("Failed to lock block encoding pool: {}", e))?;
                pool_guard.submit(Box::new(EncodeBlockCommand::new(
                    output_block_id,
                    Mutex::new(output_elements),
                )));
                Ok(())
            })?;
        }

        Ok(())
    }
}

struct EncodeBlockCommand {
    index: usize,
    elements: Mutex<Vec<Element>>,
}

impl EncodeBlockCommand {
    fn new(index: usize, elements: Mutex<Vec<Element>>) -> EncodeBlockCommand {
        EncodeBlockCommand { index, elements }
    }
}

impl Command for EncodeBlockCommand {
    fn execute(&self) -> Result<(), Error> {
        let mut elements_guard = self
            .elements
            .lock()
            .map_err(|e| anyhow!("Failed to lock elements: {}", e))?;

        let file_block = FileBlock::from_elements(self.index, std::mem::take(&mut elements_guard));

        let compression_type =
            BLOCK_COMPRESSION_TYPE.with(|ct| -> Result<CompressionType, Error> {
                Ok(ct
                    .borrow()
                    .as_ref()
                    .ok_or_else(|| anyhow!("Block compression type not initialized"))?
                    .clone())
            })?;
        let compression_level = BLOCK_COMPRESSION_LEVEL.with(|cl| -> Result<u32, Error> {
            Ok(*cl
                .borrow()
                .as_ref()
                .ok_or_else(|| anyhow!("Block compression level not initialized"))?)
        })?;
        let compression_buffer_size =
            BLOCK_COMPRESSION_BUFFER_SIZE.with(|cbs| -> Result<usize, Error> {
                Ok(*cbs
                    .borrow()
                    .as_ref()
                    .ok_or_else(|| anyhow!("Block compression buffer size not initialized"))?)
            })?;
        let (blob_header, blob_body) = FileBlock::serialize(
            &file_block,
            compression_type,
            compression_level,
            compression_buffer_size,
        )?;

        BLOCK_WRITING_POOL.with(|thread_pool| -> Result<(), Error> {
            let thread_pool = thread_pool.borrow();
            let thread_pool_guard = thread_pool
                .as_ref()
                .ok_or_else(|| anyhow!("Block writing pool not initialized"))?
                .read()
                .map_err(|e| anyhow!("Failed to lock block writing pool: {}", e))?;
            thread_pool_guard.submit(Box::new(WriteBlockBlobCommand::new(
                self.index,
                Mutex::new(blob_header),
                Mutex::new(blob_body),
            )));
            Ok(())
        })?;

        Ok(())
    }
}

struct WriteBlockBlobCommand {
    index: usize,
    blob_header: Mutex<Vec<u8>>,
    blob_body: Mutex<Vec<u8>>,
}

impl WriteBlockBlobCommand {
    fn new(
        index: usize,
        blob_header: Mutex<Vec<u8>>,
        blob_body: Mutex<Vec<u8>>,
    ) -> WriteBlockBlobCommand {
        WriteBlockBlobCommand {
            index,
            blob_header,
            blob_body,
        }
    }
}

// noinspection DuplicatedCode
impl Command for WriteBlockBlobCommand {
    fn execute(&self) -> Result<(), Error> {
        BLOCK_BLOB_ORDERING_BUFFER.with(|buffer| -> Result<(), Error> {
            let mut blob_header_guard = self
                .blob_header
                .lock()
                .map_err(|e| anyhow!("Failed to lock blob_header: {}", e))?;
            let blob_header = std::mem::take(blob_header_guard.deref_mut());
            let mut blob_body_guard = self
                .blob_body
                .lock()
                .map_err(|e| anyhow!("Failed to lock blob_body: {}", e))?;
            let blob_body = std::mem::take(blob_body_guard.deref_mut());
            buffer
                .borrow_mut()
                .insert(self.index, (blob_header, blob_body));
            Ok(())
        })?;

        BLOCK_BLOB_ORDERING_BUFFER.with(|buffer| -> Result<(), Error> {
            BLOCK_NEXT_TO_WRITE.with(|next| -> Result<(), Error> {
                let next_to_write = *next.borrow();
                for i in next_to_write..usize::MAX {
                    match buffer.borrow_mut().remove(&i) {
                        None => {
                            *next.borrow_mut() = i;
                            break;
                        }
                        Some((header, body)) => {
                            BLOCK_PBF_WRITER.with(|writer| -> Result<(), Error> {
                                writer
                                    .borrow_mut()
                                    .as_mut()
                                    .ok_or_else(|| anyhow!("Block PBF writer not initialized"))?
                                    .write_blob(header, body)?;
                                Ok(())
                            })?;
                        }
                    }
                }
                Ok(())
            })
        })?;

        Ok(())
    }
}

// noinspection DuplicatedCode
fn flush_all_blocks() {
    BLOCK_CONSOLIDATION_ACCUMULATOR.with(|accumulator| {
        BLOCK_ORDERING_BUFFER.with(|block_ordering_buffer| {
            let mut acc = accumulator.borrow_mut();
            let mut buffer = block_ordering_buffer.borrow_mut();

            // Drain all remaining blocks from buffer into accumulator
            while !buffer.is_empty() {
                let first_index = *buffer.keys().next().unwrap();
                let mut expected_index = first_index;
                let mut keys_to_remove = Vec::new();

                for (idx, elements) in buffer.iter_mut() {
                    if *idx != expected_index {
                        break;
                    }

                    acc.extend(elements.drain(..));
                    keys_to_remove.push(*idx);
                    expected_index += 1;
                }

                for key in keys_to_remove {
                    buffer.remove(&key);
                }
            }

            // Flush whatever is left in accumulator, even if less than 8000
            if !acc.is_empty() {
                OUTPUT_BLOCK_INDEX.with(|output_index| {
                    let block_id = *output_index.borrow();
                    *output_index.borrow_mut() += 1;

                    let output_elements = std::mem::take(&mut *acc);

                    BLOCK_ENCODING_POOL.with(|encoding_pool| {
                        let pool = encoding_pool.borrow();
                        let pool_guard = pool.as_ref().unwrap().read().unwrap();
                        pool_guard.submit(Box::new(EncodeBlockCommand::new(
                            block_id,
                            Mutex::new(output_elements),
                        )));
                    });
                });
            }
        })
    });
}

fn consolidate_blocks() -> Option<(usize, Vec<Element>)> {
    BLOCK_CONSOLIDATION_ACCUMULATOR.with(|accumulator| {
        BLOCK_ORDERING_BUFFER.with(|block_ordering_buffer| {
            let mut acc = accumulator.borrow_mut();
            let mut buffer = block_ordering_buffer.borrow_mut();

            let result = if !buffer.is_empty() {
                let first_index = *buffer.keys().next().unwrap();
                let mut expected_index = first_index;

                // Check for contiguous blocks and drain into accumulator
                let mut keys_to_remove = Vec::new();

                for (idx, elements) in buffer.iter_mut() {
                    if *idx != expected_index {
                        break; // Gap found, not contiguous
                    }

                    let space_available = 8000 - acc.len();
                    if elements.len() <= space_available {
                        // Take all elements from this block
                        acc.extend(elements.drain(..));
                        keys_to_remove.push(*idx);
                    } else {
                        // Take only what we need, leave rest in block
                        acc.extend(elements.drain(0..space_available));
                        break;
                    }

                    if acc.len() >= 8000 {
                        break;
                    }

                    expected_index += 1;
                }

                // Remove empty blocks
                for key in keys_to_remove {
                    buffer.remove(&key);
                }

                if acc.len() >= 8000 {
                    OUTPUT_BLOCK_INDEX.with(|output_index| {
                        let block_id = *output_index.borrow();
                        *output_index.borrow_mut() += 1;

                        let output_elements = std::mem::take(&mut *acc);
                        Some((block_id, output_elements))
                    })
                } else {
                    None
                }
            } else {
                None
            };

            result
        })
    })
}

/// Write *.osm.pbf file while performing concurrently significant parts of work using pre-ordered blocks.
///
/// The parallel block writer accepts pre-ordered blocks of elements from the parallel reader,
/// consolidates them into properly sized output blocks, and writes them to the target file.
/// The writer is composed of an ordering thread, multiple encoding threads for encoding and
/// compression, and a writing thread for sequential file writes.
/// For example please see ./examples/parallel-pbf-to-pbf.rs
pub struct ParallelBlockWriter {
    path: PathBuf,
    file_info: FileInfo,
    compression_type: CompressionType,
    block_ordering_pool: Arc<RwLock<ThreadPool>>,
    block_encoding_pool: Arc<RwLock<ThreadPool>>,
    block_writing_pool: Arc<RwLock<ThreadPool>>,
}

pub struct ParallelBlockWriterBuilder {
    path: Option<PathBuf>,
    file_info: Option<FileInfo>,
    compression_type: CompressionType,
    compression_level: u32,
    compression_buffer_size: usize,
    encoding_threads: usize,
}

impl ParallelBlockWriterBuilder {
    pub fn new() -> Self {
        ParallelBlockWriterBuilder {
            path: None,
            file_info: None,
            compression_type: CompressionType::Zlib,
            compression_level: 6,
            compression_buffer_size: 1024 * 1024,
            encoding_threads: 8,
        }
    }

    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn file_info(mut self, file_info: FileInfo) -> Self {
        self.file_info = Some(file_info);
        self
    }

    pub fn compression(mut self, compression_type: CompressionType) -> Self {
        self.compression_type = compression_type;
        self
    }

    pub fn encoding_threads(mut self, threads: usize) -> Self {
        self.encoding_threads = threads;
        self
    }

    pub fn compression_level(mut self, level: u32) -> Self {
        self.compression_level = level;
        self
    }

    pub fn compression_buffer_size(mut self, size: usize) -> Self {
        self.compression_buffer_size = size;
        self
    }

    pub fn build(self) -> Result<ParallelBlockWriter, Error> {
        let path = self.path.ok_or_else(|| anyhow!("path is required"))?;
        let file_info = self
            .file_info
            .ok_or_else(|| anyhow!("file_info is required"))?;

        ParallelBlockWriter::new(
            path,
            file_info,
            self.compression_type,
            self.compression_level,
            self.compression_buffer_size,
            self.encoding_threads,
        )
    }
}

// noinspection DuplicatedCode
impl Default for ParallelBlockWriterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// noinspection DuplicatedCode
impl ParallelBlockWriter {
    pub fn builder() -> ParallelBlockWriterBuilder {
        ParallelBlockWriterBuilder::new()
    }

    pub fn new(
        path: PathBuf,
        file_info: FileInfo,
        compression_type: CompressionType,
        compression_level: u32,
        compression_buffer_size: usize,
        encoding_threads: usize,
    ) -> Result<ParallelBlockWriter, Error> {
        let block_ordering_pool = Self::create_thread_pool("block-ordering", 1, 10000)?;
        let block_encoding_pool =
            Self::create_thread_pool("block-encoding", encoding_threads, 10000)?;
        let block_writing_pool = Self::create_thread_pool("block-writing", 1, 10000)?;

        Self::set_thread_local(
            block_ordering_pool.clone(),
            &BLOCK_ENCODING_POOL,
            Some(block_encoding_pool.clone()),
        )?;
        Self::set_thread_local(
            block_encoding_pool.clone(),
            &BLOCK_COMPRESSION_TYPE,
            Some(compression_type.clone()),
        )?;
        Self::set_thread_local(
            block_encoding_pool.clone(),
            &BLOCK_COMPRESSION_LEVEL,
            Some(compression_level),
        )?;
        Self::set_thread_local(
            block_encoding_pool.clone(),
            &BLOCK_COMPRESSION_BUFFER_SIZE,
            Some(compression_buffer_size),
        )?;
        Self::set_thread_local(
            block_encoding_pool.clone(),
            &BLOCK_WRITING_POOL,
            Some(block_writing_pool.clone()),
        )?;

        Ok(ParallelBlockWriter {
            path,
            file_info,
            compression_type,
            block_ordering_pool,
            block_encoding_pool,
            block_writing_pool,
        })
    }

    /// Write the *.osm.pbf header.
    ///
    /// Must be called before writing the first block.
    pub fn write_header(&mut self) -> Result<(), Error> {
        let block_writing_pool_guard = self
            .block_writing_pool
            .read()
            .map_err(|e| anyhow!("{}", e))?;
        let path = self.path.clone();
        let file_info = self.file_info.clone();
        let compression_type = self.compression_type.clone();
        block_writing_pool_guard.in_all_threads(Arc::new(move || {
            BLOCK_PBF_WRITER.with(|writer| {
                if writer.borrow().is_none() {
                    let mut w = Writer::from_file_info(
                        path.clone(),
                        file_info.clone(),
                        compression_type.clone(),
                    )
                    .expect("Failed to create PBF writer");
                    w.write_header().expect("Failed to write PBF header");
                    writer.replace(Some(w));
                }
            })
        }));

        Ok(())
    }

    /// Write an ordered block of elements with block index
    pub fn write_ordered_block(
        &mut self,
        block_index: usize,
        elements: Vec<Element>,
    ) -> Result<(), Error> {
        self.block_ordering_pool
            .read()
            .map_err(|e| anyhow!("Failed to lock block ordering pool: {}", e))?
            .submit(Box::new(AddBlockCommand::new(block_index, elements)));
        Ok(())
    }

    /// Flush internal buffers.
    pub fn close(&mut self) -> Result<(), Error> {
        self.flush_block_ordering()?;
        Self::shutdown(self.block_ordering_pool.clone())?;
        Self::shutdown(self.block_encoding_pool.clone())?;
        self.flush_block_writing();
        Self::shutdown(self.block_writing_pool.clone())?;
        Ok(())
    }

    fn flush_block_ordering(&self) -> Result<(), Error> {
        let block_ordering_pool_guard = self
            .block_ordering_pool
            .read()
            .map_err(|e| anyhow!("Failed to lock block ordering pool: {}", e))?;
        block_ordering_pool_guard.in_all_threads(Arc::new(flush_all_blocks));
        Ok(())
    }

    fn flush_block_writing(&self) {}

    fn create_thread_pool(
        name: &str,
        tasks: usize,
        queue_size: usize,
    ) -> Result<Arc<RwLock<ThreadPool>>, Error> {
        Ok(Arc::new(RwLock::new(
            ThreadPoolBuilder::new()
                .with_name_str(name)
                .with_tasks(tasks)
                .with_queue_size(queue_size)
                .with_shutdown_mode(ShutdownMode::CompletePending)
                .build()?,
        )))
    }

    fn set_thread_local<T>(
        thread_pool: Arc<RwLock<ThreadPool>>,
        local_key: &'static LocalKey<RefCell<T>>,
        val: T,
    ) -> Result<(), Error>
    where
        T: Sync + Send + Clone,
    {
        thread_pool
            .read()
            .map_err(|e| anyhow!("Failed to lock thread pool: {}", e))?
            .set_thread_local(local_key, val);
        Ok(())
    }

    fn shutdown(thread_pool: Arc<RwLock<ThreadPool>>) -> Result<(), Error> {
        let mut thread_pool = thread_pool
            .write()
            .map_err(|e| anyhow!("failed to lock tread pool: {e}"))?;
        thread_pool.shutdown();
        thread_pool.join()
    }
}
