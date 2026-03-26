// src/pipeline/mod.rs
pub mod decode;
pub mod decode_executor;
pub mod decoders;
pub mod job;
pub mod resample;
pub mod result;
pub mod scheduler;
pub mod types;
pub mod upload_processor;
pub mod upload_queue;
pub mod worker;
pub mod worker_pool;

pub mod concurrency_controller;
pub mod priority_queue;

pub use crate::cache::CpuDecodeCache;
use concurrency_controller::DecodingSemaphore;
use crossbeam::channel;
pub use job::DecodeJob;
pub use priority_queue::PriorityJobQueue;
pub use result::DecodeResult;
pub use scheduler::{DecodeScheduler, JobPriority};
use std::sync::Arc;
pub use upload_queue::UploadQueue;

/// Initialize pipeline with CPU decode cache (memory-based, 256 MB default).
pub fn init_pipeline_with_cache(cpu_cache: CpuDecodeCache) -> (DecodeScheduler, UploadQueue) {
    let job_queue = Arc::new(PriorityJobQueue::new());
    let (result_tx, result_rx) = channel::unbounded();
    let semaphore = DecodingSemaphore::new_auto();

    worker::start_workers(job_queue.clone(), result_tx, semaphore.clone());

    let scheduler = DecodeScheduler::with_cache(job_queue, semaphore, cpu_cache);
    let upload_queue = UploadQueue::new(result_rx);

    (scheduler, upload_queue)
}

pub struct DecodeLogInfo<'a> {
    pub worker_id: usize,
    pub name: &'a str,
    pub decoder: &'a str,
    pub orig_w: u32,
    pub orig_h: u32,
    pub dec_w: u32,
    pub dec_h: u32,
    pub queue_wait_ms: f32,
    pub read_ms: f32,
    pub decode_ms: f32,
    pub resample_ms: f32,
    pub worker_total_ms: f32,
}

pub fn log_decode_info(info: DecodeLogInfo) {
    tracing::debug!(
        "[Worker {}][Decode] {} | {}: {}x{} -> {}x{} | qwait={:.1}ms read={:.1}ms decode={:.1}ms resample={:.1}ms total={:.1}ms",
        info.worker_id,
        info.name,
        info.decoder,
        info.orig_w,
        info.orig_h,
        info.dec_w,
        info.dec_h,
        info.queue_wait_ms,
        info.read_ms,
        info.decode_ms,
        info.resample_ms,
        info.worker_total_ms
    );
}

pub fn log_resample_info(
    worker_id: usize,
    name: &str,
    mip: crate::types::MipLevel,
    res_w: u32,
    res_h: u32,
    ms: f32,
) {
    tracing::debug!(
        "[Worker {}][Resample] {} | {:?}: -> {}x{} ({:.1}ms)",
        worker_id,
        name,
        mip,
        res_w,
        res_h,
        ms
    );
}
