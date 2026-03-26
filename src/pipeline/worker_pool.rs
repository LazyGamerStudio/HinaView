use super::concurrency_controller::{DecodingPermit, DecodingSemaphore};
use super::job::PipelineJob;
use super::result::DecodeResult;
use crate::pipeline::priority_queue::PriorityJobQueue;
use ::tracing::error;
use crossbeam::channel::Sender;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const MIN_WORKER_THREADS: usize = 1;
const MAX_WORKER_THREADS: usize = 8;

pub fn start(
    job_queue: Arc<PriorityJobQueue>,
    result_tx: Sender<DecodeResult>,
    semaphore: Arc<DecodingSemaphore>,
) {
    let decode_limit = semaphore.limit();
    let thread_count = num_cpus::get()
        .saturating_sub(1)
        .max(MIN_WORKER_THREADS)
        .clamp(MIN_WORKER_THREADS, MAX_WORKER_THREADS);

    tracing::info!(
        "[WorkerPool] Spawning {} worker threads (Decoding limit: {})",
        thread_count,
        decode_limit
    );

    for i in 0..thread_count {
        let job_queue = job_queue.clone();
        let result_tx = result_tx.clone();
        let semaphore = semaphore.clone();

        thread::spawn(move || {
            while let Some(pipeline_job) = job_queue.pop() {
                match pipeline_job {
                    PipelineJob::Decode(job) => {
                        let priority = job.priority;

                        if let Some(_permit) = DecodingPermit::try_acquire(semaphore.clone()) {
                            let result = super::decode_executor::execute_decode_job(i, job);
                            if let Some(result) = result
                                && let Err(e) = result_tx.send(result)
                            {
                                error!("[Worker {}]   └─ Send ERROR: {}", i, e);
                            }
                            job_queue.complete_job(priority);
                            continue;
                        }

                        // Fall back to blocking acquire for all workers.
                        // This removes POP->PUSH churn when decode slots are saturated.
                        let wait_start = Instant::now();
                        let _permit = DecodingPermit::acquire(semaphore.clone());
                        let permit_wait_ms = wait_start.elapsed().as_secs_f32() * 1000.0;
                        if permit_wait_ms >= 0.1 {
                            tracing::debug!(
                                "[Worker {}][LockWait] decode_permit={:.2}ms prio={}",
                                i,
                                permit_wait_ms,
                                priority
                            );
                        }
                        let result = super::decode_executor::execute_decode_job(i, job);
                        if let Some(result) = result
                            && let Err(e) = result_tx.send(result)
                        {
                            error!("[Worker {}]   └─ Send ERROR: {}", i, e);
                        }
                        job_queue.complete_job(priority);
                    }
                }
            }
        });
    }
}
