// src/runtime/input/keyboard.rs
// Keyboard input handling: shortcut mapping and state management

use crate::app::App;
use crate::app::AppCommand;
use crate::input::{
    InputCommand, RuntimeCommand, is_left_right_arrow_key, map_keyboard_input_with_modifiers,
};
use crate::runtime::window_state::WindowState;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use winit::event::{ElementState, KeyEvent};
use winit::window::Window;

// Global Ctrl key state
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);

pub fn is_ctrl_pressed() -> bool {
    CTRL_PRESSED.load(Ordering::Relaxed)
}

pub fn handle_keyboard(
    state: &Option<Arc<RwLock<App>>>,
    window: &Option<Arc<Window>>,
    window_state: &mut WindowState,
    event: KeyEvent,
) {
    // Check if UI wants keyboard input (e.g. focus in a TextEdit)
    let ui_wants_keyboard = if let Some(state) = state {
        let app = state.read();
        app.toast_renderer
            .as_ref()
            .map_or(false, |r| r.wants_keyboard_input())
    } else {
        false
    };

    if ui_wants_keyboard {
        return;
    }

    let is_pressed = event.state == ElementState::Pressed;
    if is_pressed && let Some(state) = state {
        let mut app = state.write();
        let unhide = !is_left_right_arrow_key(&event.logical_key);
        app.register_ui_activity(unhide);
    }

    // Track Ctrl key state
    if matches!(
        &event.logical_key,
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Control)
    ) {
        CTRL_PRESSED.store(is_pressed, Ordering::Relaxed);
    }

    let ctrl_pressed = CTRL_PRESSED.load(Ordering::Relaxed);

    let Some(command) = map_keyboard_input_with_modifiers(&event.logical_key, ctrl_pressed) else {
        return;
    };

    match command {
        InputCommand::Runtime(runtime_cmd) => {
            if is_pressed
                && !event.repeat
                && let Some(window) = window
            {
                match runtime_cmd {
                    RuntimeCommand::ToggleFullscreen => {
                        let was_fullscreen = window.fullscreen().is_some();
                        window_state.toggle_fullscreen(window);
                        if let Some(state) = state {
                            let mut app = state.write();
                            let message = if was_fullscreen {
                                app.localizer.fullscreen_exited().to_string()
                            } else {
                                app.localizer.fullscreen_entered().to_string()
                            };
                            app.toast_overlay.show(message, 1);
                        }
                    }
                    RuntimeCommand::ToggleUiWindows => {
                        if let Some(state) = state {
                            let mut app = state.write();
                            app.toggle_ui_windows();
                            let message = if app.ui_windows_visible {
                                app.localizer.ui_windows_shown().to_string()
                            } else {
                                app.localizer.ui_windows_hidden().to_string()
                            };
                            app.toast_overlay.show(message, 1);
                        }
                    }
                }
                window.request_redraw();
            }
        }
        InputCommand::App(app_cmd) => {
            if let Some(state) = state {
                if matches!(app_cmd, AppCommand::OpenFile) && is_pressed && !event.repeat {
                    // Extract initial directory from current document if available
                    let initial_dir = {
                        let app = state.read();
                        app.nav.document.as_ref().and_then(|doc| {
                            if doc.path.is_dir() {
                                Some(doc.path.clone())
                            } else {
                                doc.path.parent().map(|p| p.to_path_buf())
                            }
                        })
                    };

                    let state_clone = state.clone();
                    let window_clone = window.clone();

                    // RFD dialog blocks, so we run it in a separate thread
                    // to prevent UI freeze and potential deadlocks with the event loop.
                    std::thread::spawn(move || {
                        let mut dialog = rfd::FileDialog::new()
                            .add_filter(
                                "All Supported",
                                &[
                                    "zip", "cbz", "jpg", "jpeg", "jfif", "png", "webp", "gif",
                                    "jxl", "heic", "heif", "avif", "bmp", "tiff", "tif", "tga",
                                    "dds", "exr", "hdr", "pnm",
                                ],
                            )
                            .add_filter("Archives", &["zip", "cbz"])
                            .add_filter(
                                "Images",
                                &[
                                    "jpg", "jpeg", "jfif", "png", "webp", "gif", "jxl", "heic",
                                    "heif", "avif", "bmp", "tiff", "tif", "tga", "dds", "exr",
                                    "hdr", "pnm",
                                ],
                            );

                        if let Some(dir) = initial_dir {
                            dialog = dialog.set_directory(dir);
                        }

                        if let Some(path) = dialog.pick_file() {
                            let mut app = state_clone.write();
                            app.handle_file_dropped(path);
                            if let Some(window) = &window_clone {
                                window.request_redraw();
                            }
                        }
                    });
                } else {
                    let mut app = state.write();
                    app.handle_command(app_cmd, event.repeat, is_pressed);
                }
            }
            if let Some(window) = window {
                window.request_redraw();
            }
        }
    }
}
