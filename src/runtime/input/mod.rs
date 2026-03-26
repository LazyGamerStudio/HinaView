// src/runtime/input/mod.rs
// Input handling module: categorized event handlers

pub mod keyboard;
pub mod mouse;
pub mod window;

pub use keyboard::handle_keyboard;
pub use mouse::{handle_cursor_moved, handle_mouse_input, handle_mouse_wheel};
pub use window::{handle_file_drop, handle_moved, handle_resize};
