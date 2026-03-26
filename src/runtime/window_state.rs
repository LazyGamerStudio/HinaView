use crate::config::window_snapshot::{WindowSnapshot, snapshot_from_window};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::{Fullscreen, Window};

const CURSOR_HIDE_DELAY: std::time::Duration = std::time::Duration::from_secs(3);

pub struct WindowState {
    windowed_size: Option<PhysicalSize<u32>>,
    windowed_position: Option<PhysicalPosition<i32>>,
    pub cursor_position: winit::dpi::PhysicalPosition<f64>,
    cursor_visible: bool,
    last_cursor_activity: std::time::Instant,
    pub is_dragging: bool,
    pub is_visible: bool,
    is_occluded: bool,
    pub last_click_instant: Option<std::time::Instant>,
    pub last_click_button: Option<winit::event::MouseButton>,
    pub last_redraw_instant: std::time::Instant,
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            windowed_size: None,
            windowed_position: None,
            cursor_position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
            cursor_visible: true,
            last_cursor_activity: std::time::Instant::now(),
            is_dragging: false,
            is_visible: true,
            is_occluded: false,
            last_click_instant: None,
            last_click_button: None,
            last_redraw_instant: std::time::Instant::now(),
        }
    }

    pub fn set_occluded(&mut self, occluded: bool, window: &Window) {
        self.is_occluded = occluded;
        self.refresh_visibility(window);
    }

    pub fn refresh_visibility(&mut self, window: &Window) {
        let size = window.inner_size();
        let is_minimized = window.is_minimized().unwrap_or(false);
        let zero_sized = size.width == 0 || size.height == 0;
        self.is_visible = !self.is_occluded && !is_minimized && !zero_sized;
    }

    pub fn toggle_fullscreen(&mut self, window: &Window) {
        if window.fullscreen().is_some() {
            window.set_fullscreen(None);
            if let Some(size) = self.windowed_size.take() {
                let _ = window.request_inner_size(size);
            }
            if let Some(pos) = self.windowed_position.take() {
                window.set_outer_position(pos);
            }
            return;
        }

        self.capture_windowed_state(window);
        window.set_fullscreen(Some(Fullscreen::Borderless(window.current_monitor())));
    }

    pub fn capture_windowed_state(&mut self, window: &Window) {
        if window.fullscreen().is_some()
            || window.is_minimized().unwrap_or(false)
            || window.is_maximized()
        {
            return;
        }

        self.windowed_size = Some(window.inner_size());
        self.windowed_position = window.outer_position().ok();
    }

    pub fn register_cursor_activity(&mut self, window: &Window) {
        self.last_cursor_activity = std::time::Instant::now();
        if !self.cursor_visible {
            window.set_cursor_visible(true);
            self.cursor_visible = true;
        }
    }

    pub fn next_cursor_hide_deadline(&self) -> Option<std::time::Instant> {
        if !self.is_visible || !self.cursor_visible {
            return None;
        }

        Some(self.last_cursor_activity + CURSOR_HIDE_DELAY)
    }

    pub fn hide_cursor_if_idle(&mut self, window: &Window) {
        if !self.cursor_visible || !self.is_visible {
            return;
        }

        if std::time::Instant::now() >= self.last_cursor_activity + CURSOR_HIDE_DELAY {
            window.set_cursor_visible(false);
            self.cursor_visible = false;
        }
    }

    pub fn snapshot_for_persist(&self, window: &Window) -> Option<WindowSnapshot> {
        // If in fullscreen, maximized, or minimized, use the stored windowed state
        // if available; otherwise, fall back to current window state.
        if window.fullscreen().is_some()
            || window.is_minimized().unwrap_or(false)
            || window.is_maximized()
        {
            // Prefer stored windowed state if we have it
            if let Some(size) = self.windowed_size {
                let position = self.windowed_position?;
                return Some(WindowSnapshot { size, position });
            }
            // Fallback: use current window size/position
            // This handles the case where the app was maximized/fullscreen from the start
        }

        snapshot_from_window(window)
    }
}
