use super::app_config::{AppConfig, WindowConfig};
use winit::dpi::{PhysicalPosition, PhysicalSize};

#[derive(Debug, Clone, Copy)]
pub struct WindowSnapshot {
    pub size: PhysicalSize<u32>,
    pub position: PhysicalPosition<i32>,
}

impl WindowSnapshot {
    pub fn into_config_window(self) -> WindowConfig {
        WindowConfig {
            width: self.size.width,
            height: self.size.height,
            x: self.position.x,
            y: self.position.y,
        }
    }
}

pub fn snapshot_from_window(window: &winit::window::Window) -> Option<WindowSnapshot> {
    let size = window.inner_size();
    let position = window.outer_position().ok()?;

    if size.width == 0 || size.height == 0 {
        return None;
    }

    Some(WindowSnapshot { size, position })
}

pub fn apply_window_to_config(config: &mut AppConfig, snapshot: WindowSnapshot) {
    config.window = snapshot.into_config_window();
}
