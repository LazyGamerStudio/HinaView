use crate::cache::TextureManager;
use crate::pipeline::{DecodeScheduler, UploadQueue};
use crate::types::MipLevel;
use crate::view::NavigationController;
use crate::view::animation_controller::AnimationController;
use tracing::error;

pub struct UploadContext<'a> {
    pub upload_queue: &'a UploadQueue,
    pub scheduler: &'a mut DecodeScheduler,
    pub nav: &'a mut NavigationController,
    pub texture_manager: &'a mut TextureManager,
    pub renderer: Option<&'a crate::renderer::Renderer>,
    pub animation_controller: &'a mut AnimationController,
    pub window_size: (u32, u32),
    pub needs_visible_check: &'a mut bool,
}

pub fn process_upload_queue_controller(ctx: UploadContext) -> bool {
    let mut gpu_updated = false;
    let mut animated_uploads_this_frame = 0;
    const MAX_ANIMATED_UPLOADS_PER_FRAME: usize = 2; // Limit to prevent staging buffer spikes

    while let Some(mut result) = ctx.upload_queue.try_recv() {
        let completed_mip = result.mip;
        ctx.scheduler
            .complete(result.doc_id, &result.page_name, completed_mip);

        let is_animated_result = result.is_animated || result.stream.is_some();

        // 2. Throttling: Limit how many heavy animated frames we upload in a single main loop frame.
        let is_fast = ctx.nav.is_fast_navigating();
        if is_fast
            && is_animated_result
            && animated_uploads_this_frame >= MAX_ANIMATED_UPLOADS_PER_FRAME
        {
            // NOTE: We don't discard these yet, as they are close to the current view.
            // But we stop processing the queue for this frame to yield to the renderer.
            // They will be picked up in the next frame.
            break;
        }

        if is_animated_result && result.mip != MipLevel::Full {
            tracing::debug!(
                "[Upload] Promote animated page to Full mip for GPU upload: {} ({:?} -> Full)",
                result.page_name,
                result.mip
            );
            result.mip = MipLevel::Full;
        }

        if let Some(renderer) = ctx.renderer {
            let current_doc_id = ctx.nav.document.as_ref().map(|d| d.id).unwrap_or(0);
            ctx.texture_manager.upload_to_gpu(
                result.clone(),
                current_doc_id,
                &renderer.device,
                &renderer.queue,
                &renderer.texture_bind_group_layout,
                &renderer.sampler,
            );
            gpu_updated = true;
            if is_animated_result {
                animated_uploads_this_frame += 1;
            }
        } else {
            error!("[App] Warning: Renderer not initialized before upload.");
        }

        let is_current_doc_result = ctx
            .nav
            .document
            .as_ref()
            .map(|d| d.id == result.doc_id)
            .unwrap_or(false);

        if is_current_doc_result && let Some(stream) = result.stream.as_ref() {
            // Check if animation is already registered and active for this page.
            // This prevents redundant "double start" (resetting to frame 0) when
            // multiple decode results for the same animated page arrive.
            if !ctx.animation_controller.has_active_for(&[result.page_id]) {
                let initial_delay = std::time::Duration::from_millis(result.first_frame_delay_ms);
                ctx.animation_controller
                    .register(result.page_id, stream.clone(), initial_delay);
                gpu_updated = true;
            }
        }

        let mut needs_layout_rebuild = false;
        if is_current_doc_result
            && let Some(doc) = ctx.nav.document.as_mut()
            && let Some(page) = doc.pages.get_mut(result.page_id)
        {
            let decoded_w = result.image.original_width;
            let decoded_h = result.image.original_height;
            let decoded_is_wide = result.image.width > result.image.height;
            let decoded_is_animated = is_animated_result;

            if page.width != decoded_w
                || page.height != decoded_h
                || page.is_wide != decoded_is_wide
                || page.is_animated != decoded_is_animated
            {
                page.width = decoded_w;
                page.height = decoded_h;
                page.is_wide = decoded_is_wide;
                page.is_animated = decoded_is_animated;
                page.metadata_probe_failed = false;
                page.format_label = result.decoder_name.to_string();
                needs_layout_rebuild = true;
            }

            if page.format_label == "Unknown" || page.format_label.is_empty() {
                page.format_label = result.decoder_name.to_string();
            }

            if let Some(ref icc) = result.image.icc_profile {
                if page.icc_profile.is_none() {
                    page.icc_profile = Some(icc.clone());
                }
            }
            if let Some(ref exif) = result.image.exif {
                if page.exif_camera.is_none() {
                    page.exif_camera.clone_from(&exif.camera);
                }
                if page.exif_lens.is_none() {
                    page.exif_lens.clone_from(&exif.lens);
                }
                if page.exif_f_stop.is_none() {
                    page.exif_f_stop.clone_from(&exif.f_stop);
                }
                if page.exif_shutter.is_none() {
                    page.exif_shutter.clone_from(&exif.shutter_speed);
                }
                if page.exif_iso.is_none() {
                    page.exif_iso.clone_from(&exif.iso);
                }
                if page.exif_datetime.is_none() {
                    page.exif_datetime.clone_from(&exif.datetime);
                }
            }
        }

        if needs_layout_rebuild {
            if let Some(doc) = ctx.nav.document.as_mut() {
                doc.rebuild_spreads(ctx.nav.view.layout_mode);
            }
            ctx.nav.refresh_layout(ctx.window_size);
            *ctx.needs_visible_check = true;
            gpu_updated = true;

            if Some(result.page_id) == ctx.nav.current_page {
                ctx.nav.center_camera_on_page(result.page_id);
            }
        }

        if !is_animated_result {
            let decoded_arc = std::sync::Arc::new(result.image);
            ctx.scheduler
                .cache_result(result.doc_id, &result.page_name, result.mip, decoded_arc);
        } else {
            // If it's animated, make sure no static versions are left in CPU cache to avoid "stale static frame" bug.
            ctx.scheduler
                .evict_page_all_mips(result.doc_id, &result.page_name);
        }
    }

    let visible = ctx.nav.get_visible_pages(ctx.window_size);
    ctx.animation_controller.retain_visible(&visible);

    gpu_updated
}
