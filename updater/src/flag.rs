// updater/src/flag.rs
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct UpdateFlag {
    pub extracted_path: String,
}

pub fn read_flag(update_temp: &Path) -> Option<UpdateFlag> {
    let flag_path = update_temp.join("update.flag");
    let content = std::fs::read_to_string(flag_path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn resolve_extracted_root(extracted: &Path) -> Option<PathBuf> {
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
