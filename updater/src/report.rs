// updater/src/report.rs
use std::path::Path;
use std::process::Command;

pub fn open_release_notes(extracted: &Path, dst_root: &Path) {
    if let Ok(entries) = std::fs::read_dir(extracted) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let os_name = entry.file_name();
                let name = os_name.to_string_lossy();
                let name_lower = name.to_lowercase();
                if name_lower.starts_with("releasenote") && name_lower.ends_with(".txt") {
                    let dest_path = dst_root.join(&*name);
                    println!("[Updater] Found release notes: {}. Copying to root and opening...", name);
                    
                    // Copy to root directory so it persists after cleanup
                    if let Err(e) = std::fs::copy(&path, &dest_path) {
                        println!("[Updater] Failed to copy release notes: {}", e);
                        continue;
                    }

                    // Use cmd /C start to open file with default handler on Windows
                    let _ = Command::new("cmd")
                        .arg("/C")
                        .arg("start")
                        .arg("")
                        .arg(&dest_path)
                        .spawn();
                }
            }
        }
    }
}
