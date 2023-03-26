use std::collections::HashMap;
use std::ops::{AddAssign, DerefMut};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use command_executor::errors::GenericError;
use command_executor::executor::Command;
use command_executor::executor::ShutdownMode;
use command_executor::executor::ThreadPool;
use command_executor::executor::ThreadPoolBuilder;
use simple_logger::SimpleLogger;
use osm_io::osm::pbf;
use osm_io::osm::pbf::blob_desc::BlobDesc;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_block::FileBlock;
use osm_io::osm::pbf::file_block_metadata::FileBlockMetadata;
use osm_io::reporting::stopwatch::StopWatch;
use std::cell::{RefCell};
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::writer::Writer;

mod common;

thread_local! {
    pub static ORDERING_BUFFER: RefCell<HashMap<usize, (FileBlockMetadata, Vec<u8>, Vec<u8>)>> = RefCell::new(HashMap::new());
    // the first expected block is #1. #0 is the header
    pub static LAST_WRITTEN: RefCell<usize> = RefCell::new(0);
    pub static PBF_WRITER: RefCell<Option<Writer>> = RefCell::new(None);
    pub static NEXT_THREAD_POOL: RefCell<Option<Arc<RwLock<ThreadPool>>>> = RefCell::new(None);
}

struct DecodeBlobCommand {
    blob: BlobDesc,
}

impl DecodeBlobCommand {
    pub fn new(
        blob: BlobDesc,
    ) -> DecodeBlobCommand {
        DecodeBlobCommand {
            blob,
        }
    }
}

impl Command for DecodeBlobCommand {
    fn execute(&self) -> Result<(), GenericError> {
        // ignore the FileBlock::Header message
        let file_block = FileBlock::from_blob_desc(&self.blob).unwrap();
        if file_block.as_osm_data().is_ok() {
            NEXT_THREAD_POOL.with(
                |next| {
                    let n = next.replace(None).unwrap();
                    next.replace(Some(n.clone()));
                    let tp = n.read().unwrap();
                    tp.submit(Box::new(EncodeBlobCommand::new(file_block)));
                }
            );
        }
        Ok(())
    }
}

struct EncodeBlobCommand {
    file_block: FileBlock,
}

impl EncodeBlobCommand {
    pub fn new(
        file_block: FileBlock,
    ) -> EncodeBlobCommand {
        EncodeBlobCommand {
            file_block,
        }
    }
}

impl Command for EncodeBlobCommand {
    fn execute(&self) -> Result<(), GenericError> {
        let (header, body) = FileBlock::serialize(&self.file_block, CompressionType::Zlib).unwrap();
        let metadata = self.file_block.metadata().clone();

        NEXT_THREAD_POOL.with(
            |next| {
                let n = next.replace(None).unwrap();
                next.replace(Some(n.clone()));
                let tp = n.read().unwrap();
                tp.submit(
                    Box::new(
                        WriteBlobsCommand::new(
                            Mutex::new(metadata),
                            Mutex::new(header),
                            Mutex::new(body),
                        )
                    )
                );
            }
        );
        Ok(())
    }
}

struct WriteBlobsCommand {
    metadata: Mutex<FileBlockMetadata>,
    header: Mutex<Vec<u8>>,
    body: Mutex<Vec<u8>>,
}

impl WriteBlobsCommand {
    pub fn new(
        metadata: Mutex<FileBlockMetadata>,
        header: Mutex<Vec<u8>>,
        body: Mutex<Vec<u8>>,
    ) -> WriteBlobsCommand {
        WriteBlobsCommand {
            metadata,
            header,
            body,
        }
    }
}


impl Command for WriteBlobsCommand {
    fn execute(&self) -> Result<(), GenericError> {
        ORDERING_BUFFER.with(
            |buffer| {
                let metadata = self.metadata.lock().unwrap();
                let mut header_guard = self.header.lock().unwrap();
                let header = std::mem::replace(header_guard.as_mut(), Vec::<u8>::new());
                let mut body_guard = self.body.lock().unwrap();
                let mut body = Vec::<u8>::new();
                std::mem::swap(body_guard.deref_mut(), &mut body);
                buffer
                    .borrow_mut()
                    .insert(
                        metadata.index(),
                        (
                            metadata.clone(),
                            header,
                            body
                        ),
                    );
            }
        );

        ORDERING_BUFFER.with(
            |buffer| {
                LAST_WRITTEN.with(|last| {
                    let last_written = *last.borrow();
                    for i in (last_written + 1)..usize::MAX {
                        match buffer.borrow_mut().remove(&i) {
                            None => {
                                *last.borrow_mut() = i - 1;
                                break;
                            }
                            Some((metadata, header, body)) => {
                                PBF_WRITER.with(
                                    |writer| {
                                        let mut w = writer.replace(None);
                                        w.as_mut().unwrap().add_bounding_box(metadata.bounding_box());
                                        w.as_mut().unwrap().write_blob(header, body).expect("Failed to write a blob");
                                        writer.replace(w);
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

#[test]
fn test_pbf_rw_parallel_pipe() {
    SimpleLogger::new().init().unwrap();
    common::setup();
    log::info!("Started OSM PBF rw parallel pipe test");
    let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    let output_path = PathBuf::from("./tests/parallel-results/malta-230109.osm.pbf");
    let fixture_analysis_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf.osm.pbf.analysis.json");

    let reader = pbf::reader::Reader::new(input_path.clone()).unwrap();
    let mut info = reader.info().clone();
    info.set_writingprogram(&Some("rw-pipe-test-writer".to_string()));
    info.set_source(&Some("from fixture".to_string()));

    log::info!("start iteration over blobs for {:?}", input_path);
    let mut stopwatch = StopWatch::new();
    stopwatch.start();

    let pbf_block_decoder_pool = Arc::new(
        RwLock::new(
            ThreadPoolBuilder::new()
                .tasks(8)
                .queue_size(64)
                .name("pbf-block-decoder".to_string())
                .shutdown_mode(ShutdownMode::CompletePending)
                .build()
                .unwrap()
        )
    );

    let pbf_block_encoder_pool = Arc::new(
        RwLock::new(
            ThreadPoolBuilder::new()
                .tasks(8)
                .queue_size(64)
                .name("pbf-block-encoder".to_string())
                .shutdown_mode(ShutdownMode::CompletePending)
                .build()
                .unwrap()
        )
    );

    let pbf_block_writer_pool = Arc::new(
        RwLock::new(
            ThreadPoolBuilder::new()
                .tasks(1)
                .queue_size(1024)
                .name("pbf-block-writer".to_string())
                .shutdown_mode(ShutdownMode::CompletePending)
                .build()
                .unwrap()
        )
    );

    set_next(pbf_block_decoder_pool.clone(), pbf_block_encoder_pool.clone());
    set_next(pbf_block_encoder_pool.clone(), pbf_block_writer_pool.clone());
    init_writer(pbf_block_writer_pool.clone(), output_path.clone(), info.clone(), CompressionType::Zlib);

    let mut i = 0_usize;
    for blob in reader.blobs().unwrap() {
        let pbd = pbf_block_decoder_pool.read().unwrap();
        pbd.submit(Box::new(DecodeBlobCommand::new(blob)));
        i.add_assign(1);
    }

    log::info!("Finished submitting blobs");

    shutdown(pbf_block_decoder_pool);
    shutdown(pbf_block_encoder_pool);
    shutdown(pbf_block_writer_pool);

    common::analyze_pbf_output(output_path, fixture_analysis_path);

    log::info!("Finished OSM PBF rw parallel pipe test, time: {stopwatch}");
}

fn init_writer(writer_thread_pool: Arc<RwLock<ThreadPool>>, output_path: PathBuf, info: FileInfo, compression_type: CompressionType) {
    let tp = writer_thread_pool.read().unwrap();
    tp.in_all_threads(
        Arc::new(
            Mutex::new(
                move || {
                    PBF_WRITER.with(
                        |writer| {
                            if writer.borrow().is_none() {
                                let mut w = Writer::from_file_info(
                                    output_path.clone(),
                                    info.clone(),
                                    compression_type.clone(),
                                ).expect("Failed to create a writer");
                                w.write_header().expect("Failed to write header");
                                writer.replace(Some(w));
                            }
                        }
                    )
                }
            )
        )
    );
}

fn shutdown(thread_pool: Arc<RwLock<ThreadPool>>) {
    let mut tp = thread_pool.write().unwrap();
    tp.shutdown();
    tp.join().expect("Failed to join thread pool");
}

fn set_next(target: Arc<RwLock<ThreadPool>>, next: Arc<RwLock<ThreadPool>>) {
    let t = target.read().unwrap();
    t.in_all_threads(
        Arc::new(
            Mutex::new(
                move || {
                    NEXT_THREAD_POOL.with(
                        |n| {
                            n.replace(Some(next.clone()));
                        }
                    );
                }
            )
        )
    );
}
