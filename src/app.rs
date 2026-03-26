mod command_handler;
mod document_lifecycle;
mod document_session;
mod mode_controller;
mod navigation_commit_controller;
mod prefetch_controller;
mod ui_actions;
mod update_loop;
mod visible_request;
mod webtoon_runtime;

use crate::cache::TextureManager;
use crate::color_management::ColorManagementController;
use crate::document::archive_navigator::ArchiveNavigator;
use crate::i18n::Localizer;
use crate::pipeline::*;
use crate::settings::SettingsState;
use crate::slideshow::SlideshowController;
use crate::ui::UiSnapshot;
use crate::ui::snapshot::{UiSnapshotContext, build_ui_snapshot};
use crate::ui_overlay::{EguiToastRenderer, ToastOverlay};
use crate::view::animation_controller::AnimationController;
use std::time::Instant;

/// High-level commands that the application can react to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppCommand {
    NavigatePrevious,
    NavigateNext,
    NavigateFirst,
    NavigateLast,
    NavigatePreviousArchive,
    NavigateNextArchive,
    SetFitScreen,
    SetFitWidth,
    SetFitHeight,
    CycleLayoutMode,
    ToggleFirstPageOffset,
    ZoomInStep,
    ZoomOutStep,
    RotateCCW,
    RotateCW,
    SaveManualBookmark,
    AdjustOffset(f32, f32),
    DragOffset(f32, f32),
    ResetView,
    OpenFile,
}

use crate::view::NavigationController;

pub struct App {
    pub nav: NavigationController,
    pub window_size: (u32, u32),
    pub texture_manager: TextureManager,
    pub renderer: Option<crate::renderer::Renderer>,
    pub toast_renderer: Option<EguiToastRenderer>,
    scheduler: DecodeScheduler,
    upload_queue: UploadQueue,
    animation_controller: AnimationController,
    archive_navigator: ArchiveNavigator,
    pub dual_first_page_offset: bool,
    pub localizer: Localizer,
    pub toast_overlay: ToastOverlay,
    pub warning_overlay: crate::ui_overlay::WarningOverlay,
    bookmark_service: crate::bookmark::BookmarkService,
    show_bookmark_limit_dialog: bool,
    pub show_bookmarks: bool,
    settings_state: SettingsState,
    slideshow: SlideshowController,
    show_about_dialog: bool,
    vram_capacity_mb: usize,
    ui_last_interaction: Instant,
    ui_manual_hidden: bool,
    pub ui_windows_visible: bool,
    color_management: ColorManagementController,
    renderer_recovery_requested: bool,
    database: crate::database::DatabaseService,
    last_update_instant: Instant,
    last_navigated_at: Instant,
    pending_resume_save: bool,
    move_hold_left: bool,
    move_hold_right: bool,
    move_hold_up: bool,
    move_hold_down: bool,
    pub needs_visible_check: bool,
    pub accent_color: Option<egui::Color32>,
    loading_spinner_start: Option<Instant>,
    show_loading_spinner: bool,
    pub last_window_title: String,
    // File Association
    pub file_association_states: Vec<(String, String, bool)>,
    pub file_association_icons: std::collections::HashMap<String, egui::TextureHandle>,
    pub show_file_association_window: bool,
    pub file_association_icons_initialized: bool,
}

impl App {
    pub fn wants_idle_ui_redraw(&self) -> bool {
        // Only request idle heartbeats if there is active motion or visible timer-based UI.
        // This reduces DWM wakeups when no UI updates are needed.
        self.slideshow.enabled()
            || self.toast_overlay.is_visible()
            || self.warning_overlay.is_visible()
            || self.nav.is_fast_navigating()
            || self.nav.webtoon_scroll_target_y.is_some()
            || !self.upload_queue.is_empty()
            || self.scheduler.has_any_inflight()
    }

    pub fn next_animation_redraw_deadline(&self) -> Option<Instant> {
        let visible = self.nav.get_visible_pages(self.window_size);
        self.animation_controller
            .next_redraw_deadline(&visible, Instant::now())
    }

    pub fn next_ui_auto_hide_deadline(&self) -> Option<Instant> {
        if !self.ui_windows_visible
            || self.ui_manual_hidden
            || self.settings_state.ui_auto_hide_sec >= 11
        {
            return None;
        }

        Some(
            self.ui_last_interaction
                + std::time::Duration::from_secs(self.settings_state.ui_auto_hide_sec as u64),
        )
    }
}

unsafe impl Send for App {}
unsafe impl Sync for App {}

impl App {
    pub fn new(
        scheduler: DecodeScheduler,
        upload_queue: UploadQueue,
        location: crate::settings::model::ConfigStorageLocation,
    ) -> Self {
        let database = crate::database::DatabaseService::new_with_location(location)
            .expect("Failed to initialize database");
        let accent_color = crate::util::os_colors::get_windows_accent_color();

        Self {
            nav: NavigationController::new(),
            window_size: (800, 600),
            texture_manager: TextureManager::new(),
            renderer: None,
            toast_renderer: None,
            scheduler,
            upload_queue,
            animation_controller: AnimationController::new(),
            archive_navigator: ArchiveNavigator::new(),
            dual_first_page_offset: false,
            localizer: Localizer::new("ko"),
            toast_overlay: ToastOverlay::new(),
            warning_overlay: crate::ui_overlay::WarningOverlay::new(),
            bookmark_service: crate::bookmark::BookmarkService::from_entries(
                database.load_bookmarks().unwrap_or_default(),
            ),
            show_bookmark_limit_dialog: false,
            show_bookmarks: false,
            settings_state: SettingsState::default(),
            slideshow: SlideshowController::new(),
            show_about_dialog: false,
            vram_capacity_mb: 2048,
            ui_last_interaction: Instant::now(),
            ui_manual_hidden: false,
            ui_windows_visible: true,
            color_management: ColorManagementController::new(),
            renderer_recovery_requested: false,
            database,
            last_update_instant: Instant::now(),
            last_navigated_at: Instant::now(),
            pending_resume_save: false,
            move_hold_left: false,
            move_hold_right: false,
            move_hold_up: false,
            move_hold_down: false,
            needs_visible_check: true,
            accent_color,
            loading_spinner_start: None,
            show_loading_spinner: false,
            last_window_title: "HinaView".to_string(),
            file_association_states: crate::ui::file_association::init_file_association_states(),
            file_association_icons: std::collections::HashMap::new(),
            show_file_association_window: false,
            file_association_icons_initialized: false,
        }
    }

    pub fn compute_window_title(&self) -> String {
        if let Some(doc) = &self.nav.document {
            let current = self.nav.current_page.unwrap_or(0);
            let total = doc.pages.len();

            let mut full_path = doc.path.clone();
            if let Some(page) = doc.pages.get(current) {
                // Determine true path whether it's a directory or inside an archive.
                full_path = full_path.join(&page.name);
            }

            let path_str = full_path.to_string_lossy().replace('\\', "/");
            format!("HinaView - {} [{}/{}]", path_str, current + 1, total)
        } else {
            "HinaView".to_string()
        }
    }

    pub fn set_locale(&mut self, code: &str) {
        if self.localizer.current_code() == code {
            return;
        }
        self.localizer = Localizer::new(code);
    }

    pub fn apply_settings_state(&mut self, state: SettingsState) {
        let mut normalized = crate::settings::service::normalize(state, self.max_gpu_setting_mb());
        // Slideshow running state is runtime-only and should never be restored on startup.
        normalized.slideshow_enabled = false;
        self.settings_state = normalized.clone();

        self.dual_first_page_offset = normalized.first_page_offset;
        self.apply_fit_mode_setting(normalized.fit_mode);
        self.apply_layout_mode_setting(normalized.layout_mode);
        self.slideshow.set_enabled(false);
        self.slideshow
            .set_interval_sec(normalized.slideshow_interval_sec);
        crate::cache::settings_adapter::apply_cpu_cache_limit(
            &mut self.scheduler,
            normalized.cpu_cache_mb,
        );
        crate::cache::settings_adapter::apply_gpu_cache_limit(
            &mut self.texture_manager,
            normalized.gpu_cache_mb,
        );
        self.archive_navigator.invalidate_cache();
    }

    pub fn set_vram_capacity_mb(&mut self, vram_mb: usize) {
        self.vram_capacity_mb = vram_mb.max(128);
    }

    pub fn request_renderer_recovery(&mut self) {
        self.renderer_recovery_requested = true;
    }

    pub fn take_renderer_recovery_request(&mut self) -> bool {
        std::mem::take(&mut self.renderer_recovery_requested)
    }

    pub fn apply_renderer_recovery(
        &mut self,
        window: &winit::window::Window,
        renderer: crate::renderer::Renderer,
        estimated_vram_mb: usize,
    ) {
        self.set_vram_capacity_mb(estimated_vram_mb);
        self.texture_manager.clear_page_table();
        self.texture_manager
            .set_gpu_cache(crate::cache::GpuTextureCache::new_from_vram(
                estimated_vram_mb,
            ));

        let normalized = crate::settings::service::normalize(
            self.settings_state.clone(),
            self.max_gpu_setting_mb(),
        );
        self.settings_state = normalized.clone();
        crate::cache::settings_adapter::apply_gpu_cache_limit(
            &mut self.texture_manager,
            normalized.gpu_cache_mb,
        );

        self.renderer = Some(renderer);
        if let Some(renderer) = self.renderer.as_ref() {
            self.toast_renderer = Some(crate::ui_overlay::EguiToastRenderer::new(
                window,
                &renderer.device,
                renderer.surface_config.format,
            ));
        }

        self.request_visible_pages_for_current_layout(false);
        self.renderer_recovery_requested = false;
    }

    pub fn toggle_ui_windows(&mut self) {
        self.ui_manual_hidden = !self.ui_manual_hidden;
        self.ui_windows_visible = !self.ui_manual_hidden;
        if self.ui_windows_visible {
            self.ui_last_interaction = Instant::now();
        }
    }

    pub fn register_ui_activity(&mut self, unhide_windows: bool) {
        self.ui_last_interaction = Instant::now();
        if unhide_windows && !self.ui_manual_hidden {
            self.ui_windows_visible = true;
        }
    }

    pub fn export_settings_state(&self) -> SettingsState {
        let mut state = self.settings_state.clone();
        // Persist interval only; always start in stopped state.
        state.slideshow_enabled = false;
        state
    }

    pub fn ui_snapshot(&self) -> UiSnapshot {
        build_ui_snapshot(UiSnapshotContext {
            nav: &self.nav,
            settings: &self.settings_state,
            localizer: &self.localizer,
            bookmark_service: &self.bookmark_service,
            scheduler: &self.scheduler,
            texture_manager: &self.texture_manager,
            color_management: &self.color_management,
            slideshow: &self.slideshow,
            show_bookmark_limit_dialog: self.show_bookmark_limit_dialog,
            show_bookmarks: self.show_bookmarks,
            show_about_dialog: self.show_about_dialog,
            ui_windows_visible: self.ui_windows_visible,
            max_gpu_setting_mb: self.max_gpu_setting_mb(),
            dual_first_page_offset: self.dual_first_page_offset,
            accent_color: self.accent_color,
            show_loading_spinner: self.show_loading_spinner,
            file_association_states: self.file_association_states.clone(),
            show_file_association_window: self.show_file_association_window,
        })
    }

    pub fn current_filter_params(&self) -> crate::filter::FilterParams {
        self.settings_state.filters
    }

    pub fn current_icc_gamma_correction(&self) -> f32 {
        let source_profile = self
            .nav
            .document
            .as_ref()
            .and_then(|doc| self.nav.current_page.and_then(|p| doc.pages.get(p)))
            .and_then(|page| page.icc_profile.as_deref());
        self.color_management
            .gamma_correction_for_source(source_profile)
    }

    /// Open a file from command line argument
    pub fn open_file_from_path(&mut self, path_str: &str) {
        use std::path::Path;
        let path = Path::new(path_str);

        if !path.exists() {
            tracing::warn!("File not found: {}", path_str);
            return;
        }

        // Stop slideshow if running
        if self.slideshow.enabled() {
            self.slideshow.set_enabled(false);
            self.settings_state.slideshow_enabled = false;
        }

        // Load the document
        match crate::app::document_lifecycle::load_document_into_app(
            path.to_path_buf(),
            &mut self.nav,
            &mut self.texture_manager,
            &mut self.scheduler,
            &mut self.animation_controller,
            &mut self.archive_navigator,
            self.window_size,
        ) {
            Ok(()) => {
                self.request_visible_pages_for_current_layout(false);
                if let Some(doc) = &self.nav.document
                    && let Some(name) = doc.path.file_name().and_then(|v| v.to_str())
                {
                    self.toast_overlay.show(self.localizer.moved_to(name), 1);
                }
            }
            Err(e) => {
                tracing::error!("Failed to open file from command line: {}", e);
                self.toast_overlay.show(format!("파일 열기 실패: {}", e), 1);
            }
        }
    }

    /// Retrieves the gamut conversion matrix and gamma correction for the current page.
    pub fn current_color_management_params(&self) -> ([[f32; 4]; 3], f32) {
        let source_profile_name = self
            .nav
            .document
            .as_ref()
            .and_then(|doc| self.nav.current_page.and_then(|p| doc.pages.get(p)))
            .and_then(|page| page.icc_profile.as_deref());
        self.color_management
            .get_params_for_source_name(source_profile_name)
    }

    pub fn save_auto_recent_on_exit(&mut self) {
        self.remember_current_document_position();
        self.flush_pending_resume_save();
        crate::bookmark::controller::save_auto_recent_bookmark(
            &self.nav,
            &mut self.bookmark_service,
            &self.database,
        );
    }

    pub fn close(&self) {
        self.database.close();
    }

    pub fn handle_window_resize(&mut self, new_size: (u32, u32)) {
        if self.window_size == new_size {
            return;
        }
        self.window_size = new_size;
        if self.nav.document.is_some() {
            self.nav.refresh_layout(self.window_size);
        } else {
            self.nav.refresh_camera();
        }
    }

    pub fn get_visible_pages(&self) -> Vec<crate::types::PageId> {
        let mut visible_pages = Vec::new();
        let Some(current_page) = self.nav.current_page else {
            return visible_pages;
        };

        visible_pages.push(current_page);

        // Include pending target if navigating
        if let Some(pending) = self.nav.pending_page {
            if pending != current_page {
                visible_pages.push(pending);
            }
        }

        // Include companion pages (Dual mode) or viewport-overlapping pages (Webtoon)
        match self.nav.view.layout_mode {
            crate::types::LayoutMode::VerticalScroll => {
                if let Some(layout) = self.nav.layout.as_ref() {
                    let zoom = self.nav.camera.zoom.max(0.0001);
                    let half_w = self.window_size.0 as f32 / (2.0 * zoom);
                    let half_h = self.window_size.1 as f32 / (2.0 * zoom);
                    let margin_y = half_h;

                    let view_min_x = self.nav.camera.pan.x - half_w;
                    let view_max_x = self.nav.camera.pan.x + half_w;
                    let view_min_y = self.nav.camera.pan.y - half_h - margin_y;
                    let view_max_y = self.nav.camera.pan.y + half_h + margin_y;

                    for placement in &layout.placements {
                        let page_min_x = placement.position[0];
                        let page_max_x = placement.position[0] + placement.size[0];
                        let page_min_y = placement.position[1];
                        let page_max_y = placement.position[1] + placement.size[1];
                        let visible = !(page_max_x < view_min_x
                            || page_min_x > view_max_x
                            || page_min_y > view_max_y
                            || page_max_y < view_min_y);
                        if visible && placement.page_index != current_page {
                            visible_pages.push(placement.page_index);
                        }
                    }
                }
            }
            crate::types::LayoutMode::Dual { .. } => {
                if let Some(doc) = self.nav.document.as_ref()
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
                        visible_pages.push(partner_page);
                    }
                }
            }
            _ => {}
        }

        visible_pages
    }

    pub fn trigger_loading_indicator_if_needed(&mut self) {
        if self.loading_spinner_start.is_some() {
            return;
        }

        let target = self
            .nav
            .target_page
            .or(self.nav.pending_page)
            .or(self.nav.current_page);
        if let Some(page_id) = target {
            if !self.texture_manager.has_page(page_id) {
                self.loading_spinner_start = Some(Instant::now());
            }
        }
    }

    pub fn update_loading_spinner(&mut self) {
        let start_time = match self.loading_spinner_start {
            Some(t) => t,
            None => {
                self.show_loading_spinner = false;
                return;
            }
        };

        // Check if the current visible pages (including pending/target) are all ready
        let target = self
            .nav
            .target_page
            .or(self.nav.pending_page)
            .or(self.nav.current_page);
        let all_ready = if let Some(page_id) = target {
            self.texture_manager.has_page(page_id)
        } else {
            true
        };

        if all_ready {
            self.loading_spinner_start = None;
            self.show_loading_spinner = false;
        } else if start_time.elapsed() >= std::time::Duration::from_millis(100) {
            // Show spinner after 100ms of waiting to avoid flickering on fast loads
            self.show_loading_spinner = true;
        }
    }
}
