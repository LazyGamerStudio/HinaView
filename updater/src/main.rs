// updater/src/main.rs
// HinaView Updater - applies files extracted by the main application.

mod flag;
mod hash;
mod copy;
mod report;
mod process;

use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::thread;
use std::time::Duration;

fn main() {
    println!("[Updater] HinaView Updater starting...");

    // Capture arguments to pass back to HinaView
    let passthrough_args: Vec<String> = std::env::args().skip(1).collect();

    // 1. Wait for HinaView.exe to exit
    let exe_name = "HinaView.exe";
    process::wait_for_process_exit(exe_name);

    // 2. Resolve paths
    let current_exe = std::env::current_exe()
        .ok()
        .unwrap_or_else(|| PathBuf::from("updater.exe"));
    let parent_dir = current_exe
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    
    let hina_view_exe = parent_dir.join("HinaView.exe");
    let hina_view_bak = parent_dir.join("HinaView.exe.bak");
    let update_temp = parent_dir.join("update_temp");
    
    // 3. Find extracted update payload
    let extracted = flag::read_flag(&update_temp)
        .and_then(|f| flag::resolve_extracted_root(Path::new(&f.extracted_path)))
        .or_else(|| flag::resolve_extracted_root(&update_temp.join("extracted")))
        .unwrap_or_else(|| update_temp.join("extracted"));

    // 4. Backup current main executable
    if hina_view_exe.exists() {
        println!("[Updater] Backing up current executable...");
        let _ = std::fs::remove_file(&hina_view_bak);
        if let Err(e) = std::fs::rename(&hina_view_exe, &hina_view_bak) {
            println!("[Updater] Failed to backup old executable: {}", e);
            exit(1);
        }
    }

    // 5. Apply update (Recursive copy with hash verification)
    println!("[Updater] Applying new files from {}...", extracted.display());
    if let Err(e) = copy::copy_update_payload(&extracted, &parent_dir, &current_exe) {
        println!("[Updater] Failed to copy update payload: {}", e);
        // Rollback
        if hina_view_bak.exists() {
            println!("[Updater] Rolling back to backup...");
            let _ = std::fs::remove_file(&hina_view_exe);
            let _ = std::fs::rename(&hina_view_bak, &hina_view_exe);
        }
        exit(1);
    }
    println!("[Updater] Files applied successfully.");

    // 6. UX: Handle release notes
    report::open_release_notes(&extracted, &parent_dir);

    // 7. Cleanup
    println!("[Updater] Cleaning up temporary files...");
    
    let flag_file = update_temp.join("update.flag");
    if flag_file.exists() {
        let _ = std::fs::remove_file(&flag_file);
    }
    let _ = std::fs::remove_file(&hina_view_bak);

    let mut retries = 0;
    while update_temp.exists() && retries < 3 {
        match std::fs::remove_dir_all(&update_temp) {
            Ok(_) => break,
            Err(_) => {
                println!("[Updater] Failed to delete update_temp (attempt {}/3). File lock detected. Waiting...", retries + 1);
                thread::sleep(Duration::from_secs(10));
                retries += 1;
            }
        }
    }

    // 8. Launch updated HinaView
    println!("[Updater] Launching HinaView...");
    if let Err(e) = Command::new(&hina_view_exe).args(&passthrough_args).spawn() {
        println!("[Updater] Failed to launch HinaView: {}", e);
        thread::sleep(Duration::from_secs(5));
        exit(1);
    }

    exit(0);
}
