// src/runtime/input/mouse.rs
// Mouse input handling: zoom-on-wheel, drag-scroll, and click detection

use crate::app::App;
use crate::app::AppCommand;
use crate::runtime::input::keyboard::is_ctrl_pressed;
use crate::runtime::window_state::WindowState;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::window::Window;

pub fn handle_cursor_moved(
    state: &Option<Arc<RwLock<App>>>,
    window: &Option<Arc<Window>>,
    window_state: &mut WindowState,
    position: winit::dpi::PhysicalPosition<f64>,
) {
    if let Some(window) = window {
        window_state.register_cursor_activity(window);
    }

    if window_state.is_dragging {
        let dx = position.x - window_state.cursor_position.x;
        let dy = position.y - window_state.cursor_position.y;

        if let Some(state) = state {
            let mut app = state.write();
            app.register_ui_activity(true);
            app.handle_command(AppCommand::DragOffset(dx as f32, dy as f32), false, true);
        }
        if let Some(window) = window {
            window.request_redraw();
        }
    }

    window_state.cursor_position = position;
}

pub fn handle_mouse_wheel(
    state: &Option<Arc<RwLock<App>>>,
    window: &Option<Arc<Window>>,
    window_state: &mut WindowState,
    delta: MouseScrollDelta,
) {
    if let Some(window) = window {
        window_state.register_cursor_activity(window);
    }

    let (_dx, dy) = match delta {
        MouseScrollDelta::LineDelta(x, y) => (x, y),
        MouseScrollDelta::PixelDelta(pos) => (pos.x as f32 / 10.0, pos.y as f32 / 10.0),
    };

    if let Some(state) = state {
        let mut app = state.write();
        app.register_ui_activity(true);

        // Check if UI (settings window, etc.) wants the pointer input.
        // If it does, skip the main window's navigation/zoom logic.
        let ui_wants_pointer = app
            .toast_renderer
            .as_ref()
            .map_or(false, |r| r.wants_pointer_input());

        if !ui_wants_pointer {
            let ctrl_pressed = is_ctrl_pressed();

            if dy > 0.0 {
                if ctrl_pressed {
                    app.handle_command(AppCommand::ZoomInStep, false, true);
                } else {
                    app.handle_command(AppCommand::NavigatePrevious, false, true);
                }
            } else if dy < 0.0 {
                if ctrl_pressed {
                    app.handle_command(AppCommand::ZoomOutStep, false, true);
                } else {
                    app.handle_command(AppCommand::NavigateNext, false, true);
                }
            }
        }
    }

    if let Some(window) = window {
        window.request_redraw();
    }
}

pub fn handle_mouse_input(
    state: &Option<Arc<RwLock<App>>>,
    window: &Option<Arc<Window>>,
    window_state: &mut WindowState,
    button_state: ElementState,
    button: MouseButton,
) {
    if let Some(window) = window {
        window_state.register_cursor_activity(window);
    }

    if button == MouseButton::Left || button == MouseButton::Middle {
        let is_pressed = button_state == ElementState::Pressed;

        if is_pressed {
            // Check for double click
            let now = Instant::now();
            let is_double_click = if let Some(last_time) = window_state.last_click_instant
                && window_state.last_click_button == Some(button)
                && now.duration_since(last_time).as_millis() < 300
            {
                true
            } else {
                false
            };

            window_state.last_click_instant = Some(now);
            window_state.last_click_button = Some(button);

            if is_double_click && button == MouseButton::Left {
                if let Some(state) = state {
                    let mut app = state.write();
                    app.handle_command(AppCommand::ResetView, false, true);
                }
            }

            // Check if UI wants the pointer before starting a drag
            let ui_wants_pointer = if let Some(state) = state {
                let app = state.read();
                app.toast_renderer
                    .as_ref()
                    .map_or(false, |r| r.wants_pointer_input())
            } else {
                false
            };

            if !ui_wants_pointer {
                window_state.is_dragging = true;
            }
        } else {
            window_state.is_dragging = false;
        }
    }
}
