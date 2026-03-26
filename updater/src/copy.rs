// updater/src/copy.rs
use std::path::Path;
use crate::hash::calculate_sha256;

pub fn copy_update_payload(src_root: &Path, dst_root: &Path, current_exe: &Path) -> Result<(), String> {
    if !src_root.exists() {
        return Err(format!("Update source does not exist: {}", src_root.display()));
    }

    if !src_root.join("HinaView.exe").exists() {
        return Err(format!("Updated HinaView.exe not found in {}", src_root.display()));
    }

    copy_dir_recursive(src_root, dst_root, current_exe)
}

pub fn copy_dir_recursive(src: &Path, dst: &Path, current_exe: &Path) -> Result<(), String> {
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let os_name = entry.file_name();
        let dst_path = dst.join(&os_name);
        let file_name = os_name.to_string_lossy();
        let file_name_lower = file_name.to_lowercase();

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|e| e.to_string())?;
            copy_dir_recursive(&src_path, &dst_path, current_exe)?;
            continue;
        }

        // Compare hashes to avoid unnecessary copying or self-update renaming
        if dst_path.exists() {
            if let (Some(src_hash), Some(dst_hash)) = (calculate_sha256(&src_path), calculate_sha256(&dst_path)) {
                if src_hash == dst_hash {
                    println!("[Updater] Skipping identical file: {}", file_name);
                    continue;
                }
            }
        }

        // Self-update bypass for updater.exe
        if file_name_lower == "updater.exe" {
            println!("[Updater] Detected new updater.exe. Renaming current updater for self-update...");
            let old_updater = current_exe.with_extension("exe.old");
            let _ = std::fs::remove_file(&old_updater);
            if let Err(e) = std::fs::rename(current_exe, &old_updater) {
                println!("[Updater] Warning: Failed to rename current updater.exe: {}. Skipping updater update.", e);
                continue;
            }
        }

        println!("[Updater] Copying: {}", file_name);
        std::fs::copy(&src_path, &dst_path).map_err(|e| {
            format!("copy failed ({} -> {}): {}", src_path.display(), dst_path.display(), e)
        })?;
    }

    Ok(())
}
