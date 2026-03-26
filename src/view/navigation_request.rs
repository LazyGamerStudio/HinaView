use crate::document::Document;
use crate::pipeline::{DecodeScheduler, JobPriority};
use crate::types::PageId;
use tracing::debug;

pub struct NavigationRequestContext<'a> {
    pub document: Option<&'a Document>,
    pub page: PageId,
    pub target_zoom: f32,
    pub skip_resample: bool,
    pub priority: JobPriority,
    pub scheduler: &'a mut DecodeScheduler,
    pub reason: &'static str,
}

pub fn enqueue_page_request(ctx: NavigationRequestContext) {
    let doc = match ctx.document {
        Some(doc) => doc,
        None => return,
    };
    let page_meta = match doc.pages.get(ctx.page) {
        Some(meta) => meta,
        None => return,
    };

    // Determine fixed MipLevel specification from dynamic UI zoom status.
    // CRITICAL: For animated pages, we ALWAYS use Full mip to ensure consistent
    // in-flight hashing and avoid redundant stream creations when zoom fluctuates.
    // This is the primary gateway where UI intent is normalized before reaching the Pipeline.
    let optimal_mip = if page_meta.is_animated {
        crate::types::MipLevel::Full
    } else {
        crate::sampling::decide_mip_level(ctx.target_zoom, false)
    };

    debug!(
        "[NavReq][{}] Request page {} ({}): zoom={:.3}, mip={:?}, skip_resample={}, dim={}x{}, doc_id={}",
        ctx.reason,
        ctx.page,
        page_meta.name,
        ctx.target_zoom,
        optimal_mip,
        ctx.skip_resample,
        page_meta.width,
        page_meta.height,
        doc.id
    );

    let job = crate::pipeline::DecodeJob {
        doc_id: doc.id,
        page_id: ctx.page,
        page_name: page_meta.name.clone(),
        mip: optimal_mip,
        is_animated: page_meta.is_animated,
        skip_resample: ctx.skip_resample,
        priority: ctx.priority.0,
        reader: doc.reader.clone(),
        enqueued_at: std::time::Instant::now(),
        reason: ctx.reason.to_string(),
    };
    let enqueued = ctx.scheduler.enqueue_with_priority(job, ctx.priority);
    if !enqueued {
        tracing::debug!(
            "[NavReq] enqueue skipped (cache/inflight): page {} ({}) mip={:?} prio={}",
            ctx.page,
            page_meta.name,
            optimal_mip,
            ctx.priority.0
        );
    }
}
