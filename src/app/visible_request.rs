use super::App;
use crate::view::visible_request::{VisibleRequestContext, request_visible_pages_controller};

impl App {
    pub(super) fn request_visible_pages_for_current_layout(&mut self, is_fast: bool) {
        request_visible_pages_controller(VisibleRequestContext {
            nav: &mut self.nav,
            scheduler: &mut self.scheduler,
            texture_manager: &mut self.texture_manager,
            animation_controller: &self.animation_controller,
            renderer: self.renderer.as_ref(),
            window_size: self.window_size,
            is_fast,
            has_pending_results: !self.upload_queue.is_empty(),
        });
    }
}
