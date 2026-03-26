// src/updater/worker.rs
// Background update worker

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

/// Spawn background update worker
pub fn spawn_update_worker() {
    thread::spawn(|| {
        tracing::info!("[Updater] Worker started - will check for updates in 5 seconds");

        // Wait for app to stabilize
        thread::sleep(Duration::from_secs(5));

        tracing::info!("[Updater] Checking for updates...");

        // Check for updates
        match crate::updater::github::check_for_updates() {
            Ok(Some(update_info)) => {
                tracing::info!("[Updater] New version available: v{}", update_info.version);

                // Download update
                let update_temp = get_update_temp_dir();
                let zip_path = update_temp.join(&update_info.zip_name);
                let extracted = update_temp.join("extracted");

                // Ensure temp directory exists
                if let Err(e) = std::fs::create_dir_all(&update_temp) {
                    tracing::error!("[Updater] Failed to create temp directory: {}", e);
                    return;
                }

                let _ = std::fs::remove_file(&zip_path);
                let _ = std::fs::remove_dir_all(&extracted);

                tracing::info!(
                    "[Updater] Downloading update from: {}",
                    update_info.download_url
                );

                // Download
                match crate::updater::downloader::download(&update_info.download_url, &zip_path) {
                    Ok(()) => {
                        tracing::info!("[Updater] Download complete: {:?}", zip_path);
                    }
                    Err(e) => {
                        tracing::error!("[Updater] Download failed: {}", e);
                        return;
                    }
                }

                // Extract
                tracing::info!("[Updater] Extracting update...");
                match crate::updater::extractor::extract(&zip_path, &extracted) {
                    Ok(()) => {
                        tracing::info!("[Updater] Extract complete: {:?}", extracted);
                    }
                    Err(e) => {
                        tracing::error!("[Updater] Extract failed: {}", e);
                        let _ = std::fs::remove_file(&zip_path);
                        return;
                    }
                }

                // Create flag file
                match crate::updater::flag::create_flag(&update_info.version, &extracted) {
                    Ok(()) => {
                        tracing::info!(
                            "[Updater] Update v{} ready - will apply on restart",
                            update_info.version
                        );
                    }
                    Err(e) => {
                        tracing::error!("[Updater] Failed to create update flag: {}", e);
                    }
                }
            }
            Ok(None) => {
                tracing::info!("[Updater] No updates available - using latest version");
            }
            Err(e) => {
                tracing::error!("[Updater] Update check failed: {}", e);
            }
        }

        tracing::info!("[Updater] Worker finished");
    });
}

/// Get update_temp directory path
fn get_update_temp_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("update_temp")
}
