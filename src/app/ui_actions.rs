use super::{App, AppCommand};
use crate::settings::{FitModeSetting, LayoutModeSetting};
use crate::ui::{UiAction, UiFitMode, UiLayoutMode};
use crate::view::FitMode;

impl App {
    pub fn handle_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::OpenBookmark(id) => self.handle_open_bookmark(id),
            UiAction::DeleteBookmark(id) => crate::bookmark::controller::delete_bookmark(
                &mut self.bookmark_service,
                &self.database,
                id,
            ),
            UiAction::DismissBookmarkLimitDialog => self.show_bookmark_limit_dialog = false,
            UiAction::SetFitMode(mode) => self.set_fit_mode_from_ui(mode),
            UiAction::SetLayoutMode(mode) => self.set_layout_mode_from_ui(mode),
            UiAction::SetFirstPageOffset(value) => {
                self.set_first_page_offset(value);
                self.needs_visible_check = true;
            }
            UiAction::SetSlideshowEnabled(value) => {
                self.settings_state.slideshow_enabled = value;
                self.slideshow.set_enabled(value);
                if value {
                    let msg = self.localizer.slideshow_started().to_string();
                    self.toast_overlay.show(msg, 1);
                } else {
                    let msg = self.localizer.slideshow_stopped().to_string();
                    self.toast_overlay.show(msg, 1);
                }
            }
            UiAction::SetSlideshowIntervalSec(sec) => {
                let sec = crate::settings::service::clamp_slideshow_sec(sec);
                self.settings_state.slideshow_interval_sec = sec;
                self.slideshow.set_interval_sec(sec);
            }
            UiAction::SetCpuCacheMb(mb) => {
                let mb = crate::settings::service::clamp_cpu_cache_mb(mb);
                self.settings_state.cpu_cache_mb = mb;
                crate::cache::settings_adapter::apply_cpu_cache_limit(&mut self.scheduler, mb);
            }
            UiAction::SetGpuCacheMb(mb) => {
                let mb =
                    crate::settings::service::clamp_gpu_cache_mb(mb, self.max_gpu_setting_mb());
                self.settings_state.gpu_cache_mb = mb;
                crate::cache::settings_adapter::apply_gpu_cache_limit(
                    &mut self.texture_manager,
                    mb,
                );
            }
            UiAction::SetUiAutoHideSec(sec) => {
                self.settings_state.ui_auto_hide_sec =
                    crate::settings::service::clamp_auto_hide_sec(sec);
            }
            UiAction::SetPrefetchCount(count) => {
                self.settings_state.prefetch_count = count.clamp(3, 10);
            }
            UiAction::SetRememberDocumentPosition(value) => {
                self.settings_state.remember_document_position = value;
            }
            UiAction::SetWebtoonScrollSpeed(value) => {
                self.settings_state.webtoon_scroll_speed_px_per_sec =
                    crate::settings::service::clamp_webtoon_scroll_speed_px_per_sec(value);
            }
            UiAction::ToggleAboutDialog(value) => self.show_about_dialog = value,
            UiAction::SetInfoSectionOpen(value) => self.settings_state.sections_open.info = value,
            UiAction::SetExifSectionOpen(value) => self.settings_state.sections_open.exif = value,
            UiAction::SetViewModeSectionOpen(value) => {
                self.settings_state.sections_open.view_mode = value
            }
            UiAction::SetFilterSectionOpen(value) => {
                self.settings_state.sections_open.filter = value
            }
            UiAction::SetPreferenceSectionOpen(value) => {
                self.settings_state.sections_open.preference = value
            }
            UiAction::SetSettingsWindowCollapsed(value) => {
                self.settings_state.settings_window_collapsed = value
            }
            UiAction::SetShortcutsWindowCollapsed(value) => {
                self.settings_state.shortcuts_window_collapsed = value
            }
            UiAction::SetBookmarkDrawerOpen(value) => {
                self.show_bookmarks = value;
            }
            UiAction::SetFilterBypassColor(value) => {
                self.settings_state.filters.bypass_color = value;
            }
            UiAction::SetFilterBypassMedian(value) => {
                self.settings_state.filters.bypass_median = value;
            }
            UiAction::SetFilterBypassFsr(value) => {
                self.settings_state.filters.bypass_fsr = value;
            }
            UiAction::SetFilterBypassDetail(value) => {
                self.settings_state.filters.bypass_detail = value;
            }
            UiAction::SetFilterBypassLevels(value) => {
                self.settings_state.filters.bypass_levels = value;
            }
            UiAction::SetFilterBright(value) => {
                self.settings_state.filters.bright = value.clamp(-1.0, 1.0);
            }
            UiAction::SetFilterContrast(value) => {
                self.settings_state.filters.contrast = value.clamp(0.0, 2.0);
            }
            UiAction::SetFilterGamma(value) => {
                self.settings_state.filters.gamma = value.clamp(0.2, 3.0);
            }
            UiAction::SetFilterExposure(value) => {
                self.settings_state.filters.exposure = value.clamp(-4.0, 4.0);
            }
            UiAction::SetFilterFsrSharpness(value) => {
                self.settings_state.filters.fsr_sharpness = value.clamp(0.0, 1.0);
            }
            UiAction::SetFilterMedianStrength(value) => {
                self.settings_state.filters.median_strength = value.clamp(0.0, 1.0);
            }
            UiAction::SetFilterMedianStride(value) => {
                self.settings_state.filters.median_stride = value.clamp(1.0, 5.0);
            }
            UiAction::SetFilterBlurRadius(value) => {
                self.settings_state.filters.blur_radius = value.clamp(0.0, 5.0);
            }
            UiAction::SetFilterUnsharpAmount(value) => {
                self.settings_state.filters.unsharp_amount = value.clamp(0.0, 2.0);
            }
            UiAction::SetFilterUnsharpThreshold(value) => {
                self.settings_state.filters.unsharp_threshold = value.clamp(0.0, 1.0);
            }
            UiAction::SetFilterLevelsInBlack(value) => {
                self.settings_state.filters.levels_in_black = value.clamp(0.0, 1.0);
            }
            UiAction::SetFilterLevelsInWhite(value) => {
                self.settings_state.filters.levels_in_white = value.clamp(0.0, 1.0);
            }
            UiAction::SetFilterLevelsGamma(value) => {
                self.settings_state.filters.levels_gamma = value.clamp(0.1, 5.0);
            }
            UiAction::SetFilterLevelsOutBlack(value) => {
                self.settings_state.filters.levels_out_black = value.clamp(0.0, 1.0);
            }
            UiAction::SetFilterLevelsOutWhite(value) => {
                self.settings_state.filters.levels_out_white = value.clamp(0.0, 1.0);
            }
            UiAction::ResetFilterColor => {
                let d = crate::filter::FilterParams::default();
                self.settings_state.filters.bright = d.bright;
                self.settings_state.filters.contrast = d.contrast;
                self.settings_state.filters.gamma = d.gamma;
                self.settings_state.filters.exposure = d.exposure;
            }
            UiAction::ResetFilterMedian => {
                let d = crate::filter::FilterParams::default();
                self.settings_state.filters.median_strength = d.median_strength;
                self.settings_state.filters.median_stride = d.median_stride;
            }
            UiAction::ResetFilterFsr => {
                let d = crate::filter::FilterParams::default();
                self.settings_state.filters.fsr_sharpness = d.fsr_sharpness;
            }
            UiAction::ResetFilterDetail => {
                let d = crate::filter::FilterParams::default();
                self.settings_state.filters.blur_radius = d.blur_radius;
                self.settings_state.filters.unsharp_amount = d.unsharp_amount;
                self.settings_state.filters.unsharp_threshold = d.unsharp_threshold;
            }
            UiAction::ResetFilterLevels => {
                let d = crate::filter::FilterParams::default();
                self.settings_state.filters.levels_in_black = d.levels_in_black;
                self.settings_state.filters.levels_in_white = d.levels_in_white;
                self.settings_state.filters.levels_gamma = d.levels_gamma;
                self.settings_state.filters.levels_out_black = d.levels_out_black;
                self.settings_state.filters.levels_out_white = d.levels_out_white;
            }
            UiAction::ResetFilters => {
                self.settings_state.filters = crate::filter::FilterParams::default();
            }
            UiAction::ResetView => {
                self.handle_command(AppCommand::ResetView, false, true);
            }
            UiAction::SetLocale(code) => {
                self.set_locale(&code);
            }
            UiAction::SetArchiveSortingMode(mode) => {
                self.settings_state.archive_sorting_mode = match mode {
                    crate::ui::UiArchiveSortingMode::Mixed => {
                        crate::settings::model::ArchiveSortingMode::Mixed
                    }
                    crate::ui::UiArchiveSortingMode::FoldersFirst => {
                        crate::settings::model::ArchiveSortingMode::FoldersFirst
                    }
                };
                self.archive_navigator.invalidate_cache();
            }
            UiAction::SetThemeMode(mode) => {
                self.settings_state.theme_mode = match mode {
                    crate::ui::UiThemeMode::Auto => crate::settings::model::ThemeModeSetting::Auto,
                    crate::ui::UiThemeMode::Dark => crate::settings::model::ThemeModeSetting::Dark,
                    crate::ui::UiThemeMode::Light => {
                        crate::settings::model::ThemeModeSetting::Light
                    }
                };
            }
            UiAction::SetSingleInstanceMode(enabled) => {
                self.settings_state.single_instance = enabled;
            }
            UiAction::SetConfigStorageLocation(location) => {
                self.settings_state.config_storage_location = location;
            }
            UiAction::ShowFileAssociationWindow(show) => {
                self.show_file_association_window = show;
            }
            UiAction::UpdateFileAssociation(ext, is_associated) => {
                for (state_ext, _, state_associated) in &mut self.file_association_states {
                    if state_ext == &ext {
                        *state_associated = is_associated;
                        break;
                    }
                }
            }
            UiAction::ApplyFileAssociations => {
                self.apply_file_associations();
                self.show_file_association_window = false;
            }
            UiAction::SelectAllFileAssociations(select_all) => {
                for (_, _, is_associated) in &mut self.file_association_states {
                    *is_associated = select_all;
                }
            }
            UiAction::DeleteAllFileAssociations => {
                if let Err(e) = crate::system::win_registry::unregister_all() {
                    tracing::error!("Failed to delete file associations: {}", e);
                    self.toast_overlay
                        .show(self.localizer.file_association_failed(&e), 1);
                } else {
                    for (_, _, is_associated) in &mut self.file_association_states {
                        *is_associated = false;
                    }
                    self.toast_overlay
                        .show(self.localizer.file_association_deleted(), 1);
                }
            }
            UiAction::AddContextMenu => {
                let text = self.localizer.preference().context_menu_item_text.clone();

                // Register for file extensions
                if let Err(e) = crate::system::win_registry::register_all_context_menus(&text) {
                    tracing::error!("Failed to register context menu: {}", e);
                    self.toast_overlay
                        .show(self.localizer.context_menu_register_failed(&e), 1);
                    return;
                }

                // Register for directories
                if let Err(e) = crate::system::win_registry::register_directory_context_menu(&text)
                {
                    tracing::error!("Failed to register directory context menu: {}", e);
                    self.toast_overlay
                        .show(self.localizer.directory_context_menu_register_failed(&e), 1);
                    return;
                }

                self.toast_overlay
                    .show(self.localizer.context_menu_registered(), 1);
            }
            UiAction::DeleteContextMenu => {
                // Unregister for file extensions
                if let Err(e) = crate::system::win_registry::unregister_all_context_menus() {
                    tracing::error!("Failed to unregister context menu: {}", e);
                    self.toast_overlay
                        .show(self.localizer.context_menu_unregister_failed(&e), 1);
                    return;
                }

                // Unregister for directories
                if let Err(e) = crate::system::win_registry::unregister_directory_context_menu() {
                    tracing::error!("Failed to unregister directory context menu: {}", e);
                    self.toast_overlay.show(
                        self.localizer.directory_context_menu_unregister_failed(&e),
                        1,
                    );
                    return;
                }

                self.toast_overlay
                    .show(self.localizer.context_menu_unregistered(), 1);
            }
            UiAction::RegisterStartMenuShortcut => {
                if let Err(e) = crate::system::start_menu::register_shortcut() {
                    tracing::error!("Failed to register start menu shortcut: {}", e);
                    self.toast_overlay
                        .show(self.localizer.start_menu_register_failed(&e), 1);
                } else {
                    self.toast_overlay
                        .show(self.localizer.start_menu_registered(), 1);
                }
            }
            UiAction::UnregisterStartMenuShortcut => {
                if let Err(e) = crate::system::start_menu::unregister_shortcut() {
                    tracing::error!("Failed to unregister start menu shortcut: {}", e);
                    self.toast_overlay
                        .show(self.localizer.start_menu_unregister_failed(&e), 1);
                } else {
                    self.toast_overlay
                        .show(self.localizer.start_menu_unregistered(), 1);
                }
            }
        }
    }

    fn set_fit_mode_from_ui(&mut self, mode: UiFitMode) {
        match mode {
            UiFitMode::FitScreen => {
                self.settings_state.fit_mode = FitModeSetting::FitScreen;
                self.apply_fit_mode(FitMode::FitScreen);
            }
            UiFitMode::FitWidth => {
                self.settings_state.fit_mode = FitModeSetting::FitWidth;
                self.apply_fit_mode(FitMode::FitWidth);
            }
            UiFitMode::FitHeight => {
                self.settings_state.fit_mode = FitModeSetting::FitHeight;
                self.apply_fit_mode(FitMode::FitHeight);
            }
            UiFitMode::Zoom => {
                self.settings_state.fit_mode = FitModeSetting::Zoom;
                self.apply_fixed_zoom(self.nav.view.zoom.max(0.05));
            }
        }
    }

    fn set_layout_mode_from_ui(&mut self, mode: UiLayoutMode) {
        match mode {
            UiLayoutMode::Single => {
                self.settings_state.layout_mode = LayoutModeSetting::Single;
                self.apply_layout_mode_transition(crate::types::LayoutMode::Single);
            }
            UiLayoutMode::DualLtr => {
                self.settings_state.layout_mode = LayoutModeSetting::DualLtr;
                self.apply_layout_mode_transition(crate::types::LayoutMode::Dual {
                    rtl: false,
                    first_page_offset: self.dual_first_page_offset,
                });
            }
            UiLayoutMode::DualRtl => {
                self.settings_state.layout_mode = LayoutModeSetting::DualRtl;
                self.apply_layout_mode_transition(crate::types::LayoutMode::Dual {
                    rtl: true,
                    first_page_offset: self.dual_first_page_offset,
                });
            }
            UiLayoutMode::VerticalScroll => {
                self.settings_state.layout_mode = LayoutModeSetting::VerticalScroll;
                self.apply_layout_mode_transition(crate::types::LayoutMode::VerticalScroll);
            }
        }
    }

    fn set_first_page_offset(&mut self, enabled: bool) {
        self.dual_first_page_offset = enabled;
        self.settings_state.first_page_offset = enabled;
        if let crate::types::LayoutMode::Dual { rtl, .. } = self.nav.view.layout_mode {
            self.nav.view.layout_mode = crate::types::LayoutMode::Dual {
                rtl,
                first_page_offset: enabled,
            };
            self.nav.refresh_layout(self.window_size);
        }
    }

    fn handle_open_bookmark(&mut self, id: u64) {
        let Some(entry) =
            crate::bookmark::controller::get_bookmark_for_opening(&self.bookmark_service, id)
        else {
            return;
        };
        self.show_bookmark_limit_dialog = false;
        self.remember_current_document_position();

        let target_path = entry.path.clone();
        let target_page = entry.page_index;

        if self.slideshow.enabled() {
            self.slideshow.set_enabled(false);
            self.settings_state.slideshow_enabled = false;
        }

        if self
            .nav
            .document
            .as_ref()
            .is_some_and(|d| d.path == target_path)
        {
            if let Some(current) = self.nav.current_page {
                let delta = target_page as i32 - current as i32;
                if delta != 0 {
                    self.nav.navigate_step(delta, false, true);
                }
            }
            return;
        }

        match crate::app::document_lifecycle::load_document_into_app(
            target_path,
            &mut self.nav,
            &mut self.texture_manager,
            &mut self.scheduler,
            &mut self.animation_controller,
            &mut self.archive_navigator,
            self.window_size,
        ) {
            Ok(()) => {
                self.nav.navigate(target_page);
                self.request_visible_pages_for_current_layout(false);
                if let Some(doc) = &self.nav.document
                    && let Some(name) = doc.path.file_name().and_then(|v| v.to_str())
                {
                    self.toast_overlay.show(self.localizer.moved_to(name), 1);
                }
            }
            Err(e) => {
                tracing::error!("[App] Failed to open bookmark: {}", e);
            }
        }
    }

    fn apply_file_associations(&mut self) {
        let exts_to_associate: Vec<String> = self
            .file_association_states
            .iter()
            .filter(|(_, _, is_associated)| *is_associated)
            .map(|(ext, _, _)| ext.clone())
            .collect();

        let exts_to_disassociate: Vec<String> = self
            .file_association_states
            .iter()
            .filter(|(_, _, is_associated)| !*is_associated)
            .map(|(ext, _, _)| ext.clone())
            .collect();

        // Apply associations directly (no admin required with HKCU)
        if let Err(e) = crate::system::win_registry::update_associations(
            &exts_to_associate,
            &exts_to_disassociate,
        ) {
            tracing::error!("Failed to apply file associations: {}", e);
            self.toast_overlay
                .show(self.localizer.file_association_apply_failed(&e), 1);
        } else {
            self.toast_overlay
                .show(self.localizer.file_association_applied(), 1);
        }
    }
}
