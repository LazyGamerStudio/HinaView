// src/updater/flag.rs
// Update flag file management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Update flag file content
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFlag {
    pub version: String,
    pub extracted_path: String,
}

/// Get the update flag file path
pub fn get_flag_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    exe_dir.join("update_temp").join("update.flag")
}

/// Create update flag file
pub fn create_flag(version: &str, extracted_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let flag = UpdateFlag {
        version: version.to_string(),
        extracted_path: extracted_path.to_string_lossy().to_string(),
    };

    let flag_path = get_flag_path();

    // Ensure parent directory exists
    if let Some(parent) = flag_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(&flag)?;
    fs::write(&flag_path, content)?;

    Ok(())
}

/// Read update flag file (unused - kept for future use)
#[allow(dead_code)]
pub fn read_flag() -> Option<UpdateFlag> {
    let flag_path = get_flag_path();
    let content = fs::read_to_string(&flag_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Delete update flag file (unused - kept for future use)
#[allow(dead_code)]
pub fn delete_flag() -> Result<(), Box<dyn std::error::Error>> {
    let flag_path = get_flag_path();
    if flag_path.exists() {
        fs::remove_file(&flag_path)?;
    }
    Ok(())
}

/// Check if update flag exists (unused - kept for future use)
#[allow(dead_code)]
pub fn has_flag() -> bool {
    get_flag_path().exists()
}
