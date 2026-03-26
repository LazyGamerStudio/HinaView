use super::App;
use crate::types::MipLevel;
use crate::view::navigation_types::NavigationDirection;
use std::collections::HashMap;

impl App {
    pub(super) fn run_prefetch_logic(&mut self) {
        if self.nav.is_fast_navigating() {
            return;
        }

        let prefetch_count = self.settings_state.prefetch_count as usize;

        // 1. Update sliding window protection in caches
        if let Some(doc) = self.nav.document.as_ref()
            && let Some(current) = self.nav.current_page
        {
            let doc_id = doc.id;
            let direction = self
                .nav
                .last_navigation_direction
                .unwrap_or(NavigationDirection::Next);

            let prefetch_dir = match direction {
                NavigationDirection::Next => crate::cache::PrefetchDirection::Next,
                NavigationDirection::Previous => crate::cache::PrefetchDirection::Previous,
            };

            // Compute the prioritized sliding window
            let protections = crate::cache::compute_sliding_window_priorities(
                current,
                prefetch_dir,
                prefetch_count,
                doc.pages.len(),
            );

            // Update GPU Cache protection
            self.texture_manager
                .update_protection(doc_id, protections.clone(), &doc.pages);

            // Update CPU Cache protection (via Scheduler)
            let mut cpu_protections = HashMap::with_capacity(protections.len() * 4);
            for p in &protections {
                if let Some(meta) = doc.pages.get(p.page_id) {
                    // Protect all major mip levels in CPU cache as well
                    let mips = [
                        MipLevel::Full,
                        MipLevel::Half,
                        MipLevel::Quarter,
                        MipLevel::Eighth,
                    ];
                    for mip in mips {
                        let hash = crate::cache::gpu_uploader::cache_key(doc_id, &meta.name, mip);
                        cpu_protections.insert(hash, p.priority);
                    }
                }
            }
            self.scheduler.set_protection(cpu_protections);
        }

        // 2. Ask NavigationController for what to prefetch (standard list)
        let plan = self.nav.get_prefetch_plan(prefetch_count);
        if plan.is_empty() {
            return;
        }

        let (doc_id, fit_mode, layout_mode, rotation, window_size) = {
            let Some(doc) = self.nav.document.as_ref() else {
                return;
            };
            (
                doc.id,
                self.nav.view.fit_mode,
                self.nav.view.layout_mode,
                self.nav.view.rotation,
                self.window_size,
            )
        };

        // 3. Reuse the same target_zoom calculation logic as VisibleRequest
        let target_zoom = {
            let doc = self.nav.document.as_ref();
            let current = self.nav.current_page.unwrap_or(0);
            crate::view::layout_sync::calculate_target_zoom(
                doc,
                fit_mode,
                layout_mode,
                rotation,
                current,
                window_size,
            )
        };

        // 4. Process each prefetch target through the standard gateway
        for (page_id, priority) in plan {
            let page_animated = self
                .nav
                .document
                .as_ref()
                .and_then(|d| d.pages.get(page_id))
                .map_or(false, |p| p.is_animated);

            let mip = if page_animated {
                crate::types::MipLevel::Full
            } else {
                crate::sampling::decide_mip_level(target_zoom, false)
            };

            // Same checks as VisibleRequest: GPU -> Cache -> In-flight -> Decode
            if !self
                .texture_manager
                .has_optimal_mip(page_id, mip, page_animated)
            {
                let page_name = self
                    .nav
                    .document
                    .as_ref()
                    .and_then(|d| d.pages.get(page_id))
                    .map(|p| p.name.clone());

                if let Some(name) = page_name {
                    // Note: Prefetching doesn't typically do "immediate GPU promotion"
                    // unless we want them ready on GPU immediately.
                    // For now, just ensure they are in-flight or cached in CPU.
                    // Animated pages bypass CPU cache to ensure fresh streaming.
                    let in_cache = !page_animated
                        && self.scheduler.get_from_cache(doc_id, &name, mip).is_some();

                    if !self.scheduler.is_inflight(doc_id, &name, mip) && !in_cache {
                        crate::view::navigation_request::enqueue_page_request(
                            crate::view::navigation_request::NavigationRequestContext {
                                document: self.nav.document.as_ref(),
                                page: page_id,
                                target_zoom,
                                skip_resample: false, // Prefetch always high quality
                                priority,
                                scheduler: &mut self.scheduler,
                                reason: "PREFETCH",
                            },
                        );
                    }
                }
            }
        }
    }
}
