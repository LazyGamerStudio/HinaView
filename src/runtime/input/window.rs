// src/runtime/input/window.rs
// Window event handling: resizing, file-drop, and OS-level lifecycle events

use crate::app::App;
use crate::runtime::window_state::WindowState;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

pub fn handle_resize(
    state: &Option<Arc<RwLock<App>>>,
    window: &Option<Arc<Window>>,
    window_state: &mut WindowState,
    new_size: winit::dpi::PhysicalSize<u32>,
) {
    if let Some(state) = state {
        let mut app = state.write();
        app.handle_window_resize((new_size.width, new_size.height));
        if let Some(renderer) = &mut app.renderer {
            renderer.resize((new_size.width, new_size.height));
        }

        if let Some(window) = window {
            window_state.refresh_visibility(window);
            window_state.capture_windowed_state(window);
            if window_state.is_visible {
                window.request_redraw();
            }
        }
    }
}

pub fn handle_moved(window: &Option<Arc<Window>>, window_state: &mut WindowState) {
    if let Some(window) = window {
        window_state.capture_windowed_state(window);
    }
}

pub fn handle_file_drop(
    state: &Option<Arc<RwLock<App>>>,
    window: &Option<Arc<Window>>,
    path: PathBuf,
) {
    let t0 = Instant::now();
    tracing::info!("[Drop] received: {}", path.display());
    if let Some(state) = state {
        let mut app = state.write();
        app.handle_file_dropped(path);
        if let Some(window) = window {
            window.request_redraw();
        }
    }
    tracing::info!("[Drop] handled in {}ms", t0.elapsed().as_millis());
}
