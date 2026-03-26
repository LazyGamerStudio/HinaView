// src/system/win_registry.rs
// Windows Registry manipulation for file associations

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegDeleteKeyW, RegDeleteTreeW, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW,
};

const APP_PROG_ID_PREFIX: &str = "HinaView";

/// Get the full path to the current executable
fn get_app_exe_path() -> Option<PathBuf> {
    let mut buffer = vec![0u16; 2048];
    unsafe {
        // SAFETY: `buffer` is a writable UTF-16 buffer large enough for the requested size.
        let len = GetModuleFileNameW(
            std::ptr::null_mut(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        );
        if len > 0 {
            buffer.truncate(len as usize);
            let path_str = OsString::from_wide(&buffer);
            Some(PathBuf::from(path_str))
        } else {
            None
        }
    }
}

/// Convert a Rust string to a null-terminated wide string (UTF-16)
fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Helper function to set a registry value (creates key if it doesn't exist)
fn set_reg_value(root: HKEY, subkey: &str, value: &str) -> Result<(), String> {
    let subkey_wide = to_wide_null(subkey);
    let mut hkey: HKEY = std::ptr::null_mut();
    let mut disposition = 0;

    unsafe {
        // SAFETY: All UTF-16 buffers are null-terminated and remain alive for the duration of
        // these registry API calls. `hkey` is closed before the block exits.
        let result = RegCreateKeyExW(
            root,
            subkey_wide.as_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut hkey,
            &mut disposition,
        );

        if result != ERROR_SUCCESS {
            return Err(format!(
                "Failed to create/open key '{}': {}",
                subkey, result
            ));
        }

        let value_name = to_wide_null("");
        let value_wide = to_wide_null(value);
        let result = RegSetValueExW(
            hkey,
            value_name.as_ptr(),
            0,
            REG_SZ,
            value_wide.as_ptr() as *const u8,
            (value_wide.len() * 2) as u32,
        );

        RegCloseKey(hkey);

        if result != ERROR_SUCCESS {
            return Err(format!("Failed to set value for '{}': {}", subkey, result));
        }
    }
    Ok(())
}

/// Check if a specific extension is currently associated with HinaView
pub fn is_associated(ext: &str) -> bool {
    let ext_with_dot = if ext.starts_with('.') {
        ext.to_string()
    } else {
        format!(".{}", ext)
    };
    let key_path = format!("Software\\Classes\\{}", ext_with_dot);
    let key_path_wide = to_wide_null(&key_path);

    unsafe {
        // SAFETY: The key path and value name buffers are null-terminated and remain valid during
        // the registry calls. `hkey` is closed before returning from this block.
        let mut hkey: HKEY = std::ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            key_path_wide.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        ) != ERROR_SUCCESS
        {
            return false;
        }

        let mut data_type = 0;
        let mut data_size = 1024;
        let mut data = vec![0u8; 1024];
        let result = RegQueryValueExW(
            hkey,
            to_wide_null("").as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            data.as_mut_ptr(),
            &mut data_size,
        );
        RegCloseKey(hkey);

        if result == ERROR_SUCCESS && data_type == REG_SZ {
            let prog_id = String::from_utf16_lossy(
                &data
                    .chunks_exact(2)
                    .take_while(|chunk| *chunk != [0, 0])
                    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect::<Vec<u16>>(),
            );

            return prog_id.starts_with(APP_PROG_ID_PREFIX)
                && prog_id.contains(ext.trim_start_matches('.'));
        }
    }
    false
}

/// Register a single extension with HinaView (HKCU - no admin required)
pub fn register_ext(ext: &str) -> Result<(), String> {
    let ext_no_dot = ext.trim_start_matches('.');
    let prog_id = format!("{}.{}.1", APP_PROG_ID_PREFIX, ext_no_dot);
    let exe_path_str = get_app_exe_path()
        .ok_or("Failed to get executable path")?
        .to_string_lossy()
        .to_string();
    let icon_path = get_icon_resource_for_ext(ext).unwrap_or_else(|| exe_path_str.clone());

    // 1. Map extension to ProgID
    set_reg_value(
        HKEY_CURRENT_USER,
        &format!("Software\\Classes\\.{}", ext_no_dot),
        &prog_id,
    )?;

    // 2. Set ProgID description
    let desc = format!("{} File", ext_no_dot.to_uppercase());
    set_reg_value(
        HKEY_CURRENT_USER,
        &format!("Software\\Classes\\{}", prog_id),
        &desc,
    )?;

    // 3. Set Icon
    set_reg_value(
        HKEY_CURRENT_USER,
        &format!("Software\\Classes\\{}\\DefaultIcon", prog_id),
        &icon_path,
    )?;

    // 4. Set Open Command
    let command = format!("\"{}\" \"%1\"", exe_path_str);
    set_reg_value(
        HKEY_CURRENT_USER,
        &format!("Software\\Classes\\{}\\shell\\open\\command", prog_id),
        &command,
    )?;

    Ok(())
}

/// Unregister a single extension
pub fn unregister_ext(ext: &str) -> Result<(), String> {
    let ext_with_dot = if ext.starts_with('.') {
        ext.to_string()
    } else {
        format!(".{}", ext)
    };
    let prog_id = format!("{}.{}.1", APP_PROG_ID_PREFIX, ext.trim_start_matches('.'));

    unsafe {
        // SAFETY: These calls operate on null-terminated key path buffers and ignore missing keys.
        let _ = RegDeleteTreeW(
            HKEY_CURRENT_USER,
            to_wide_null(&format!("Software\\Classes\\{}", prog_id)).as_ptr(),
        );
        let _ = RegDeleteKeyW(
            HKEY_CURRENT_USER,
            to_wide_null(&format!("Software\\Classes\\{}", ext_with_dot)).as_ptr(),
        );
    }
    Ok(())
}

/// Get the icon resource for a specific extension
/// Returns the icon path as "HinaView.exe,RESOURCE_ID"
fn get_icon_resource_for_ext(ext: &str) -> Option<String> {
    let exe_path_str = get_app_exe_path()?.to_string_lossy().to_string();

    let resource_id = match ext.trim_start_matches('.').to_lowercase().as_str() {
        "webp" => 101,
        "avif" => 102,
        "heif" => 103,
        "heic" => 104,
        "jxl" => 105,
        "jpg" | "jpeg" => 106,
        "png" => 107,
        "gif" => 108,
        "bmp" => 109,
        "tiff" | "tif" => 110,
        "tga" => 111,
        "dds" => 112,
        "exr" => 113,
        "hdr" => 114,
        "pnm" => 115,
        "ico" => 116,
        "cbz" => 201,
        _ => return None,
    };

    Some(format!("{},-{}", exe_path_str, resource_id))
}

/// Update file associations
pub fn update_associations(
    exts_to_associate: &[String],
    exts_to_disassociate: &[String],
) -> Result<(), String> {
    for ext in exts_to_disassociate {
        let _ = unregister_ext(ext);
    }
    for ext in exts_to_associate {
        register_ext(ext)?;
    }
    Ok(())
}

/// Unregister all file associations registered by HinaView
pub fn unregister_all() -> Result<(), String> {
    for ext in crate::util::formats::SUPPORTED_IMAGE_EXTENSIONS {
        let _ = unregister_ext(ext);
    }
    Ok(())
}

/// Helper function to set a named registry value
fn set_reg_value_named(root: HKEY, subkey: &str, name: &str, value: &str) -> Result<(), String> {
    let subkey_wide = to_wide_null(subkey);
    let mut hkey: HKEY = std::ptr::null_mut();
    let mut disposition = 0;

    unsafe {
        // SAFETY: All UTF-16 buffers are null-terminated and remain alive for the duration of
        // these registry API calls. `hkey` is closed before the block exits.
        let result = RegCreateKeyExW(
            root,
            subkey_wide.as_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut hkey,
            &mut disposition,
        );

        if result != ERROR_SUCCESS {
            return Err(format!(
                "Failed to create/open key '{}': {}",
                subkey, result
            ));
        }

        let name_wide = to_wide_null(name);
        let value_wide = to_wide_null(value);
        let result = RegSetValueExW(
            hkey,
            name_wide.as_ptr(),
            0,
            REG_SZ,
            value_wide.as_ptr() as *const u8,
            (value_wide.len() * 2) as u32,
        );

        RegCloseKey(hkey);

        if result != ERROR_SUCCESS {
            return Err(format!(
                "Failed to set value '{}' in '{}': {}",
                name, subkey, result
            ));
        }
    }
    Ok(())
}

/// Register context menu for a specific extension using a specific key path
fn register_context_menu_internal(
    key_path: &str,
    menu_text: &str,
    command_arg: &str,
) -> Result<(), String> {
    let exe_path_str = get_app_exe_path()
        .ok_or("Failed to get executable path")?
        .to_string_lossy()
        .to_string();

    let shell_key = format!("{}\\shell\\HinaView", key_path);

    // 0. Clean up existing key first to avoid conflicts
    unsafe {
        // SAFETY: The registry path buffer is null-terminated and valid for the duration of the call.
        let _ = RegDeleteTreeW(HKEY_CURRENT_USER, to_wide_null(&shell_key).as_ptr());
    }

    // 1. Set Menu Item Text (Default value)
    set_reg_value(HKEY_CURRENT_USER, &shell_key, menu_text)?;

    // 2. Set MUIVerb for display name (Windows uses this for context menu display)
    set_reg_value_named(HKEY_CURRENT_USER, &shell_key, "MUIVerb", menu_text)?;

    // 3. Set Icon (using exe file with icon resource ID 100 for generic folder icon)
    // Using the exe itself with icon index 0 (first icon in the exe)
    let icon_path = format!("{},0", exe_path_str);
    set_reg_value_named(HKEY_CURRENT_USER, &shell_key, "Icon", &icon_path)?;

    // 4. Set Command
    let command = format!("\"{}\" \"{}\"", exe_path_str, command_arg);
    set_reg_value_named(
        HKEY_CURRENT_USER,
        &format!("{}\\command", shell_key),
        "",
        &command,
    )?;

    Ok(())
}

/// Unregister context menu for a specific key path
fn unregister_context_menu_internal(key_path: &str) -> Result<(), String> {
    // Delete the command subkey first
    let command_key = format!("{}\\shell\\HinaView\\command", key_path);
    unsafe {
        // SAFETY: The registry path buffer is null-terminated and valid for the duration of the call.
        let _ = RegDeleteTreeW(HKEY_CURRENT_USER, to_wide_null(&command_key).as_ptr());
    }

    // Delete the HinaView shell key
    let shell_key = format!("{}\\shell\\HinaView", key_path);
    unsafe {
        // SAFETY: The registry path buffer is null-terminated and valid for the duration of the call.
        let _ = RegDeleteTreeW(HKEY_CURRENT_USER, to_wide_null(&shell_key).as_ptr());
    }

    Ok(())
}

/// Register context menu for a specific extension
pub fn register_context_menu(ext: &str, menu_text: &str) -> Result<(), String> {
    let ext_no_dot = ext.trim_start_matches('.');
    let prog_id = format!("{}.{}.1", APP_PROG_ID_PREFIX, ext_no_dot);

    // 1. Register for our ProgID (shows up when we are default)
    let prog_id_key = format!("Software\\Classes\\{}", prog_id);
    let _ = register_context_menu_internal(&prog_id_key, menu_text, "%1");

    // 2. Register via SystemFileAssociations (shows up even if we aren't default)
    let ext_with_dot = format!(".{}", ext_no_dot);
    let sfa_key = format!(
        "Software\\Classes\\SystemFileAssociations\\{}",
        ext_with_dot
    );
    register_context_menu_internal(&sfa_key, menu_text, "%1")
}

/// Unregister context menu for a specific extension
pub fn unregister_context_menu(ext: &str) -> Result<(), String> {
    let ext_no_dot = ext.trim_start_matches('.');
    let prog_id = format!("{}.{}.1", APP_PROG_ID_PREFIX, ext_no_dot);

    let prog_id_key = format!("Software\\Classes\\{}", prog_id);
    let _ = unregister_context_menu_internal(&prog_id_key);

    let ext_with_dot = format!(".{}", ext_no_dot);
    let sfa_key = format!(
        "Software\\Classes\\SystemFileAssociations\\{}",
        ext_with_dot
    );
    unregister_context_menu_internal(&sfa_key)
}

/// Register context menu for all supported extensions
pub fn register_all_context_menus(menu_text: &str) -> Result<(), String> {
    // Standard associated extensions
    for ext in crate::util::formats::SUPPORTED_IMAGE_EXTENSIONS {
        let _ = register_context_menu(ext, menu_text);
    }

    // Extra extensions (like .zip) that should have the menu but aren't default
    let _ = register_context_menu(".zip", menu_text);

    Ok(())
}

/// Unregister all context menus
pub fn unregister_all_context_menus() -> Result<(), String> {
    for ext in crate::util::formats::SUPPORTED_IMAGE_EXTENSIONS {
        let _ = unregister_context_menu(ext);
    }
    let _ = unregister_context_menu(".zip");
    Ok(())
}

/// Register context menu for directories (folders)
pub fn register_directory_context_menu(menu_text: &str) -> Result<(), String> {
    // Register for Directory (right-click on folder)
    let dir_key = "Software\\Classes\\Directory";
    register_context_menu_internal(&dir_key, menu_text, "%1")?;

    // Register for Directory background (right-click inside folder)
    let dir_bg_key = "Software\\Classes\\Directory\\background";
    // For background context menus, Windows does not pass %1, it passes %V
    register_context_menu_internal(&dir_bg_key, menu_text, "%V")?;

    Ok(())
}

/// Unregister context menu for directories
pub fn unregister_directory_context_menu() -> Result<(), String> {
    // Unregister for Directory
    let dir_key = "Software\\Classes\\Directory";
    let _ = unregister_context_menu_internal(&dir_key);

    // Unregister for Directory background
    let dir_bg_key = "Software\\Classes\\Directory\\background";
    let _ = unregister_context_menu_internal(&dir_bg_key);

    Ok(())
}
