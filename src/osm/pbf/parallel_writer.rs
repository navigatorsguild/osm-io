use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ops::{AddAssign, Deref, DerefMut};
use std::path::PathBuf;
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
    static ELEMENT_ORDERING_BUFFER: RefCell<VecDeque<Element>> = RefCell::new(VecDeque::new());
    static ELEMENT_ORDERING_BUFFER_SIZE: RefCell<usize> = RefCell::new(0);
    static FILE_BLOCK_SIZE: RefCell<usize> = RefCell::new(0);
    static FILE_BLOCK_INDEX: RefCell<usize> = RefCell::new(1);
    static NEXT_THREAD_POOL: RefCell<Option<Arc<RwLock<ThreadPool>>>> = RefCell::new(None);
    static COMPRESSION_TYPE: RefCell<Option<CompressionType>> = RefCell::new(None);
    static CURRENT_MIN_ELEMENT: RefCell<Option<Element>> = RefCell::new(None);

    pub static BLOB_ORDERING_BUFFER: RefCell<HashMap<usize, (Vec<u8>, Vec<u8>)>> = RefCell::new(HashMap::new());
    // the first expected block is #1. #0 is the header
    pub static NEXT_TO_WRITE: RefCell<usize> = RefCell::new(1);
    pub static PBF_WRITER: RefCell<Option<Writer>> = RefCell::new(None);
}

fn flush_sorted_top() {
    ELEMENT_ORDERING_BUFFER.with(|element_ordering_buffer| {
        element_ordering_buffer.borrow_mut().make_contiguous().sort();
        let elements = split_file_block(element_ordering_buffer);
        set_current_min_element(elements.get(0));
        NEXT_THREAD_POOL.with(|thread_pool| {
            let thread_pool = thread_pool.borrow();
            let thread_pool_guard = thread_pool.as_ref().unwrap().read().unwrap();
            thread_pool_guard.submit(Box::new(EncodeFileBlockCommand::new(file_block_index(), Mutex::new(elements))));
            inc_file_block_index();
        })
    });
}

fn flush_all_sorted() {
    ELEMENT_ORDERING_BUFFER.with(|element_ordering_buffer| {
        element_ordering_buffer.borrow_mut().make_contiguous().sort();
        while element_ordering_buffer.borrow().len() > 0 {
            let elements = split_file_block(element_ordering_buffer);
            set_current_min_element(elements.get(0));
            NEXT_THREAD_POOL.with(|thread_pool| {
                let thread_pool = thread_pool.borrow();
                let thread_pool_guard = thread_pool.as_ref().unwrap().read().unwrap();
                thread_pool_guard.submit(Box::new(EncodeFileBlockCommand::new(file_block_index(), Mutex::new(elements))));
                inc_file_block_index();
            })
        }
    });
}

fn split_file_block(element_ordering_buffer: &RefCell<VecDeque<Element>>) -> Vec<Element> {
    let mut elements = Vec::with_capacity(file_block_size());
    for _i in 0..file_block_size() {
        let element = element_ordering_buffer.borrow_mut().pop_front();
        match element {
            None => {
                break;
            }
            Some(e) => {
                if elements.is_empty() {
                    elements.push(e);
                } else if Element::same_type(&e, &elements[0]) {
                    elements.push(e);
                } else {
                    element_ordering_buffer.borrow_mut().push_front(e);
                    break;
                }
            }
        }
    }
    elements
}

fn element_ordering_buffer_size() -> usize {
    ELEMENT_ORDERING_BUFFER_SIZE.with(|s| *s.borrow().deref())
}

fn file_block_size() -> usize {
    FILE_BLOCK_SIZE.with(|s| *s.borrow().deref())
}

fn file_block_index() -> usize {
    FILE_BLOCK_INDEX.with(|i| *i.borrow().deref())
}

fn inc_file_block_index() {
    FILE_BLOCK_INDEX.with(|i| i.borrow_mut().deref_mut().add_assign(1))
}

fn compression_type() -> CompressionType {
    COMPRESSION_TYPE.with(|compression_type| compression_type.borrow().as_ref().unwrap().clone())
}

fn assert_order(element: &Element) {
    assert!(
        compare_to_current_min_element(&element).is_ge(),
        "Element order, required by OSM PBF definition is lost. \
                    Possible cause is that the length of the ordering buffer ({}) is too short \
                    to for compensate for the loss of order caused by concurrent processing. \
                    Recommended: reader_tasks * 8000 * n",
        element_ordering_buffer_size()
    );
}

fn compare_to_current_min_element(element: &Element) -> Ordering {
    CURRENT_MIN_ELEMENT.with(|current_min_element|
        match current_min_element.borrow().deref() {
            None => {
                Ordering::Greater
            }
            Some(e) => {
                element.cmp(e)
            }
        }
    )
}

fn set_current_min_element(element: Option<&Element>) {
    CURRENT_MIN_ELEMENT.with(|current_min_element| {
        match element {
            None => {}
            Some(e) => {
                current_min_element.borrow_mut().replace(e.clone());
            }
        }
    });
}

struct AddElementCommand {
    element: Mutex<Option<Element>>,
}

impl AddElementCommand {
    fn new(element: Element) -> AddElementCommand {
        AddElementCommand {
            element: Mutex::new(Some(element)),
        }
    }
}

impl Command for AddElementCommand {
    fn execute(&self) -> Result<(), Error> {
        ELEMENT_ORDERING_BUFFER.with(|element_ordering_buffer| {
            let mut element_guard = self.element.lock().unwrap();
            assert_order(element_guard.as_ref().unwrap());
            element_ordering_buffer.borrow_mut().push_back(element_guard.take().unwrap());
            if element_ordering_buffer.borrow().len() > element_ordering_buffer_size() {
                flush_sorted_top()
            }
        });
        Ok(())
    }
}

struct AddElementsCommand {
    elements: Mutex<Option<Vec<Element>>>,
}

impl AddElementsCommand {
    fn new(elements: Vec<Element>) -> AddElementsCommand {
        AddElementsCommand {
            elements: Mutex::new(Some(elements)),
        }
    }
}

impl Command for AddElementsCommand {
    fn execute(&self) -> Result<(), Error> {
        ELEMENT_ORDERING_BUFFER.with(|element_ordering_buffer| {
            let mut elements_guard = self.elements.lock().unwrap();
            for element in elements_guard.take().unwrap() {
                assert_order(&element);
                element_ordering_buffer.borrow_mut().push_back(element);
            }
            if element_ordering_buffer.borrow().len() > element_ordering_buffer_size() {
                flush_sorted_top();
            }
        });
        Ok(())
    }
}

struct EncodeFileBlockCommand {
    index: usize,
    elements: Mutex<Vec<Element>>,
}

impl EncodeFileBlockCommand {
    fn new(index: usize, elements: Mutex<Vec<Element>>) -> EncodeFileBlockCommand {
        EncodeFileBlockCommand {
            index,
            elements,
        }
    }
}

impl Command for EncodeFileBlockCommand {
    fn execute(&self) -> Result<(), Error> {
        let mut elements_guard = self.elements.lock().unwrap();
        let file_block = FileBlock::from_elements(self.index, std::mem::take(&mut elements_guard));
        let (blob_header, blob_body) = FileBlock::serialize(&file_block, compression_type())?;
        NEXT_THREAD_POOL.with(|thread_pool| {
            let thread_pool = thread_pool.borrow();
            let thread_pool_guard = thread_pool.as_ref().unwrap().read().unwrap();
            thread_pool_guard.submit(
                Box::new(
                    WriteBlobCommand::new(
                        self.index, Mutex::new(blob_header), Mutex::new(blob_body),
                    )
                )
            );
        });
        Ok(())
    }
}

struct WriteBlobCommand {
    index: usize,
    blob_header: Mutex<Vec<u8>>,
    blob_body: Mutex<Vec<u8>>,
}

impl WriteBlobCommand {
    fn new(index: usize, blob_header: Mutex<Vec<u8>>, blob_body: Mutex<Vec<u8>>) -> WriteBlobCommand {
        WriteBlobCommand {
            index,
            blob_header,
            blob_body,
        }
    }
}

impl Command for WriteBlobCommand {
    fn execute(&self) -> Result<(), Error> {
        BLOB_ORDERING_BUFFER.with(
            |buffer| {
                let mut blob_header_guard = self.blob_header.lock().unwrap();
                let blob_header = std::mem::take(blob_header_guard.deref_mut());
                let mut blob_body_guard = self.blob_body.lock().unwrap();
                let blob_body = std::mem::take(blob_body_guard.deref_mut());
                buffer
                    .borrow_mut()
                    .insert(self.index, (blob_header, blob_body));
            }
        );

        BLOB_ORDERING_BUFFER.with(
            |buffer| {
                NEXT_TO_WRITE.with(|next| {
                    let next_to_write = *next.borrow();
                    for i in next_to_write..usize::MAX {
                        match buffer.borrow_mut().remove(&i) {
                            None => {
                                *next.borrow_mut() = i;
                                break;
                            }
                            Some((header, body)) => {
                                PBF_WRITER.with(
                                    |writer| {
                                        writer.borrow_mut().as_mut().unwrap().write_blob(header, body).expect("Failed to write a blob");
                                    }
                                );
                            }
                        }
                    }
                });
            }
        );

        Ok(())
    }
}

pub struct ParallelWriter {
    path: PathBuf,
    file_info: FileInfo,
    compression_type: CompressionType,
    element_ordering_pool: Arc<RwLock<ThreadPool>>,
    encoding_pool: Arc<RwLock<ThreadPool>>,
    writing_pool: Arc<RwLock<ThreadPool>>,
}

impl ParallelWriter {
    pub fn from_file_info(
        element_ordering_buffer_size: usize,
        file_block_size: usize,
        path: PathBuf,
        file_info: FileInfo,
        compression_type: CompressionType,
    ) -> Result<ParallelWriter, anyhow::Error> {
        let element_ordering_pool = Self::create_thread_pool("element-ordering", 1, 256)?;
        let encoding_pool = Self::create_thread_pool("encoding", 4, 256)?;
        let writing_pool = Self::create_thread_pool("writing", 1, 256)?;

        Self::set_thread_local(element_ordering_pool.clone(), &ELEMENT_ORDERING_BUFFER_SIZE, element_ordering_buffer_size);
        Self::set_thread_local(element_ordering_pool.clone(), &FILE_BLOCK_SIZE, file_block_size);
        Self::set_thread_local(encoding_pool.clone(), &COMPRESSION_TYPE, Some(compression_type.clone()));
        Self::set_thread_local(element_ordering_pool.clone(), &NEXT_THREAD_POOL, Some(encoding_pool.clone()));
        Self::set_thread_local(encoding_pool.clone(), &NEXT_THREAD_POOL, Some(writing_pool.clone()));

        Ok(
            ParallelWriter {
                path,
                file_info,
                compression_type,
                element_ordering_pool,
                encoding_pool,
                writing_pool,
            }
        )
    }

    pub fn write_header(&mut self) -> Result<(), anyhow::Error> {
        let writing_pool_guard = self.writing_pool.read()
            .map_err(|e| anyhow!("{}", e))?;
        let path = self.path.clone();
        let file_info = self.file_info.clone();
        let compression_type = self.compression_type.clone();
        writing_pool_guard.in_all_threads(
            Arc::new(move || {
                PBF_WRITER.with(|writer| {
                    if writer.borrow().is_none() {
                        let mut w = Writer::from_file_info(
                            path.clone(),
                            file_info.clone(),
                            compression_type.clone(),
                        ).unwrap();
                        w.write_header().unwrap();
                        writer.replace(Some(w));
                    }
                })
            })
        );
        Ok(())
    }

    pub fn write_element(&mut self, element: Element) -> Result<(), anyhow::Error> {
        self.element_ordering_pool
            .read()
            .unwrap()
            .submit(Box::new(AddElementCommand::new(element)));
        Ok(())
    }

    pub fn write_elements(&mut self, elements: Vec<Element>) -> Result<(), anyhow::Error> {
        self.element_ordering_pool
            .read()
            .unwrap()
            .submit(Box::new(AddElementsCommand::new(elements)));
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), anyhow::Error> {
        self.flush_element_ordering();
        Self::shutdown(self.element_ordering_pool.clone())?;
        Self::shutdown(self.encoding_pool.clone())?;
        self.flush_writing();
        Self::shutdown(self.writing_pool.clone())?;
        Ok(())
    }

    fn flush_element_ordering(&self) {
        let element_ordering_pool_guard = self.element_ordering_pool.read().unwrap();
        element_ordering_pool_guard.in_all_threads(Arc::new(|| flush_all_sorted()))
    }

    fn flush_writing(&self) {}

    fn create_thread_pool(name: &str, tasks: usize, queue_size: usize) -> Result<Arc<RwLock<ThreadPool>>, anyhow::Error> {
        Ok(
            Arc::new(
                RwLock::new(
                    ThreadPoolBuilder::new()
                        .with_name_str(name)
                        .with_tasks(tasks)
                        .with_queue_size(queue_size)
                        .with_shutdown_mode(ShutdownMode::CompletePending)
                        .build()?
                )
            )
        )
    }

    fn set_thread_local<T>(thread_pool: Arc<RwLock<ThreadPool>>, local_key: &'static LocalKey<RefCell<T>>, val: T)
        where T: Sync + Send + Clone {
        thread_pool
            .read()
            .unwrap()
            .set_thread_local(local_key, val);
    }

    fn shutdown(thread_pool: Arc<RwLock<ThreadPool>>) -> Result<(), anyhow::Error> {
        let mut thread_pool = thread_pool
            .write()
            .map_err(|e| anyhow!("failed to lock tread pool: {e}"))?;
        thread_pool.shutdown();
        thread_pool.join()
    }
}