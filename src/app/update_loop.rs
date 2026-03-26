use super::App;
use super::navigation_commit_controller::finalize_navigation_state_controller;
use crate::pipeline::upload_processor::{UploadContext, process_upload_queue_controller};
use crate::view::animation_processor::process_animations_controller;
use std::time::Instant;

impl App {
    pub fn update(&mut self) -> bool {
        let mut gpu_updated = false;
        let now = Instant::now();
        let dt_sec = (now - self.last_update_instant)
            .as_secs_f32()
            .clamp(0.0, 0.05);
        self.last_update_instant = now;

        self.toast_overlay.update();
        self.warning_overlay.update();
        self.update_loading_spinner();
        self.ensure_current_page_file_size();
        self.enforce_ui_auto_hide();

        // 1. Stop slideshow BEFORE checking advance conditions if we are at the last page.
        // This prevents attempting to navigate past the end during fast 0s slideshows.
        gpu_updated |= self.stop_slideshow_if_at_last_page();

        // 2. Check for newly committed pages.
        let committed = self.finalize_navigation_state();
        if committed {
            self.request_visible_pages_for_current_layout(self.nav.is_fast_navigating());
        }

        // 3. Determine if slideshow should advance.
        let mut slideshow_should_advance = false;
        let slideshow_enabled = self.slideshow.enabled();

        if slideshow_enabled && !committed {
            // Single hashmap lookup for both has_animation and completed_loops
            let (has_animation, completed_loops) = self
                .nav
                .current_page
                .map(|id| self.animation_controller.get_animation_status(id))
                .unwrap_or((false, 0));

            if has_animation {
                // Animated page: use loop count (1:1 with interval_sec, max 5)
                // completed_loops now increments AFTER each loop completes.
                let target_loops = self.settings_state.slideshow_interval_sec.min(5);

                if completed_loops >= target_loops {
                    slideshow_should_advance = true;
                    self.slideshow.reset_tick();
                }
            } else if self.slideshow.should_advance() {
                // Static page: use time interval
                slideshow_should_advance = true;
                self.slideshow.reset_tick();
            }
        }

        // 4. Perform navigation if needed.
        if slideshow_should_advance {
            self.nav.navigate_step(1, false, true);
            // Explicitly request decode for the new pending page
            self.request_visible_pages_for_current_layout(false);
            gpu_updated = true;
        }

        gpu_updated |= self.process_webtoon_scroll(dt_sec);
        gpu_updated |= self.process_upload_queue();
        gpu_updated |= self.enforce_single_mode_for_animated_in_webtoon();

        let prefetch_needed = self.nav.maybe_run_deferred_prefetch(&self.texture_manager);
        // FAST NAVIGATION OPTIMIZATION: Skip prefetching while the user is rapidly
        // scrolling/navigating. Prefetching during high-speed movement causes
        // I/O congestion and queue bloat with pages that are bypassed before they can be displayed.
        if prefetch_needed && !self.nav.is_fast_navigating() {
            self.run_prefetch_logic();
            self.scheduler.restore_default_decode_limit();
        }

        // Note: Slideshow timer is reset in the slideshow_should_advance block above.
        // We do NOT reset it on every commit - only when slideshow actually advances.

        gpu_updated |= committed;
        gpu_updated |= self.process_animations();

        // 7. During fast navigation, aggressively drop far-away GPU pages so animated Full-mip
        // textures do not stay resident across long page runs.
        if self.nav.is_fast_navigating() {
            let keep_window = (self.settings_state.prefetch_count as usize + 3).max(8);
            self.nav
                .evict_gpu_cache_far_pages(&mut self.texture_manager, keep_window);
            if let Some(renderer) = &self.renderer {
                let _ = renderer.device.poll(wgpu::PollType::Poll);
            }
        }

        // 8. Visibility check: Only perform if requested by an event (dirty flag)
        if self.needs_visible_check && self.nav.document.is_some() {
            self.request_visible_pages_for_current_layout(false); // Normal quality check
            self.needs_visible_check = false;
        }

        // 6. Debounced Resume Save: Flush only after 2s of inactivity
        if self.pending_resume_save
            && self.last_navigated_at.elapsed() >= std::time::Duration::from_secs(2)
        {
            self.flush_pending_resume_save();
        }

        gpu_updated
    }

    fn ensure_current_page_file_size(&mut self) {
        let Some(page_id) = self.nav.current_page else {
            return;
        };

        let Some(doc) = self.nav.document.as_mut() else {
            return;
        };

        let need_lookup = doc
            .pages
            .get(page_id)
            .map(|p| p.file_size_bytes.is_none())
            .unwrap_or(false);
        if !need_lookup {
            return;
        }

        let page_name = match doc.pages.get(page_id) {
            Some(p) => p.name.clone(),
            None => return,
        };
        let size = doc.reader.file_size_bytes(&page_name);
        if let Some(page) = doc.pages.get_mut(page_id) {
            page.file_size_bytes = size;
        }
    }

    fn enforce_ui_auto_hide(&mut self) {
        let hide_sec = self.settings_state.ui_auto_hide_sec;
        if hide_sec >= 11 || !self.ui_windows_visible {
            return;
        }
        if self.ui_last_interaction.elapsed() >= std::time::Duration::from_secs(hide_sec as u64) {
            self.ui_windows_visible = false;
        }
    }

    fn process_upload_queue(&mut self) -> bool {
        process_upload_queue_controller(UploadContext {
            upload_queue: &self.upload_queue,
            scheduler: &mut self.scheduler,
            nav: &mut self.nav,
            texture_manager: &mut self.texture_manager,
            renderer: self.renderer.as_ref(),
            animation_controller: &mut self.animation_controller,
            window_size: self.window_size,
            needs_visible_check: &mut self.needs_visible_check,
        })
    }

    fn finalize_navigation_state(&mut self) -> bool {
        finalize_navigation_state_controller(
            &mut self.nav,
            &mut self.texture_manager,
            &mut self.scheduler,
            self.window_size,
            &mut self.animation_controller,
        )
    }

    fn process_animations(&mut self) -> bool {
        let visible = self.get_visible_pages();
        process_animations_controller(
            &mut self.animation_controller,
            &visible,
            &self.texture_manager,
            self.renderer.as_ref(),
        )
    }

    fn stop_slideshow_if_at_last_page(&mut self) -> bool {
        if !self.slideshow.enabled() {
            return false;
        }
        let (Some(doc), Some(current)) = (self.nav.document.as_ref(), self.nav.current_page) else {
            return false;
        };
        let last = doc.pages.len().saturating_sub(1);
        if current < last {
            return false;
        }

        self.slideshow.set_enabled(false);
        self.settings_state.slideshow_enabled = false;
        true
    }

    pub(super) fn show_quick_page_indicator(&mut self) {
        if let Some(doc) = &self.nav.document {
            // Prioritize target_page (where we are going) over current_page (where we are)
            // for immediate feedback during fast navigation.
            let display_page = self.nav.target_page.or(self.nav.current_page);

            if let Some(idx) = display_page {
                let total = doc.pages.len();
                self.toast_overlay
                    .show(format!("{} / {}", idx + 1, total), 0);
            }
        }
    }
}
