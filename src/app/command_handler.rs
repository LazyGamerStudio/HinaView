use super::{App, AppCommand};
use crate::settings::FitModeSetting;
use crate::view::FitMode;

impl App {
    fn stop_slideshow_on_manual_navigation(&mut self, is_pressed: bool) {
        if !is_pressed || !self.slideshow.enabled() {
            return;
        }
        self.slideshow.set_enabled(false);
        self.settings_state.slideshow_enabled = false;
    }

    pub fn handle_command(&mut self, command: AppCommand, is_repeat: bool, is_pressed: bool) {
        match command {
            AppCommand::NavigatePrevious => {
                self.stop_slideshow_on_manual_navigation(is_pressed);
                if matches!(
                    self.nav.view.layout_mode,
                    crate::types::LayoutMode::VerticalScroll
                ) {
                    if is_pressed && !is_repeat {
                        self.queue_webtoon_scroll_page_delta(-1, true);
                    }
                    if is_pressed {
                        return;
                    }
                }
                self.nav.navigate_step(-1, is_repeat, is_pressed);
                let visible = self.get_visible_pages();
                self.animation_controller.retain_visible(&visible);

                // High-speed navigation (Chasing Navigation):
                // In fast navigation mode (repeat or held keys), we prioritize the final destination
                // by skipping heavy decodes of intermediate pages. The 'fast' flag ensures
                // efficient resource management and a lag-free visual experience.
                self.request_visible_pages_for_current_layout(
                    is_repeat || self.nav.is_fast_navigating(),
                );

                // Real-time UI feedback:
                // Even when decodes are pending, the quick page indicator provides immediate
                // feedback on the current/target page index, maintaining perceived responsiveness.
                if is_repeat || self.nav.is_fast_navigating() {
                    self.show_quick_page_indicator();
                }

                if is_pressed && !is_repeat {
                    self.trigger_loading_indicator_if_needed();
                }
            }
            AppCommand::NavigateNext => {
                self.stop_slideshow_on_manual_navigation(is_pressed);
                if matches!(
                    self.nav.view.layout_mode,
                    crate::types::LayoutMode::VerticalScroll
                ) {
                    if is_pressed && !is_repeat {
                        self.queue_webtoon_scroll_page_delta(1, true);
                    }
                    if is_pressed {
                        return;
                    }
                }
                self.nav.navigate_step(1, is_repeat, is_pressed);
                let visible = self.get_visible_pages();
                self.animation_controller.retain_visible(&visible);

                // High-speed navigation (Chasing Navigation):
                // Same logic as NavigatePrevious. We skip intensive tasks for intermediate pages
                // to keep up with high-frequency user input (e.g., holding down arrow keys).
                self.request_visible_pages_for_current_layout(
                    is_repeat || self.nav.is_fast_navigating(),
                );

                // Consistent UI feedback:
                // Provides immediate visual confirmation of movement across the document,
                // regardless of the current decoding status of the underlying images.
                if is_repeat || self.nav.is_fast_navigating() {
                    self.show_quick_page_indicator();
                }

                if is_pressed && !is_repeat {
                    self.trigger_loading_indicator_if_needed();
                }
            }
            AppCommand::NavigateFirst => {
                self.stop_slideshow_on_manual_navigation(is_pressed);
                if matches!(
                    self.nav.view.layout_mode,
                    crate::types::LayoutMode::VerticalScroll
                ) {
                    if is_pressed {
                        self.queue_webtoon_scroll_to_page(0, true, false);
                    }
                    return;
                }
                if is_pressed {
                    // For Dual mode, navigate to first spread directly
                    if matches!(
                        self.nav.view.layout_mode,
                        crate::types::LayoutMode::Dual { .. }
                    ) {
                        if let Some(doc) = &self.nav.document
                            && let Some(first_spread) = doc.spreads.first()
                        {
                            let target_page = first_spread.left.or(first_spread.right).unwrap_or(0);
                            self.nav.navigate(target_page);
                            let visible = self.get_visible_pages();
                            self.animation_controller.retain_visible(&visible);
                            self.request_visible_pages_for_current_layout(false);
                            self.show_quick_page_indicator();
                            return;
                        }
                    }
                    // For Single mode, use original logic
                    let current_page = self.nav.current_page.unwrap_or(0);
                    self.nav
                        .navigate_step(-(current_page as i32), false, is_pressed);
                    let visible = self.get_visible_pages();
                    self.animation_controller.retain_visible(&visible);
                    self.request_visible_pages_for_current_layout(false);
                    self.show_quick_page_indicator();
                }
            }
            AppCommand::NavigateLast => {
                self.stop_slideshow_on_manual_navigation(is_pressed);
                if matches!(
                    self.nav.view.layout_mode,
                    crate::types::LayoutMode::VerticalScroll
                ) {
                    if is_pressed
                        && let Some(doc) = &self.nav.document
                        && !doc.pages.is_empty()
                    {
                        let last = doc.pages.len() - 1;
                        self.queue_webtoon_scroll_to_page(last, true, false);
                    }
                    return;
                }
                if is_pressed {
                    // For Dual mode, navigate to last spread directly
                    if matches!(
                        self.nav.view.layout_mode,
                        crate::types::LayoutMode::Dual { .. }
                    ) {
                        if let Some(doc) = &self.nav.document
                            && let Some(last_spread) = doc.spreads.last()
                        {
                            // RTL 모드에서는 왼쪽 페이지가 마지막, LTR 에서는 오른쪽 페이지가 마지막
                            let target_page = match self.nav.view.layout_mode {
                                crate::types::LayoutMode::Dual { rtl: true, .. } => {
                                    last_spread.left.or(last_spread.right)
                                }
                                _ => last_spread.right.or(last_spread.left),
                            }
                            .unwrap_or_else(|| doc.pages.len() - 1);
                            self.nav.navigate(target_page);
                            let visible = self.get_visible_pages();
                            self.animation_controller.retain_visible(&visible);
                            self.request_visible_pages_for_current_layout(false);
                            self.show_quick_page_indicator();
                            return;
                        }
                    }
                    // For Single mode, use original logic
                    if let Some(doc) = &self.nav.document {
                        let last_page = doc.pages.len().saturating_sub(1) as i32;
                        let current_page = self.nav.current_page.unwrap_or(0) as i32;
                        self.nav
                            .navigate_step(last_page - current_page, false, is_pressed);
                        let visible = self.get_visible_pages();
                        self.animation_controller.retain_visible(&visible);
                        self.request_visible_pages_for_current_layout(false);
                        self.show_quick_page_indicator();
                    }
                }
            }
            AppCommand::NavigatePreviousArchive => {
                self.stop_slideshow_on_manual_navigation(is_pressed);
                if is_pressed && !is_repeat {
                    self.navigate_neighbor_archive(-1)
                }
            }
            AppCommand::NavigateNextArchive => {
                self.stop_slideshow_on_manual_navigation(is_pressed);
                if is_pressed && !is_repeat {
                    self.navigate_neighbor_archive(1)
                }
            }
            AppCommand::SetFitScreen => {
                if is_pressed {
                    self.apply_fit_mode(FitMode::FitScreen);
                    self.settings_state.fit_mode = FitModeSetting::FitScreen;
                    self.toast_overlay.show(self.localizer.fit_screen(), 1);
                }
            }
            AppCommand::SetFitWidth => {
                if is_pressed {
                    self.apply_fit_mode(FitMode::FitWidth);
                    self.settings_state.fit_mode = FitModeSetting::FitWidth;
                    self.toast_overlay.show(self.localizer.fit_width(), 1);
                }
            }
            AppCommand::SetFitHeight => {
                if is_pressed {
                    self.apply_fit_mode(FitMode::FitHeight);
                    self.settings_state.fit_mode = FitModeSetting::FitHeight;
                    self.toast_overlay.show(self.localizer.fit_height(), 1);
                }
            }
            AppCommand::CycleLayoutMode => {
                if is_pressed && !is_repeat {
                    self.cycle_layout_mode();
                }
            }
            AppCommand::ToggleFirstPageOffset => {
                if is_pressed && !is_repeat {
                    self.toggle_first_page_offset();
                }
            }
            AppCommand::ZoomInStep => {
                if is_pressed {
                    let next = crate::view::zoom_policy::zoom_in_step(self.nav.view.zoom);
                    self.apply_fixed_zoom(next);
                    self.settings_state.fit_mode = FitModeSetting::Zoom;
                    self.toast_overlay
                        .show(self.localizer.zoom_percent(self.nav.view.zoom), 1);
                }
            }
            AppCommand::ZoomOutStep => {
                if is_pressed {
                    let next = crate::view::zoom_policy::zoom_out_step(self.nav.view.zoom);
                    self.apply_fixed_zoom(next);
                    self.settings_state.fit_mode = FitModeSetting::Zoom;
                    self.toast_overlay
                        .show(self.localizer.zoom_percent(self.nav.view.zoom), 1);
                }
            }
            AppCommand::RotateCCW | AppCommand::RotateCW => {
                if !is_pressed || is_repeat {
                    return;
                }

                if matches!(
                    self.nav.view.layout_mode,
                    crate::types::LayoutMode::Dual { .. }
                        | crate::types::LayoutMode::VerticalScroll
                ) {
                    self.warning_overlay
                        .show(self.localizer.rotation_not_supported_in_dual());
                    return;
                }

                match command {
                    AppCommand::RotateCCW => self.nav.view.rotation.rotate_ccw(),
                    AppCommand::RotateCW => self.nav.view.rotation.rotate_cw(),
                    _ => {}
                }
                self.nav.refresh_layout(self.window_size);
            }
            AppCommand::SaveManualBookmark => {
                if is_pressed && !is_repeat {
                    match crate::bookmark::controller::save_manual_bookmark(
                        &self.nav,
                        &mut self.bookmark_service,
                        &self.database,
                    ) {
                        Ok(()) => {
                            self.toast_overlay
                                .show(self.localizer.bookmark_saved().to_string(), 1);
                        }
                        Err(crate::bookmark::BookmarkError::ManualLimitExceeded) => {
                            self.show_bookmark_limit_dialog = true;
                        }
                    }
                }
            }
            AppCommand::AdjustOffset(dx, dy) => {
                if dx < 0.0 {
                    self.move_hold_left = is_pressed;
                } else if dx > 0.0 {
                    self.move_hold_right = is_pressed;
                }
                if matches!(
                    self.nav.view.layout_mode,
                    crate::types::LayoutMode::VerticalScroll
                ) {
                    if dy > 0.0 {
                        self.move_hold_up = is_pressed;
                    } else if dy < 0.0 {
                        self.move_hold_down = is_pressed;
                    }
                    if is_pressed && dy != 0.0 {
                        // Holding W/S takes precedence over one-shot smooth step.
                        self.nav.webtoon_scroll_target_y = None;
                    }
                    return;
                }
                if dy > 0.0 {
                    self.move_hold_up = is_pressed;
                } else if dy < 0.0 {
                    self.move_hold_down = is_pressed;
                }
                return;
            }
            AppCommand::DragOffset(dx, dy) => {
                let scaled_dx = dx / self.nav.view.zoom;
                let scaled_dy = dy / self.nav.view.zoom;
                // Drag behavior: invert X axis, keep Y axis direction.
                if matches!(
                    self.nav.view.layout_mode,
                    crate::types::LayoutMode::VerticalScroll
                ) {
                    self.nav.webtoon_scroll_target_y = None;
                } else {
                    self.nav.view.image_offset[0] -= scaled_dx;
                }
                self.nav.view.image_offset[1] += scaled_dy;
                self.nav.refresh_camera();
            }
            AppCommand::OpenFile => {}
            AppCommand::ResetView => {
                if is_pressed {
                    self.nav.view.rotation = crate::view::RotationQuarter::Deg0;
                    self.nav.view.image_offset = [0.0, 0.0];
                    self.nav.view.fit_mode = FitMode::FitScreen;
                    self.settings_state.fit_mode = FitModeSetting::FitScreen;

                    self.nav.refresh_layout(self.window_size);
                    self.needs_visible_check = true;
                    self.toast_overlay
                        .show(self.localizer.view_mode().view_reset.clone(), 1);
                }
            }
        }
    }
}
