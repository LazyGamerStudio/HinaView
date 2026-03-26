use super::App;
use super::document_lifecycle::load_document_into_app;
use tracing::error;

impl App {
    pub(super) fn navigate_neighbor_archive(&mut self, step: i32) {
        // Stop slideshow when navigating to another archive
        if self.slideshow.enabled() {
            self.slideshow.set_enabled(false);
            self.settings_state.slideshow_enabled = false;
        }

        self.remember_current_document_position();
        let current_path = match &self.nav.document {
            Some(doc) => doc.path.clone(),
            None => return,
        };

        if let Some(result) = self.archive_navigator.find_neighbor(
            &current_path,
            step,
            self.settings_state.archive_sorting_mode,
        ) {
            for folder in result.skipped_folders {
                self.warning_overlay
                    .show(self.localizer.nav_skipped_empty(&folder));
            }

            if result.looped {
                let msg = if step > 0 {
                    self.localizer.nav_looped_first()
                } else {
                    self.localizer.nav_looped_last()
                };
                self.warning_overlay.show(msg);
            }

            match load_document_into_app(
                result.path,
                &mut self.nav,
                &mut self.texture_manager,
                &mut self.scheduler,
                &mut self.animation_controller,
                &mut self.archive_navigator,
                self.window_size,
            ) {
                Ok(()) => {
                    self.restore_remembered_position_for_current_doc();
                    self.request_visible_pages_for_current_layout(false);
                }
                Err(e) => {
                    error!("[App] Failed to navigate to neighbor: {}", e);
                }
            }
        } else {
            self.warning_overlay
                .show(self.localizer.nav_no_valid_targets());
        }
    }

    pub fn handle_file_dropped(&mut self, path: std::path::PathBuf) {
        let t0 = std::time::Instant::now();
        tracing::info!("[Open] start: {}", path.display());
        self.remember_current_document_position();
        match load_document_into_app(
            path,
            &mut self.nav,
            &mut self.texture_manager,
            &mut self.scheduler,
            &mut self.animation_controller,
            &mut self.archive_navigator,
            self.window_size,
        ) {
            Ok(()) => {
                self.restore_remembered_position_for_current_doc();
                self.request_visible_pages_for_current_layout(false);
                if let Some(doc) = &self.nav.document
                    && let Some(name) = doc.path.file_name().and_then(|v| v.to_str())
                {
                    self.toast_overlay.show(self.localizer.moved_to(name), 1);
                }
                tracing::info!("[Open] success in {}ms", t0.elapsed().as_millis());
            }
            Err(e) => {
                error!("[App] Failed to initialize document: {}", e);
                self.toast_overlay
                    .show(self.localizer.load_failed().to_string(), 1);
                tracing::info!("[Open] failed in {}ms", t0.elapsed().as_millis());
            }
        }
    }

    pub(super) fn remember_current_document_position(&mut self) {
        if !self.settings_state.remember_document_position {
            return;
        }
        if self.nav.document.is_some() && self.nav.current_page.is_some() {
            self.last_navigated_at = std::time::Instant::now();
            self.pending_resume_save = true;
        }
    }

    pub fn flush_pending_resume_save(&mut self) {
        if !self.pending_resume_save {
            return;
        }
        if let Some(doc) = self.nav.document.as_ref()
            && let Some(page_index) = self.nav.current_page
        {
            let path_key = crate::util::formats::normalize_path(&doc.path);
            let _ = self.database.save_resume_position(&path_key, page_index);
            self.pending_resume_save = false;
        }
    }

    pub(super) fn restore_remembered_position_for_current_doc(&mut self) {
        if !self.settings_state.remember_document_position {
            return;
        }
        let Some(doc) = self.nav.document.as_ref() else {
            return;
        };
        let path_key = crate::util::formats::normalize_path(&doc.path);

        match self.database.load_resume_position(&path_key) {
            Ok(Some(page_index)) => {
                if page_index >= doc.pages.len() {
                    return;
                }
                if self.nav.current_page == Some(page_index) {
                    return;
                }

                self.nav.navigate(page_index);
                self.request_visible_pages_for_current_layout(false);
            }
            Ok(None) => {}
            Err(e) => {
                error!("[App] Failed to load resume position from DB: {}", e);
            }
        }
    }
}
