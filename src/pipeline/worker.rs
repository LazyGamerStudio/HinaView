// src/pipeline/worker.rs
use super::concurrency_controller::DecodingSemaphore;
use super::result::DecodeResult;
use crate::pipeline::priority_queue::PriorityJobQueue;
use crossbeam::channel::Sender;
use std::sync::Arc;

pub fn start_workers(
    job_queue: Arc<PriorityJobQueue>,
    result_tx: Sender<DecodeResult>,
    semaphore: Arc<DecodingSemaphore>,
) {
    super::worker_pool::start(job_queue, result_tx, semaphore);
}
