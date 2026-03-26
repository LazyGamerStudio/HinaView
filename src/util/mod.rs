// src/util/mod.rs
pub mod formats;
pub mod math;
pub mod os_colors;
pub mod sorting;
pub mod tiling;

pub fn now_unix_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
