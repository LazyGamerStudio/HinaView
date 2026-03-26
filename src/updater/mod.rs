// src/updater/mod.rs
// Auto-update module for HinaView

pub mod downloader;
pub mod extractor;
pub mod flag;
pub mod github;
pub mod worker;

use serde::Deserialize;
use std::process::Command;

/// GitHub release API response
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GitHubAsset>,
}

/// GitHub asset information
#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// Update information
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub zip_name: String,
}

/// Get current version from Cargo.toml
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn try_start_pending_update() -> bool {
    if !flag::has_flag() {
        return false;
    }

    let updater_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("updater.exe")));

    let Some(updater_path) = updater_path else {
        tracing::error!("[Updater] Failed to resolve updater.exe path");
        return false;
    };

    if !updater_path.exists() {
        tracing::error!(
            "[Updater] Pending update found but updater executable is missing: {:?}",
            updater_path
        );
        return false;
    }

    match Command::new(&updater_path).spawn() {
        Ok(_) => {
            tracing::info!("[Updater] Started updater executable: {:?}", updater_path);
            true
        }
        Err(e) => {
            tracing::error!("[Updater] Failed to start updater executable: {}", e);
            false
        }
    }
}

/// Clean up leftover updater.exe.old after a self-update
pub fn cleanup_old_updater() {
    let old_updater = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("updater.exe.old")));

    if let Some(path) = old_updater {
        if path.exists() {
            tracing::info!("[Updater] Found old updater executable, cleaning up: {:?}", path);
            if let Err(e) = std::fs::remove_file(&path) {
                tracing::warn!("[Updater] Failed to remove old updater: {}", e);
            } else {
                tracing::info!("[Updater] Successfully removed old updater.");
            }
        }
    }
}
