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

pub fn load_config() -> AppConfig {
    // Try to load from app directory first
    let app_dir_path = config_path(ConfigStorageLocation::AppDir).ok();
    if let Some(path) = &app_dir_path {
        if let Ok(text) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&text) {
                // Use the saved config_storage_location from file
                return config;
            }
        }
    }

    // Fall back to system config directory
    let system_config_path = config_path(ConfigStorageLocation::SystemConfig).ok();
    if let Some(path) = &system_config_path {
        if let Ok(text) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&text) {
                // Use the saved config_storage_location from file
                return config;
            }
        }
    }

    // No config exists - return default with AppDir
    let mut default_config = AppConfig::default();
    default_config.locale = detect_os_locale();
    default_config.settings.config_storage_location = ConfigStorageLocation::AppDir;
    default_config
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let location = config.settings.config_storage_location;
    let dir = get_config_base_dir(location)?;
    fs::create_dir_all(&dir)?;
    let path = dir.join("config.json");
    let text = serde_json::to_string_pretty(config)?;
    fs::write(path, text)?;
    Ok(())
}
