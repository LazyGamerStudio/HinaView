// src/pipeline/job.rs
use crate::pipeline::types::ArchiveReader;
use crate::types::*;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct DecodeJob {
    pub doc_id: u64,
    pub page_id: PageId,
    pub page_name: String,
    pub mip: MipLevel,
    /// Indicates if the target image is an animated format (e.g., WebP, GIF).
    /// NECESSITY: Required for the scheduler to enforce MipLevel::Full normalization,
    /// preventing redundant decode jobs for the same animated page at different resolutions.
    pub is_animated: bool,
    pub skip_resample: bool,
    pub priority: u32,
    pub reader: Arc<dyn ArchiveReader + Send + Sync>,
    pub enqueued_at: Instant,
    /// Contextual information for debugging (e.g., "CURRENT", "PREFETCH").
    pub reason: String,
}

#[derive(Clone)]
pub enum PipelineJob {
    Decode(DecodeJob),
}

impl PipelineJob {
    pub fn priority(&self) -> u32 {
        match self {
            PipelineJob::Decode(job) => job.priority,
        }
    }
}

/// Wrapper for PipelineJob to include insertion sequence for FIFO behavior within same priority.
#[derive(Clone)]
pub struct QueuedJob {
    pub job: PipelineJob,
    pub sequence: u64,
}

impl PartialEq for QueuedJob {
    fn eq(&self, other: &Self) -> bool {
        self.job.priority() == other.job.priority() && self.sequence == other.sequence
    }
}

impl Eq for QueuedJob {}

impl Ord for QueuedJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare priority (lower number = higher priority)
        let prio_cmp = other.job.priority().cmp(&self.job.priority());
        if prio_cmp != std::cmp::Ordering::Equal {
            return prio_cmp;
        }
        // If priority is equal, compare sequence (lower sequence = entered earlier = higher priority)
        other.sequence.cmp(&self.sequence)
    }
}

impl PartialOrd for QueuedJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
