use super::App;
use crate::settings::{FitModeSetting, LayoutModeSetting};
use crate::view::FitMode;

impl App {
    pub(super) fn apply_fit_mode_setting(&mut self, mode: FitModeSetting) {
        match mode {
            FitModeSetting::FitScreen => self.apply_fit_mode(FitMode::FitScreen),
            FitModeSetting::FitWidth => self.apply_fit_mode(FitMode::FitWidth),
            FitModeSetting::FitHeight => self.apply_fit_mode(FitMode::FitHeight),
            FitModeSetting::Zoom => self.apply_fixed_zoom(self.nav.view.zoom.max(0.05)),
        }
    }

    pub(super) fn apply_layout_mode_setting(&mut self, mode: LayoutModeSetting) {
        self.nav.view.layout_mode = match mode {
            LayoutModeSetting::Single => crate::types::LayoutMode::Single,
            LayoutModeSetting::DualLtr => crate::types::LayoutMode::Dual {
                rtl: false,
                first_page_offset: self.dual_first_page_offset,
            },
            LayoutModeSetting::DualRtl => crate::types::LayoutMode::Dual {
                rtl: true,
                first_page_offset: self.dual_first_page_offset,
            },
            LayoutModeSetting::VerticalScroll => crate::types::LayoutMode::VerticalScroll,
        };
        self.nav.refresh_layout(self.window_size);
        self.needs_visible_check = true;
    }

    pub(super) fn max_gpu_setting_mb(&self) -> usize {
        ((self.vram_capacity_mb / 2).max(64) / 64) * 64
    }

    pub(super) fn apply_fit_mode(&mut self, mode: FitMode) {
        self.nav.view.fit_mode = mode;
        self.nav.refresh_layout(self.window_size);
        self.needs_visible_check = true;
    }

    pub(super) fn apply_fixed_zoom(&mut self, zoom: f32) {
        self.nav.view.fit_mode = FitMode::Fixed(zoom);
        self.nav.refresh_layout(self.window_size);
        self.needs_visible_check = true;
    }

    pub(super) fn cycle_layout_mode(&mut self) {
        let target_mode = crate::view::layout_mode_cycle::cycle_layout_mode(
            self.nav.view.layout_mode,
            self.dual_first_page_offset,
        );
        self.settings_state.layout_mode = match target_mode {
            crate::types::LayoutMode::Single => LayoutModeSetting::Single,
            crate::types::LayoutMode::Dual { rtl: false, .. } => LayoutModeSetting::DualLtr,
            crate::types::LayoutMode::Dual { rtl: true, .. } => LayoutModeSetting::DualRtl,
            crate::types::LayoutMode::VerticalScroll => LayoutModeSetting::VerticalScroll,
        };
        self.apply_layout_mode_transition(target_mode);
        let message = self.localizer.layout_mode_label(target_mode);
        self.toast_overlay.show(message, 1);
    }

    pub(super) fn apply_layout_mode_transition(&mut self, target_mode: crate::types::LayoutMode) {
        let mut effective_target = target_mode;
        if matches!(effective_target, crate::types::LayoutMode::VerticalScroll)
            && self.is_current_page_animated()
        {
            effective_target = crate::types::LayoutMode::Single;
            self.settings_state.layout_mode = LayoutModeSetting::Single;
            self.warning_overlay
                .show(self.localizer.webtoon_not_supported_for_animated());
        }
        let entering_dual = !matches!(
            self.nav.view.layout_mode,
            crate::types::LayoutMode::Dual { .. }
        ) && matches!(effective_target, crate::types::LayoutMode::Dual { .. });

        if entering_dual {
            self.complete_metadata_before_dual_switch();
        }
        if matches!(effective_target, crate::types::LayoutMode::Dual { .. }) {
            // Dual mode is always rendered at 0 degrees by product requirement.
            self.nav.view.rotation = crate::view::RotationQuarter::Deg0;
        }
        if matches!(effective_target, crate::types::LayoutMode::VerticalScroll) {
            // Webtoon mode policy: no rotation.
            self.nav.view.rotation = crate::view::RotationQuarter::Deg0;
        }
        self.nav.webtoon_scroll_target_y = None;
        self.nav.webtoon_last_request_pan_y = None;
        self.move_hold_left = false;
        self.move_hold_right = false;
        self.move_hold_up = false;
        self.move_hold_down = false;

        // Always reset image offset when changing layout modes to prevent huge drifting offsets.
        self.nav.view.image_offset = [0.0, 0.0];

        self.nav.view.layout_mode = effective_target;
        self.nav.refresh_layout(self.window_size);

        // Webtoon mode's refresh_layout preserves pan Y to prevent jerky scrolling max/min on resize.
        // But on layout MODE transition, we MUST center on the current page, otherwise the old pan Y
        // (e.g. from Single mode) places us at the very top of the webtoon timeline (Page 0).
        if matches!(effective_target, crate::types::LayoutMode::VerticalScroll) {
            if let Some(curr) = self.nav.current_page {
                self.nav.center_camera_on_page(curr);
            }
        }

        self.needs_visible_check = true;
    }

    fn is_current_page_animated(&self) -> bool {
        self.nav
            .document
            .as_ref()
            .and_then(|doc| self.nav.current_page.and_then(|p| doc.pages.get(p)))
            .is_some_and(|page| page.is_animated)
    }

    pub(super) fn enforce_single_mode_for_animated_in_webtoon(&mut self) -> bool {
        if !matches!(
            self.nav.view.layout_mode,
            crate::types::LayoutMode::VerticalScroll
        ) {
            return false;
        }
        if !self.is_current_page_animated() {
            return false;
        }
        self.settings_state.layout_mode = LayoutModeSetting::Single;
        self.warning_overlay
            .show(self.localizer.webtoon_not_supported_for_animated());
        self.apply_layout_mode_transition(crate::types::LayoutMode::Single);
        true
    }

    fn complete_metadata_before_dual_switch(&mut self) {
        const DUAL_SCAN_FAST_HEADER_LIMIT: usize = crate::document::archive::METADATA_HEADER_SIZE;
        const DUAL_SCAN_DEEP_HEADER_LIMIT: usize = 131_072;

        let Some(doc) = self.nav.document.as_mut() else {
            return;
        };

        let total = doc.pages.len();
        if total == 0 {
            return;
        }

        let reader = doc.reader.clone();
        let mut last_percent = 0usize;
        let mut updated = 0usize;
        let mut failed = 0usize;

        for idx in 0..total {
            let (needs_probe, name) = {
                let page = &doc.pages[idx];
                (
                    page.width == 0 || page.height == 0 || page.metadata_probe_failed,
                    page.name.clone(),
                )
            };

            if needs_probe {
                let mut width = 0u32;
                let mut height = 0u32;
                let mut is_animated = false;

                if let Ok(header) = reader.read_file_partial(&name, DUAL_SCAN_FAST_HEADER_LIMIT)
                    && let Some(meta) = crate::document::format_probe::probe_image_metadata(&header)
                {
                    width = meta.width;
                    height = meta.height;
                    is_animated = meta.is_animated;
                }

                if (width == 0 || height == 0)
                    && let Ok(header) = reader.read_file_partial(&name, DUAL_SCAN_DEEP_HEADER_LIMIT)
                    && let Some(meta) = crate::document::format_probe::probe_image_metadata(&header)
                {
                    width = meta.width;
                    height = meta.height;
                    is_animated = meta.is_animated;
                }

                if (width == 0 || height == 0)
                    && let Ok(bytes) = reader.read_file(&name)
                {
                    if let Some(meta) = crate::document::format_probe::probe_image_metadata(&bytes)
                    {
                        width = meta.width;
                        height = meta.height;
                        is_animated = meta.is_animated;
                    } else {
                        let (decoded, _) = crate::pipeline::decode::decode_bytes(
                            &bytes,
                            crate::types::MipLevel::Full, // MipLevel::Full for full metadata extraction
                        );
                        if let Ok(decoded) = decoded {
                            width = decoded.original_width;
                            height = decoded.original_height;
                        }
                    }
                }

                if let Some(page) = doc.pages.get_mut(idx) {
                    if width > 0 && height > 0 {
                        page.width = width;
                        page.height = height;
                        page.is_wide = width > height;
                        page.is_animated = is_animated;
                        page.metadata_probe_failed = false;
                        updated += 1;
                    } else {
                        page.metadata_probe_failed = true;
                        failed += 1;
                    }
                }
            }

            if total > 100 {
                let percent = ((idx + 1) * 100) / total;
                if percent > last_percent {
                    tracing::debug!(
                        "[DualSwitch] metadata progress: {}/100 ({}/{})",
                        percent,
                        idx + 1,
                        total
                    );
                    last_percent = percent;
                }
            }
        }

        if total > 100 {
            tracing::debug!(
                "[DualSwitch] metadata scan complete: updated={}, failed={}, total={}",
                updated,
                failed,
                total
            );
        }
    }

    pub(super) fn toggle_first_page_offset(&mut self) {
        self.dual_first_page_offset = !self.dual_first_page_offset;
        self.settings_state.first_page_offset = self.dual_first_page_offset;
        if let crate::types::LayoutMode::Dual { rtl, .. } = self.nav.view.layout_mode {
            self.nav.view.layout_mode = crate::types::LayoutMode::Dual {
                rtl,
                first_page_offset: self.dual_first_page_offset,
            };
            self.nav.refresh_layout(self.window_size);
            self.needs_visible_check = true;
        }

        if self.dual_first_page_offset {
            self.toast_overlay
                .show(self.localizer.first_page_offset_on(), 1);
        } else {
            self.toast_overlay
                .show(self.localizer.first_page_offset_off(), 1);
        }
    }
}
