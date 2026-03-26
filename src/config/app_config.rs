use crate::settings::SettingsState;
use serde::{Deserialize, Serialize};

/// Default window width in pixels
pub const DEFAULT_WINDOW_WIDTH: u32 = 1280;

/// Default window height in pixels
pub const DEFAULT_WINDOW_HEIGHT: u32 = 720;

/// Default window X position in pixels
pub const DEFAULT_WINDOW_X: i32 = 100;

/// Default window Y position in pixels
pub const DEFAULT_WINDOW_Y: i32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
            x: DEFAULT_WINDOW_X,
            y: DEFAULT_WINDOW_Y,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub locale: String,
    pub settings: SettingsState,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            locale: "en".to_string(),
            settings: SettingsState::default(),
        }
    }
}
