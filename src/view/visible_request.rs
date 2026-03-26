use crate::cache::TextureManager;
use crate::cache::gpu_uploader::GpuUploadContext;
use crate::pipeline::DecodeScheduler;
use crate::view::NavigationController;
use std::collections::HashSet;

pub struct VisibleRequestContext<'a> {
    pub nav: &'a mut NavigationController,
    pub scheduler: &'a mut DecodeScheduler,
    pub texture_manager: &'a mut TextureManager,
    pub animation_controller: &'a crate::view::animation_controller::AnimationController,
    pub renderer: Option<&'a crate::renderer::Renderer>,
    pub window_size: (u32, u32),
    pub is_fast: bool,
    pub has_pending_results: bool,
}

pub fn request_visible_pages_controller(mut ctx: VisibleRequestContext) {
    let (doc_id, current_page, fit_mode, layout_mode, rotation, window_size) = {
        let Some(doc) = ctx.nav.document.as_ref() else {
            return;
        };
        let Some(current) = ctx.nav.current_page else {
            return;
        };
        (
            doc.id,
            current,
            ctx.nav.view.fit_mode,
            ctx.nav.view.layout_mode,
            ctx.nav.view.rotation,
            ctx.window_size,
        )
    };

    // 1. Determine primary target zoom
    let target_zoom = {
        let doc = ctx.nav.document.as_ref();
        crate::view::layout_sync::calculate_target_zoom(
            doc,
            fit_mode,
            layout_mode,
            rotation,
            current_page,
            window_size,
        )
    };

    // 2. Identify all visible or soon-to-be-visible pages
    let mut pages_to_request = vec![current_page];

    // Include pending target if navigating
    if let Some(pending) = ctx.nav.pending_page {
        if pending != current_page {
            pages_to_request.push(pending);
        }
    }

    // Include companion pages (Dual mode) or viewport-overlapping pages (Webtoon)
    if !ctx.is_fast {
        if matches!(layout_mode, crate::types::LayoutMode::VerticalScroll)
            && let Some(layout) = ctx.nav.layout.as_ref()
        {
            let zoom = ctx.nav.camera.zoom.max(0.0001);
            let half_h = ctx.window_size.1 as f32 / (2.0 * zoom);
            let margin_y = half_h;

            let view_min_y = ctx.nav.camera.pan.y - half_h - margin_y;
            let view_max_y = ctx.nav.camera.pan.y + half_h + margin_y;

            for placement in &layout.placements {
                let _page_min_x = placement.position[0];
                let _page_max_x = placement.position[0] + placement.size[0];
                let page_min_y = placement.position[1];
                let page_max_y = placement.position[1] + placement.size[1];
                let visible = !(page_max_y < view_min_y || page_min_y > view_max_y);
                if visible {
                    pages_to_request.push(placement.page_index);
                }
            }
        }

        if let crate::types::LayoutMode::Dual { .. } = layout_mode
            && let Some(doc) = ctx.nav.document.as_ref()
            && let Some(spread) = doc
                .spreads
                .iter()
                .find(|s| s.left == Some(current_page) || s.right == Some(current_page))
        {
            let partner = if spread.left == Some(current_page) {
                spread.right
            } else {
                spread.left
            };
            if let Some(partner_page) = partner {
                pages_to_request.push(partner_page);
            }
        }
    }

    // 3. Process each page through the Unified Request Gateway
    // CRITICAL: During fast navigation, do NOT request any new pages if
    // there's already work in flight OR results waiting to be uploaded to GPU.
    // This strict throttle prevents worker and memory exhaustion.
    if ctx.is_fast && (ctx.scheduler.has_any_inflight() || ctx.has_pending_results) {
        return;
    }

    let mut layout_dirty = false;
    let mut seen_in_frame = HashSet::new();

    for page_id in pages_to_request {
        // Strict frame-local deduplication
        if !seen_in_frame.insert(page_id) {
            continue;
        }

        let page_meta = ctx.nav.document.as_ref().and_then(|d| d.pages.get(page_id));
        let page_animated = page_meta.map_or(false, |p| p.is_animated);

        // Standardize optimal MIP. For animations, navigation_request will force Full.
        let optimal_mip = if page_animated {
            crate::types::MipLevel::Full
        } else {
            crate::sampling::decide_mip_level(target_zoom, false)
        };

        let is_optimal = ctx
            .texture_manager
            .has_optimal_mip(page_id, optimal_mip, page_animated);

        // ANIMATION CONTINUITY FIX:
        // Even if the image exists in the GPU cache (is_optimal == true), we MUST request
        // a decode if the animation stream is not currently active in the AnimationController.
        // This scenario occurs when navigating back to a previously cached animated page;
        // the static texture is available, but the playback stream (FrameStream) needs to be
        // restarted via a fresh decode job.
        let needs_decode = if page_animated {
            !is_optimal || !ctx.animation_controller.is_active(page_id)
        } else {
            !is_optimal
        };

        if needs_decode {
            if let Some(meta) = page_meta {
                let name = meta.name.clone();

                // Step A: Immediate promotion from CPU cache (Only for static images)
                if !page_animated
                    && let Some(cached_image) =
                        ctx.scheduler.get_from_cache(doc_id, &name, optimal_mip)
                {
                    if let Some(doc) = ctx.nav.document.as_mut()
                        && let Some(page) = doc.pages.get_mut(page_id)
                    {
                        let decoded_w = cached_image.original_width;
                        let decoded_h = cached_image.original_height;
                        if decoded_w > 0
                            && decoded_h > 0
                            && (page.width != decoded_w || page.height != decoded_h)
                        {
                            page.width = decoded_w;
                            page.height = decoded_h;
                            page.is_wide = decoded_w > decoded_h;
                            layout_dirty = true;
                        }
                    }
                    if let Some(renderer) = &ctx.renderer {
                        let evicted = crate::cache::texture_manager::upload_image_to_gpu_internal(
                            &mut ctx.texture_manager.textures,
                            &mut ctx.texture_manager.gpu_cache,
                            &mut ctx.texture_manager.gpu_cache_reverse_index,
                            &mut ctx.texture_manager.page_ref_count,
                            GpuUploadContext {
                                doc_id,
                                _page_id: page_id,
                                page_name: &name,
                                mip: optimal_mip,
                                image: &cached_image,
                                device: &renderer.device,
                                queue: &renderer.queue,
                                texture_bind_group_layout: &renderer.texture_bind_group_layout,
                                sampler: &renderer.sampler,
                            },
                            doc_id,
                        );
                        ctx.texture_manager.handle_evicted(evicted);
                    }
                }
                // Step B: Dispatch decode job if not already in flight
                else if !ctx.scheduler.is_inflight(doc_id, &name, optimal_mip) {
                    let (priority, reason) = if page_id == current_page {
                        (crate::pipeline::JobPriority::CURRENT, "CURRENT")
                    } else if ctx.nav.pending_page == Some(page_id) {
                        (crate::pipeline::JobPriority::CURRENT, "PENDING")
                    } else {
                        (crate::pipeline::JobPriority::PREFETCH_CLOSE, "PREFETCH")
                    };

                    crate::view::navigation_request::enqueue_page_request(
                        crate::view::navigation_request::NavigationRequestContext {
                            document: ctx.nav.document.as_ref(),
                            page: page_id,
                            target_zoom,
                            skip_resample: ctx.is_fast,
                            priority,
                            scheduler: &mut ctx.scheduler,
                            reason,
                        },
                    );
                }
            }
        }
    }

    if layout_dirty {
        if let Some(doc) = ctx.nav.document.as_mut() {
            doc.rebuild_spreads(ctx.nav.view.layout_mode);
        }
        ctx.nav.refresh_layout(ctx.window_size);

        if let Some(current) = ctx.nav.current_page {
            ctx.nav.center_camera_on_page(current);
        }
    }
}
