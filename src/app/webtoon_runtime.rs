use super::App;
use crate::view::webtoon_scroll::{
    WebtoonScrollContext, process_webtoon_scroll_controller,
    queue_webtoon_scroll_page_delta_controller, queue_webtoon_scroll_to_page_controller,
};

impl App {
    pub(super) fn process_webtoon_scroll(&mut self, dt_sec: f32) -> bool {
        let changed = process_webtoon_scroll_controller(WebtoonScrollContext {
            nav: &mut self.nav,
            holds: (
                self.move_hold_left,
                self.move_hold_right,
                self.move_hold_up,
                self.move_hold_down,
            ),
            scroll_speed: self.settings_state.webtoon_scroll_speed_px_per_sec,
            dt_sec,
            window_height: self.window_size.1 as f32,
        });

        if changed {
            // Check visibility after scroll.
            // Note: The controller doesn't have access to the full pipeline, but App does.
            self.request_visible_pages_for_current_layout(false);
        }

        changed
    }

    pub(super) fn queue_webtoon_scroll_page_delta(&mut self, delta: i32, is_pressed: bool) {
        queue_webtoon_scroll_page_delta_controller(&mut self.nav, delta, is_pressed);
    }

    pub(super) fn queue_webtoon_scroll_to_page(
        &mut self,
        page: usize,
        is_pressed: bool,
        honor_existing: bool,
    ) {
        queue_webtoon_scroll_to_page_controller(&mut self.nav, page, is_pressed, honor_existing);
    }
}
