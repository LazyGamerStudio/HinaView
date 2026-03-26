// src/updater/mod.rs
// Auto-update module for HinaView

pub mod downloader;
pub mod extractor;
pub mod flag;
pub mod github;
pub mod worker;

use serde::Deserialize;
use std::process::Command;

/// Represents the response from GitHub Releases API containing release information.
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GitHubAsset>,
}

/// Contains metadata and download URL for an individual release asset from GitHub.
#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// Encapsulates consolidated information about an available update package.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub zip_name: String,
}

/// Retrieves the current application version as defined in the Cargo package metadata.
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Checks for a pending update and attempts to launch the external updater process.
///
/// This function verifies if an update flag exists and, if so, starts the `updater.exe`
/// while passing along any relevant command-line arguments to maintain application context.
///
/// # Arguments
/// * `args` - A vector of command-line arguments to be passed through to the updated application.
///
/// # Returns
/// Returns `true` if the updater process was successfully launched, `false` otherwise.
pub fn try_start_pending_update(args: Vec<String>) -> bool {
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

    // Skip the first argument (current exe path) and pass the rest to the updater
    let passthrough_args: Vec<String> = args.into_iter().skip(1).collect();

    match Command::new(&updater_path).args(&passthrough_args).spawn() {
        Ok(_) => {
            tracing::info!(
                "[Updater] Started updater executable with {} args: {:?}",
                passthrough_args.len(),
                updater_path
            );
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

    if let Some(path) = old_updater
        && path.exists()
    {
        tracing::info!(
            "[Updater] Found old updater executable, cleaning up: {:?}",
            path
        );
        if let Err(e) = std::fs::remove_file(&path) {
            tracing::warn!("[Updater] Failed to remove old updater: {}", e);
        } else {
            tracing::info!("[Updater] Successfully removed old updater.");
        }
    }
}
