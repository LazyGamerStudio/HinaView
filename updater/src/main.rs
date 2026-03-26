// updater/src/main.rs
// HinaView Updater - applies files extracted by the main application.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::thread;
use std::time::{Duration, Instant};
use tracing::error;

#[derive(Deserialize)]
struct UpdateFlag {
    extracted_path: String,
}

fn main() {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    // Wait for HinaView.exe to fully exit
    let exe_name = "HinaView.exe";
    wait_for_process_exit(exe_name);

    // Get paths
    let current_exe = std::env::current_exe()
        .ok()
        .unwrap_or_else(|| PathBuf::from("updater.exe"));
    let parent_dir = current_exe
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let hina_view_exe = parent_dir.join("HinaView.exe");
    let hina_view_bak = parent_dir.join("HinaView.exe.bak");
    let update_temp = parent_dir.join("update_temp");
    let extracted = read_flag(&update_temp)
        .and_then(|flag| resolve_extracted_root(Path::new(&flag.extracted_path)))
        .or_else(|| resolve_extracted_root(&update_temp.join("extracted")))
        .unwrap_or_else(|| update_temp.join("extracted"));

    // Backup old executable
    if hina_view_exe.exists() {
        let _ = std::fs::remove_file(&hina_view_bak);
        if let Err(e) = std::fs::rename(&hina_view_exe, &hina_view_bak) {
            log_error(&format!("Failed to backup old executable: {}", e));
            exit(1);
        }
    }

    if let Err(e) = copy_update_payload(&extracted, &parent_dir) {
        log_error(&format!("Failed to copy update payload: {}", e));
        let _ = std::fs::remove_file(&hina_view_exe);
        let _ = std::fs::rename(&hina_view_bak, &hina_view_exe);
        exit(1);
    }

    // Clean up
    let _ = std::fs::remove_dir_all(&update_temp);
    let _ = std::fs::remove_file(&hina_view_bak);

    // Launch HinaView.exe
    if let Err(e) = Command::new(&hina_view_exe).spawn() {
        log_error(&format!("Failed to launch HinaView: {}", e));
        exit(1);
    }

    // Exit updater
    exit(0);
}

fn wait_for_process_exit(exe_name: &str) {
    let timeout = Duration::from_secs(30);
    let start = Instant::now();

    while start.elapsed() < timeout {
        if !is_process_running(exe_name) {
            return;
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn is_process_running(exe_name: &str) -> bool {
    // Simple check: try to open the file
    // If it fails, the process is likely still running
    let current_exe = std::env::current_exe()
        .ok()
        .unwrap_or_else(|| PathBuf::from("updater.exe"));
    let parent_dir = current_exe
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let exe_path = parent_dir.join(exe_name);

    // Try to open the file for writing - if it fails, process is still running
    std::fs::OpenOptions::new()
        .write(true)
        .open(&exe_path)
        .is_err()
}

fn log_error(msg: &str) {
    error!("[Updater] {}", msg);
}

fn read_flag(update_temp: &Path) -> Option<UpdateFlag> {
    let flag_path = update_temp.join("update.flag");
    let content = std::fs::read_to_string(flag_path).ok()?;
    serde_json::from_str(&content).ok()
}

fn resolve_extracted_root(extracted: &Path) -> Option<PathBuf> {
    if !extracted.exists() {
        return None;
    }

    let direct_exe = extracted.join("HinaView.exe");
    if direct_exe.exists() {
        return Some(extracted.to_path_buf());
    }

    let entries: Vec<PathBuf> = std::fs::read_dir(extracted)
        .ok()?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();

    if entries.len() == 1 && entries[0].is_dir() && entries[0].join("HinaView.exe").exists() {
        return Some(entries[0].clone());
    }

    Some(extracted.to_path_buf())
}

fn copy_update_payload(src_root: &Path, dst_root: &Path) -> Result<(), String> {
    if !src_root.exists() {
        return Err(format!(
            "Update source does not exist: {}",
            src_root.display()
        ));
    }

    if !src_root.join("HinaView.exe").exists() {
        return Err(format!(
            "Updated HinaView.exe not found in {}",
            src_root.display()
        ));
    }

    copy_dir_recursive(src_root, dst_root)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|e| e.to_string())?;
            copy_dir_recursive(&src_path, &dst_path)?;
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if file_name == "updater.exe" {
            continue;
        }

        std::fs::copy(&src_path, &dst_path).map_err(|e| {
            format!(
                "copy failed ({} -> {}): {}",
                src_path.display(),
                dst_path.display(),
                e
            )
        })?;
    }

    Ok(())
}
