use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock, TryLockResult};
use std::sync::atomic::Ordering;
use std::thread;
use command_executor::errors::GenericError;
use command_executor::executor::command::Command;
use command_executor::executor::shutdown_mode::ShutdownMode;
use command_executor::executor::thread_pool::ThreadPool;
use command_executor::executor::thread_pool_builder::ThreadPoolBuilder;
use simple_logger::SimpleLogger;
use osm_io::config::Config;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::blob_desc::BlobDesc;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_block;
use osm_io::osm::pbf::file_block::FileBlock;
use osm_io::osm::pbf::file_block_metadata::FileBlockMetadata;
use osm_io::reporting::stopwatch::StopWatch;

mod common;

use std::cell::{Ref, RefCell};
use std::time::Duration;
use bytes::Buf;
use osm_io::osm::pbf::writer::Writer;

thread_local! {
    pub static ORDERING_BUFFER: RefCell<HashMap<usize, (FileBlockMetadata, Vec<u8>, Vec<u8>)>> = RefCell::new(HashMap::new());
    // the first expected block is #1. #0 is the header
    pub static LAST_WRITTEN: RefCell<usize> = RefCell::new(0);
    pub static PBF_WRITER: RefCell<Option<Writer>> = RefCell::new(None);
}

struct DecodeBlobCommand {
    blob: BlobDesc,
    work_stages: Arc<RwLock<HashMap::<String, Arc<RwLock<ThreadPool>>>>>,
    config: Arc<RwLock<Config>>,
}

impl DecodeBlobCommand {
    pub fn new(
        blob: BlobDesc,
        work_stages: Arc<RwLock<HashMap::<String, Arc<RwLock<ThreadPool>>>>>,
        config: Arc<RwLock<Config>>,
    ) -> DecodeBlobCommand {
        DecodeBlobCommand {
            blob,
            work_stages,
            config,
        }
    }
}

impl Command for DecodeBlobCommand {
    fn execute(&self) -> Result<(), GenericError> {
        // ignore the FileBlock::Header message
        let file_block = FileBlock::from_blob_desc(&self.blob).unwrap();
        if file_block.as_osm_data().is_ok() {
            let ws = self.work_stages.read().unwrap();
            let pbw = ws.get("pbf-block-encoder").unwrap().read().unwrap();
            pbw.submit(Box::new(EncodeBlobCommand::new(file_block, self.work_stages.clone(), self.config.clone())));
        }
        Ok(())
    }
}

struct EncodeBlobCommand {
    file_block: FileBlock,
    work_stages: Arc<RwLock<HashMap::<String, Arc<RwLock<ThreadPool>>>>>,
    config: Arc<RwLock<Config>>,
}

impl EncodeBlobCommand {
    pub fn new(
        file_block: FileBlock,
        work_stages: Arc<RwLock<HashMap::<String, Arc<RwLock<ThreadPool>>>>>,
        config: Arc<RwLock<Config>>,
    ) -> EncodeBlobCommand {
        EncodeBlobCommand {
            file_block,
            work_stages,
            config,
        }
    }
}

impl Command for EncodeBlobCommand {
    fn execute(&self) -> Result<(), GenericError> {
        let (header, body) = FileBlock::serialize(&self.file_block, CompressionType::Zlib).unwrap();
        let ws = self.work_stages.read().unwrap();
        let pbo = ws.get("pbf-block-ordering").unwrap().read().unwrap();
        pbo.submit(
            Box::new(
                OrderBlobsCommand::new(
                    Mutex::new(self.file_block.metadata().clone()),
                    Mutex::new(header),
                    Mutex::new(body),
                    self.work_stages.clone(),
                    self.config.clone(),
                )
            )
        );
        Ok(())
    }
}

struct OrderBlobsCommand {
    metadata: Mutex<FileBlockMetadata>,
    header: Mutex<Vec<u8>>,
    body: Mutex<Vec<u8>>,
    work_stages: Arc<RwLock<HashMap::<String, Arc<RwLock<ThreadPool>>>>>,
    config: Arc<RwLock<Config>>,
}

impl OrderBlobsCommand {
    pub fn new(
        metadata: Mutex<FileBlockMetadata>,
        header: Mutex<Vec<u8>>,
        body: Mutex<Vec<u8>>,
        work_stages: Arc<RwLock<HashMap::<String, Arc<RwLock<ThreadPool>>>>>,
        config: Arc<RwLock<Config>>,
    ) -> OrderBlobsCommand {
        OrderBlobsCommand {
            metadata,
            header,
            work_stages,
            config,
            body,
        }
    }
}


impl Command for OrderBlobsCommand {
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

        PBF_WRITER.with(
            |mut writer| {
                if writer.borrow().is_none() {
                    let config = self.config.read().unwrap();
                    let mut w = Writer::from_file_info(
                        config.output.clone(),
                        config.file_info.clone(),
                        CompressionType::Zlib,
                        true,
                    ).expect("Failed to create a writer");
                    w.write_header().expect("Failed to write header");
                    writer.replace(Some(w));
                }
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
    let input_path = PathBuf::from("./tests/fixtures/germany-230109.osm.pbf");
    let output_path = PathBuf::from("./tests/parallel-results/germany-230109.osm.pbf");


    let reader = pbf::reader::Reader::new(input_path.clone()).unwrap();
    let mut info = reader.info().clone();
    info.writingprogram = Some("rw-pipe-test-writer".to_string());
    info.source = Some("from fixture".to_string());

    let config = Arc::new(
        RwLock::new(
            Config::new(
                input_path.clone(),
                "pbf".to_string(),
                output_path.clone(),
                "pbf".to_string(),
                info.clone(),
                true,
            )
        )
    );

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

    let pbf_block_ordering_pool = Arc::new(
        RwLock::new(
            ThreadPoolBuilder::new()
                .tasks(1)
                .queue_size(1024)
                .name("pbf-block-ordering".to_string())
                .shutdown_mode(ShutdownMode::CompletePending)
                .build()
                .unwrap()
        )
    );

    let work_stages = Arc::new(RwLock::new(HashMap::<String, Arc<RwLock<ThreadPool>>>::new()));
    {
        let mut ws = work_stages.write().unwrap();
        ws.insert("pbf-block-decoder".to_string(), pbf_block_decoder_pool);
        ws.insert("pbf-block-encoder".to_string(), pbf_block_encoder_pool);
        ws.insert("pbf-block-ordering".to_string(), pbf_block_ordering_pool);
    }

    for blob in reader.blobs().unwrap() {
        let ws = work_stages.read().unwrap();
        let pbd = ws.get("pbf-block-decoder").unwrap().read().unwrap();
        pbd.submit(Box::new(DecodeBlobCommand::new(blob, work_stages.clone(), config.clone())));
    }

    log::info!("Finished submitting blobs");

    shutdown_work_stage(&work_stages, "pbf-block-decoder");
    shutdown_work_stage(&work_stages, "pbf-block-encoder");
    shutdown_work_stage(&work_stages, "pbf-block-ordering");
    log::info!("Finished OSM PBF rw parallel pipe test, time: {stopwatch}");
}

fn shutdown_work_stage(work_stages: &Arc<RwLock<HashMap<String, Arc<RwLock<ThreadPool>>>>>, name: &str) {
    loop {
        let mut tp_lock_opt = None;
        match work_stages.try_write() {
            Ok(mut ws) => {
                tp_lock_opt = ws.remove(name);
                log::info!("removed {name} thread pool from work stages");
            }
            Err(_) => {
                log::info!("Failed to get a write lock for work stages. sleeping");
                thread::sleep(Duration::from_millis(10));
            }
        }
        match tp_lock_opt {
            None => {}
            Some(pbd_lock) => {
                let mut tp = pbd_lock.write().unwrap();
                tp.shutdown();
                tp.join().expect("failed joining {name} pool");
                log::info!("Shut down {name} thread pool");
                break;
            }
        }
    }
}