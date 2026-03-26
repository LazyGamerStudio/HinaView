// updater/src/main.rs
// HinaView Updater - applies files extracted by the main application.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Deserialize)]
struct UpdateFlag {
    extracted_path: String,
}

fn main() {
    println!("[Updater] HinaView Updater starting...");

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
    
    // Find extracted root
    let extracted = read_flag(&update_temp)
        .and_then(|flag| resolve_extracted_root(Path::new(&flag.extracted_path)))
        .or_else(|| resolve_extracted_root(&update_temp.join("extracted")))
        .unwrap_or_else(|| update_temp.join("extracted"));

    // Backup old executable
    if hina_view_exe.exists() {
        println!("[Updater] Backing up current executable...");
        let _ = std::fs::remove_file(&hina_view_bak);
        if let Err(e) = std::fs::rename(&hina_view_exe, &hina_view_bak) {
            println!("[Updater] Failed to backup old executable: {}", e);
            exit(1);
        }
    }

    println!("[Updater] Applying new files from {}...", extracted.display());
    if let Err(e) = copy_update_payload(&extracted, &parent_dir, &current_exe) {
        println!("[Updater] Failed to copy update payload: {}", e);
        // Rollback
        let _ = std::fs::remove_file(&hina_view_exe);
        let _ = std::fs::rename(&hina_view_bak, &hina_view_exe);
        exit(1);
    }
    println!("[Updater] Files copied successfully.");

    // Clean up
    println!("[Updater] Cleaning up temporary files...");
    
    // DELIBERATE BUGFIX: Remove flag file first to prevent update loops
    let flag_file = update_temp.join("update.flag");
    if flag_file.exists() {
        let _ = std::fs::remove_file(&flag_file);
    }

    let _ = std::fs::remove_file(&hina_view_bak);

    let mut retries = 0;
    while update_temp.exists() && retries < 3 {
        match std::fs::remove_dir_all(&update_temp) {
            Ok(_) => {
                println!("[Updater] Successfully removed update_temp directory.");
                break;
            }
            Err(_) => {
                println!(
                    "[Updater] Failed to delete update_temp (attempt {}/3). File lock detected. Waiting 10 seconds...",
                    retries + 1
                );
                thread::sleep(Duration::from_secs(10));
                retries += 1;
            }
        }
    }

    if update_temp.exists() {
        println!("[Updater] Could not delete temp folder automatically. Please delete the 'update_temp' directory manually.");
        println!("[Updater] 임시 폴더 자동 삭제에 실패했습니다. 수동으로 'update_temp' 폴더를 삭제해 주세요.");
        println!("[Updater] 一時フォルダの自動削除に失敗しました。手動で 'update_temp' フォルダを削除してください。");
        println!("[Updater] 无法自动删除临时文件夹。请手动删除 'update_temp' 目录。");
        println!("[Updater] 無法自動刪除臨時資料夾。請手動刪除 'update_temp' 目錄。");
        thread::sleep(Duration::from_secs(5));
    }

    println!("[Updater] Update complete! Launching HinaView...");

    // Launch HinaView.exe
    if let Err(e) = Command::new(&hina_view_exe).spawn() {
        println!("[Updater] Failed to launch HinaView: {}", e);
        thread::sleep(Duration::from_secs(5));
        exit(1);
    }

    // Exit updater
    exit(0);
}

fn wait_for_process_exit(exe_name: &str) {
    let timeout = Duration::from_secs(60);
    let start = Instant::now();
    let mut warned = false;

    while start.elapsed() < timeout {
        if !is_process_running(exe_name) {
            return;
        }

        if !warned && start.elapsed() > Duration::from_secs(2) {
            println!("[Updater] HinaView is still running. Please close HinaView.exe to continue the update...");
            println!("[Updater] 프로그램이 아직 실행 중입니다. 업데이트를 계속하려면 HinaView.exe를 종료해 주세요...");
            println!("[Updater] プログラムがまだ実行中です。アップデートを続行するにはHinaView.exeを終了してください...");
            println!("[Updater] 程序仍在运行。请关闭 HinaView.exe 以继续更新...");
            println!("[Updater] 程式仍在執行中。請關閉 HinaView.exe 以繼續更新...");
            warned = true;
        }

        thread::sleep(Duration::from_millis(500));
    }
}

fn is_process_running(exe_name: &str) -> bool {
    let current_exe = std::env::current_exe()
        .ok()
        .unwrap_or_else(|| PathBuf::from("updater.exe"));
    let parent_dir = current_exe
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let exe_path = parent_dir.join(exe_name);

    if !exe_path.exists() {
        return false;
    }

    // Try to open the file for writing - if it fails, process is still running
    std::fs::OpenOptions::new()
        .write(true)
        .open(&exe_path)
        .is_err()
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

fn copy_update_payload(src_root: &Path, dst_root: &Path, current_exe: &Path) -> Result<(), String> {
    if !src_root.exists() {
        return Err(format!("Update source does not exist: {}", src_root.display()));
    }

    if !src_root.join("HinaView.exe").exists() {
        return Err(format!("Updated HinaView.exe not found in {}", src_root.display()));
    }

    copy_dir_recursive(src_root, dst_root, current_exe)
}

fn copy_dir_recursive(src: &Path, dst: &Path, current_exe: &Path) -> Result<(), String> {
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_name = entry.file_name().to_string_lossy().to_ascii_lowercase();

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|e| e.to_string())?;
            copy_dir_recursive(&src_path, &dst_path, current_exe)?;
            continue;
        }

        // Self-update bypass for updater.exe
        if file_name == "updater.exe" {
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
