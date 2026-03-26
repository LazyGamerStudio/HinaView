// src/pipeline/scheduler.rs
use super::concurrency_controller::DecodingSemaphore;
use super::job::{DecodeJob, PipelineJob};
use crate::cache::CpuDecodeCache;
use crate::pipeline::priority_queue::PriorityJobQueue;
use crate::types::MipLevel;
use std::collections::HashSet;
use std::sync::Arc;

/// Priority for decode jobs. Lower number = higher priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct JobPriority(pub u32);

impl JobPriority {
    /// Highest priority: current page being displayed
    pub const CURRENT: JobPriority = JobPriority(0);
    /// High priority: next/previous page (prefetch)
    pub const PREFETCH_CLOSE: JobPriority = JobPriority(1000);
    /// Medium priority: +2/-2 pages (prefetch)
    pub const PREFETCH_MEDIUM: JobPriority = JobPriority(1001);
    /// Low priority: background prefetch
    pub const PREFETCH_FAR: JobPriority = JobPriority(1002);
}

pub struct DecodeScheduler {
    job_queue: Arc<PriorityJobQueue>,
    decode_semaphore: Arc<DecodingSemaphore>,
    default_decode_limit: usize,
    inflight: HashSet<u64>,
    cpu_cache: Option<CpuDecodeCache>,
}

impl DecodeScheduler {
    /// Create scheduler with CPU cache enabled (memory-based).
    pub fn with_cache(
        job_queue: Arc<PriorityJobQueue>,
        decode_semaphore: Arc<DecodingSemaphore>,
        cpu_cache: CpuDecodeCache,
    ) -> Self {
        let default_decode_limit = decode_semaphore.limit();
        Self {
            job_queue,
            decode_semaphore,
            default_decode_limit,
            inflight: HashSet::new(),
            cpu_cache: Some(cpu_cache),
        }
    }

    /// Generate a unique hash for (doc_id, page_name, mip) combination.
    fn make_hash(doc_id: u64, page_name: &str, mip: MipLevel) -> u64 {
        crate::cache::gpu_uploader::cache_key(doc_id, page_name, mip)
    }

    /// Check if a decoded image is available in CPU cache.
    #[allow(dead_code)]
    pub fn get_from_cache(
        &mut self,
        doc_id: u64,
        page_name: &str,
        mip: MipLevel,
    ) -> Option<std::sync::Arc<crate::types::DecodedImage>> {
        let hash = Self::make_hash(doc_id, page_name, mip);
        if let Some(ref mut cache) = self.cpu_cache {
            cache.get(hash, mip)
        } else {
            None
        }
    }

    /// Enqueue a decode job with priority.
    /// Lower priority number = higher priority.
    /// Returns true if the job was enqueued, false if skipped (cache hit or in-flight).
    pub fn enqueue_with_priority(&mut self, job: DecodeJob, priority: JobPriority) -> bool {
        let mut job = job;

        // ANIMATION MIP NORMALIZATION:
        // Animated images (WebP/GIF) must maintain a consistent FrameStream at Full resolution.
        // If we allow different MIP levels (e.g., Half, Quarter) for the same animated page,
        // the scheduler hash (which includes MIP) would treat them as distinct jobs,
        // leading to redundant decodes and queue explosion during fast navigation.
        // REFACTORING NOTE: Do not remove this normalization without updating the 'make_hash'
        // function to exclude MIP for animated pages, otherwise deduplication will fail.
        if job.is_animated {
            job.mip = MipLevel::Full;
        }

        let hash = Self::make_hash(job.doc_id, &job.page_name, job.mip);

        if self.inflight.contains(&hash) {
            tracing::debug!(
                "[Scheduler] SKIP enqueue: inflight hit | doc_id={} page={} mip={:?} prio={} reason={}",
                job.doc_id,
                job.page_name,
                job.mip,
                priority.0,
                job.reason
            );
            return false;
        }

        if let Some(ref cache) = self.cpu_cache
            && cache.contains(hash, job.mip)
        {
            tracing::debug!(
                "[Scheduler] SKIP enqueue: CPU cache hit | doc_id={} page={} mip={:?} prio={} reason={}",
                job.doc_id,
                job.page_name,
                job.mip,
                priority.0,
                job.reason
            );
            return false;
        }

        self.inflight.insert(hash);

        job.priority = priority.0;
        self.job_queue.push(PipelineJob::Decode(job));
        true
    }

    /// Mark a job as completed (remove from in-flight).
    pub fn complete(&mut self, doc_id: u64, page_name: &str, mip: MipLevel) {
        let hash = Self::make_hash(doc_id, page_name, mip);
        self.inflight.remove(&hash);
    }

    pub fn is_inflight(&self, doc_id: u64, page_name: &str, mip: MipLevel) -> bool {
        let hash = Self::make_hash(doc_id, page_name, mip);
        self.inflight.contains(&hash)
    }

    pub fn has_any_inflight(&self) -> bool {
        !self.inflight.is_empty()
    }

    /// Store a decoded image in the CPU cache.
    pub fn cache_result(
        &mut self,
        doc_id: u64,
        page_name: &str,
        mip: MipLevel,
        image: std::sync::Arc<crate::types::DecodedImage>,
    ) {
        let hash = Self::make_hash(doc_id, page_name, mip);
        if let Some(ref mut cache) = self.cpu_cache {
            cache.insert(hash, mip, image);
        }
    }

    /// Clear all in-flight jobs and pending queue (used when navigating to a new page).
    pub fn clear_inflight(&mut self) {
        self.inflight.clear();
        self.job_queue.clear_all();
    }

    /// Get CPU cache memory usage in MB.
    pub fn cpu_cache_memory_mb(&self) -> usize {
        self.cpu_cache.as_ref().map_or(0, |c| c.memory_usage_mb())
    }

    pub fn cpu_cache_max_mb(&self) -> usize {
        self.cpu_cache.as_ref().map_or(0, |c| c.max_memory_mb())
    }

    pub fn set_cpu_cache_limit_mb(&mut self, mb: usize) {
        if let Some(ref mut cache) = self.cpu_cache {
            cache.set_max_memory_mb(mb);
        } else {
            self.cpu_cache = Some(CpuDecodeCache::new_with_memory_limit(mb));
        }
    }

    pub fn evict_page_all_mips(&mut self, doc_id: u64, page_name: &str) {
        if let Some(ref mut cache) = self.cpu_cache {
            // Since we don't know the exact hash used for the page_name (it's internal to make_hash),
            // we should technically iterate through all possible MIPs and generate the hash for each.
            // But actually, we already have a specialized method in CpuDecodeCache if we change it to take
            // (doc_id, page_name) or if we compute the base hash here.

            // For now, let's use the most reliable way:
            let mips = [
                MipLevel::Eighth,
                MipLevel::Quarter,
                MipLevel::ThreeEighths,
                MipLevel::Half,
                MipLevel::FiveEighths,
                MipLevel::ThreeQuarters,
                MipLevel::SevenEighths,
                MipLevel::Full,
            ];

            for mip in mips {
                let hash = Self::make_hash(doc_id, page_name, mip);
                cache.remove(hash, mip);
            }
        }
    }

    pub fn restore_default_decode_limit(&self) {
        self.decode_semaphore.set_limit(self.default_decode_limit);
    }

    pub fn set_protection(&mut self, protections: std::collections::HashMap<u64, usize>) {
        if let Some(ref mut cache) = self.cpu_cache {
            cache.set_protection(protections);
        }
    }
}
