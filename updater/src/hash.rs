// updater/src/hash.rs
use sha2::{Sha256, Digest};
use std::io::Read;
use std::path::Path;

pub fn calculate_sha256(path: &Path) -> Option<Vec<u8>> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Some(hasher.finalize().to_vec())
}
