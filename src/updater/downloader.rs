// src/updater/downloader.rs
// File downloader for updates

use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Download a file from URL to destination
pub fn download(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?;

    let mut file = File::create(dest)?;
    let bytes = response.bytes()?;
    file.write_all(&bytes)?;

    Ok(())
}
