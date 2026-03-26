// src/updater/extractor.rs
// ZIP extractor for updates

use std::fs::{self, File};
use std::io;
use std::path::Path;
use zip::ZipArchive;

/// Extract ZIP archive to destination directory
pub fn extract(zip_path: &Path, dest_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create destination directory
    fs::create_dir_all(dest_dir)?;

    // Open ZIP file
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = dest_dir.join(file.mangled_name());

        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&out_path)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}
