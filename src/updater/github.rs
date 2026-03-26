// src/updater/github.rs
// GitHub API client for checking releases

use crate::updater::{GitHubRelease, UpdateInfo};

const OWNER: &str = "LazyGamerStudio";
const REPO: &str = "HinaView";

/// Check for updates on GitHub
pub fn check_for_updates() -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
    let current_version = crate::updater::get_current_version();
    tracing::info!("[Updater] Current version: v{}", current_version);

    // GitHub API URL
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        OWNER, REPO
    );
    tracing::debug!("[Updater] GitHub API URL: {}", url);

    // Make request
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "HinaView-Updater")
        .send()
        .map_err(|e| {
            tracing::error!("[Updater] GitHub API request failed: {}", e);
            e
        })?;

    if !response.status().is_success() {
        tracing::warn!(
            "[Updater] GitHub API returned status: {}",
            response.status()
        );
        return Ok(None); // Silently fail
    }

    let release: GitHubRelease = response.json().map_err(|e| {
        tracing::error!("[Updater] Failed to parse GitHub response: {}", e);
        e
    })?;

    tracing::debug!("[Updater] Latest release: {}", release.tag_name);

    // Parse version from tag (e.g., "v0.7.0" -> "0.7.0")
    let new_version = release.tag_name.trim_start_matches('v');

    // Compare versions
    if is_newer_version(&current_version, new_version) {
        tracing::info!("[Updater] Found newer version: v{}", new_version);

        // Prefer a Windows x64 package, but keep a fallback for differently named release assets.
        if let Some(asset) = release
            .assets
            .iter()
            .find(|a| is_preferred_windows_asset(&a.name))
            .or_else(|| {
                release
                    .assets
                    .iter()
                    .find(|a| is_fallback_zip_asset(&a.name))
            })
        {
            tracing::info!("[Updater] Found Windows x64 asset: {}", asset.name);
            return Ok(Some(UpdateInfo {
                version: new_version.to_string(),
                download_url: asset.browser_download_url.clone(),
                zip_name: asset.name.clone(),
            }));
        } else {
            tracing::warn!("[Updater] No Windows x64 asset found");
        }
    } else {
        tracing::info!("[Updater] Current version is up to date");
    }

    Ok(None)
}

fn is_preferred_windows_asset(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".zip")
        && lower.contains("windows")
        && (lower.contains("x64") || lower.contains("amd64") || lower.contains("win64"))
}

fn is_fallback_zip_asset(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".zip") && lower.contains("hinaview")
}

/// Check if new version is newer than current
fn is_newer_version(current: &str, new: &str) -> bool {
    use semver::Version;

    let current_ver = Version::parse(current).ok();
    let new_ver = Version::parse(new).ok();

    match (current_ver, new_ver) {
        (Some(c), Some(n)) => {
            tracing::debug!(
                "[Updater] Version compare: {} vs {} -> {}",
                current,
                new,
                n > c
            );
            n > c
        }
        _ => {
            tracing::warn!("[Updater] Failed to parse versions: {} vs {}", current, new);
            false
        }
    }
}
