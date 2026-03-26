use super::app_config::AppConfig;
use crate::settings::model::ConfigStorageLocation;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Get the base directory for configuration storage
fn get_config_base_dir(location: ConfigStorageLocation) -> Result<PathBuf> {
    match location {
        ConfigStorageLocation::AppDir => {
            let exe_path = std::env::current_exe()
                .map_err(|e| anyhow::anyhow!("Failed to get executable path: {}", e))?;
            let app_dir = exe_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?
                .to_path_buf();
            Ok(app_dir.join("config"))
        }
        ConfigStorageLocation::SystemConfig => {
            let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
            Ok(base.join("HinaView"))
        }
    }
}

fn config_path(location: ConfigStorageLocation) -> Result<PathBuf> {
    let dir = get_config_base_dir(location)?;
    Ok(dir.join("config.json"))
}

fn locale_exists(code: &str) -> bool {
    PathBuf::from("assets")
        .join("lang")
        .join(format!("{code}.json"))
        .exists()
}

fn resolve_supported_locale(code: &str) -> String {
    let normalized = code.replace('_', "-");
    if locale_exists(&normalized) {
        return normalized;
    }

    if let Some((base, _)) = normalized.split_once('-')
        && locale_exists(base)
    {
        return base.to_string();
    }

    "en".to_string()
}

#[cfg(target_os = "windows")]
fn detect_os_locale() -> String {
    use windows_sys::Win32::Globalization::GetUserDefaultLocaleName;

    let mut buffer = [0u16; 85];
    let len = unsafe {
        // SAFETY: `buffer` points to writable UTF-16 storage sized for LOCALE_NAME_MAX_LENGTH.
        GetUserDefaultLocaleName(buffer.as_mut_ptr(), buffer.len() as i32)
    };

    if len <= 1 {
        return "en".to_string();
    }

    let locale = String::from_utf16_lossy(&buffer[..(len as usize - 1)]);
    resolve_supported_locale(&locale)
}

#[cfg(not(target_os = "windows"))]
fn detect_os_locale() -> String {
    if let Ok(locale) = std::env::var("LANG") {
        let locale = locale.split('.').next().unwrap_or("en");
        resolve_supported_locale(locale)
    } else {
        "en".to_string()
    }
}

/// Loads the application configuration from prioritized storage locations.
///
/// This function first attempts to load the configuration from the application
/// executable's directory. If not found, it falls back to the system's standard
/// configuration directory (e.g., AppData on Windows). If no configuration file
/// exists, a default configuration with auto-detected locale is returned.
pub fn load_config() -> (AppConfig, ConfigStorageLocation) {
    // Try to load from app directory first
    let app_dir_path = config_path(ConfigStorageLocation::AppDir).ok();
    if let Some(path) = &app_dir_path
        && let Ok(text) = fs::read_to_string(path)
        && let Ok(config) = serde_json::from_str::<AppConfig>(&text)
    {
        // Use the saved config_storage_location from file
        return (config, ConfigStorageLocation::AppDir);
    }

    // Fall back to system config directory
    let system_config_path = config_path(ConfigStorageLocation::SystemConfig).ok();
    if let Some(path) = &system_config_path
        && let Ok(text) = fs::read_to_string(path)
        && let Ok(config) = serde_json::from_str::<AppConfig>(&text)
    {
        // Use the saved config_storage_location from file
        return (config, ConfigStorageLocation::SystemConfig);
    }

    // No config exists - return default with AppDir
    let default_config = AppConfig {
        locale: detect_os_locale(),
        settings: crate::settings::SettingsState {
            config_storage_location: ConfigStorageLocation::AppDir,
            ..crate::settings::SettingsState::default()
        },
        ..AppConfig::default()
    };
    (default_config, ConfigStorageLocation::AppDir)
}

/// Persists the application configuration to the filesystem.
pub fn save_config(config: &AppConfig) -> Result<()> {
    let location = config.settings.config_storage_location;
    let dir = get_config_base_dir(location)?;
    fs::create_dir_all(&dir)?;
    let path = dir.join("config.json");
    let text = serde_json::to_string_pretty(config)?;
    fs::write(path, text)?;
    Ok(())
}

/// Moves configuration and database files from source to target location.
pub fn migrate_config(source: ConfigStorageLocation, target: ConfigStorageLocation) -> Result<()> {
    if source == target {
        return Ok(());
    }

    let src_dir = get_config_base_dir(source)?;
    let dst_dir = get_config_base_dir(target)?;

    if !src_dir.exists() {
        return Ok(());
    }

    fs::create_dir_all(&dst_dir)?;

    // We don't move config.json here because save_config will write it to the new location anyway.
    // However, we MUST delete the old config.json so load_config doesn't find it first next time.
    let src_config = src_dir.join("config.json");
    if src_config.exists() {
        let _ = fs::remove_file(src_config);
    }

    // Move hinaview.db and its sidecars (WAL, SHM)
    let db_files = ["hinaview.db", "hinaview.db-wal", "hinaview.db-shm"];
    for file in db_files {
        let s = src_dir.join(file);
        let d = dst_dir.join(file);
        if s.exists() {
            // Overwrite if exists in target
            fs::copy(&s, &d)?;
            let _ = fs::remove_file(s);
        }
    }

    // After moving all recognized files, attempt to remove the source directory if it's empty.
    let _ = fs::remove_dir(src_dir);

    Ok(())
}
