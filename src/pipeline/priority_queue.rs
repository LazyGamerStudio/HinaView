// src/pipeline/priority_queue.rs
use super::job::{PipelineJob, QueuedJob};
use crate::pipeline::scheduler::JobPriority;
use parking_lot::{Condvar, Mutex};
use std::collections::BinaryHeap;
use tracing::{debug, info};

pub struct PriorityJobQueue {
    inner: Mutex<QueueInner>,
    cvar: Condvar,
}

struct QueueInner {
    decode_heap: BinaryHeap<QueuedJob>,
    sequence_counter: u64,
    urgent_in_progress: usize,
    is_closed: bool,
}

impl PriorityJobQueue {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(QueueInner {
                decode_heap: BinaryHeap::new(),
                sequence_counter: 0,
                urgent_in_progress: 0,
                is_closed: false,
            }),
            cvar: Condvar::new(),
        }
    }

    pub fn push(&self, job: PipelineJob) {
        let mut inner = self.inner.lock();
        if inner.is_closed {
            return;
        }

        let sequence = inner.sequence_counter;
        inner.sequence_counter += 1;

        let job_type = match &job {
            PipelineJob::Decode(j) => format!("Decode({})", j.page_name),
        };
        let prio = job.priority();

        let queued = QueuedJob { job, sequence };
        match queued.job {
            PipelineJob::Decode(_) => inner.decode_heap.push(queued),
        }

        debug!(
            "[Queue] PUSH: {} | prio={} | q_len(D:{})",
            job_type,
            prio,
            inner.decode_heap.len(),
        );

        self.cvar.notify_one();
    }

    pub fn pop(&self) -> Option<PipelineJob> {
        let mut inner = self.inner.lock();

        loop {
            if inner.is_closed && inner.decode_heap.is_empty() {
                return None;
            }

            let job = self.pick_best_job(&mut inner);

            if let Some(job) = job {
                let job_type = match &job {
                    PipelineJob::Decode(j) => format!("Decode({})", j.page_name),
                };

                debug!(
                    "[Queue] POP: {} | prio={} | urgent_cnt={} | q_len(D:{})",
                    job_type,
                    job.priority(),
                    inner.urgent_in_progress,
                    inner.decode_heap.len(),
                );

                if JobPriority(job.priority()) == JobPriority::CURRENT {
                    inner.urgent_in_progress += 1;
                }
                return Some(job);
            }

            if inner.is_closed {
                return None;
            }
            self.cvar.wait(&mut inner);
        }
    }

    fn pick_best_job(&self, inner: &mut QueueInner) -> Option<PipelineJob> {
        inner.decode_heap.pop().map(|q| q.job)
    }

    pub fn complete_job(&self, priority: u32) {
        let mut inner = self.inner.lock();
        if JobPriority(priority) == JobPriority::CURRENT {
            inner.urgent_in_progress = inner.urgent_in_progress.saturating_sub(1);
            self.cvar.notify_all();
        }
    }

    pub fn clear_all(&self) {
        let mut inner = self.inner.lock();
        let d_len = inner.decode_heap.len();
        inner.decode_heap.clear();
        inner.urgent_in_progress = 0;
        info!("[Queue] CLEAR_ALL: dropped {} decode jobs.", d_len);
    }
}
