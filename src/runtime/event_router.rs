use crate::app::App;
use crate::config::store::save_config;
use crate::config::window_snapshot::apply_window_to_config;
use crate::runtime::input::{
    handle_cursor_moved, handle_file_drop, handle_keyboard, handle_mouse_input, handle_mouse_wheel,
    handle_moved, handle_resize,
};
use crate::runtime::redraw_handler::handle_redraw;
use crate::runtime::window_state::WindowState;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{error, info};
use winit::event::WindowEvent;
use winit::window::Window;

pub fn route_window_event(
    state: &Option<Arc<RwLock<App>>>,
    window: &Option<Arc<Window>>,
    window_state: &mut WindowState,
    config: &mut crate::config::app_config::AppConfig,
    event_loop: &winit::event_loop::ActiveEventLoop,
    event: WindowEvent,
) {
    if let (Some(state), Some(window)) = (state, window) {
        let mut app = state.write();
        if let Some(toast_renderer) = app.toast_renderer.as_mut() {
            // OPTIMIZATION: Do not pass RedrawRequested to egui here.
            // Redraws are handled explicitly in the match block below.
            // Passing it here might cause egui to return true (wants repaint)
            // which would trigger an infinite loop of request_redraw().
            if !matches!(event, WindowEvent::RedrawRequested) {
                let wants_repaint = toast_renderer.on_window_event(window, &event);
                if wants_repaint && window_state.is_visible {
                    window.request_redraw();
                }
            }
        }
    }

    match event {
        WindowEvent::CloseRequested => {
            if let Some(state) = state {
                let mut app = state.write();
                config.locale = app.localizer.current_code().to_string();
                config.settings = app.export_settings_state();
                app.save_auto_recent_on_exit();
                app.close();
            }
            if let Some(window) = window {
                if let Some(snapshot) = window_state.snapshot_for_persist(window) {
                    apply_window_to_config(config, snapshot);
                }
                if let Err(e) = save_config(config) {
                    error!("[Config] Failed to save config: {}", e);
                }
            }
            info!("[App] Close requested. Exiting...");
            event_loop.exit();
        }
        WindowEvent::Resized(new_size) => {
            handle_resize(state, window, window_state, new_size);
        }
        WindowEvent::Moved(_) => {
            handle_moved(window, window_state);
        }
        WindowEvent::KeyboardInput { event, .. } => {
            handle_keyboard(state, window, window_state, event);
        }
        WindowEvent::DroppedFile(path) => {
            handle_file_drop(state, window, path);
        }
        WindowEvent::CursorMoved { position, .. } => {
            if let Some(state) = state {
                let mut app = state.write();
                app.register_ui_activity(true);
            }
            handle_cursor_moved(state, window, window_state, position);
        }
        WindowEvent::MouseInput {
            state: button_state,
            button,
            ..
        } => {
            if let Some(state) = state {
                let mut app = state.write();
                app.register_ui_activity(true);
            }
            handle_mouse_input(state, window, window_state, button_state, button);
        }
        WindowEvent::MouseWheel { delta, .. } => {
            handle_mouse_wheel(state, window, window_state, delta);
        }
        WindowEvent::TouchpadPressure { .. } => {}
        WindowEvent::Occluded(occluded) => {
            if let Some(window) = window {
                window_state.set_occluded(occluded, window);
                if window_state.is_visible {
                    window.request_redraw();
                }
            }
        }
        WindowEvent::RedrawRequested => {
            handle_redraw(state, window, window_state);
        }
        _ => (),
    }
}
