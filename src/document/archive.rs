// src/document/archive.rs
use crate::pipeline::types::ArchiveReader;
use crate::util::formats::{is_supported_image_name, is_supported_image_path};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use zip::ZipArchive;

/// Header size for fast metadata extraction (8KB)
pub const METADATA_HEADER_SIZE: usize = 8_192;
const ZIP_HANDLE_POOL_MAX: usize = 16;
const ZIP_HANDLE_POOL_EXPAND_ENTRY_THRESHOLD: usize = 256;

pub struct FolderReader {
    root: PathBuf,
}

impl FolderReader {
    pub fn new(path: PathBuf) -> Self {
        Self { root: path }
    }
}

impl ArchiveReader for FolderReader {
    fn list_images(&self) -> Vec<String> {
        let entries: Vec<_> = match fs::read_dir(&self.root) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
            Err(_) => return Vec::new(),
        };

        let mut images: Vec<String> = entries
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                if is_supported_image_path(&path) {
                    path.file_name().map(|n| n.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();

        images.sort_by(|a, b| crate::util::sorting::natural_cmp_ci(Path::new(a), Path::new(b)));
        images
    }

    fn read_file(&self, name: &str) -> Result<Vec<u8>> {
        let path = self.root.join(name);
        Ok(fs::read(path)?)
    }

    fn read_file_partial(&self, name: &str, limit: usize) -> Result<Vec<u8>> {
        let path = self.root.join(name);
        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; limit];
        let n = file.read(&mut buffer)?;
        buffer.truncate(n);
        Ok(buffer)
    }

    fn file_size_bytes(&self, name: &str) -> Option<u64> {
        let path = self.root.join(name);
        fs::metadata(path).ok().map(|m| m.len())
    }

    fn get_dimensions_fast(&self, name: &str) -> Option<(u32, u32)> {
        let header = self.read_file_partial(name, METADATA_HEADER_SIZE).ok()?;
        let meta = crate::document::format_probe::probe_image_metadata(&header)?;
        Some((meta.width, meta.height))
    }
}

pub struct ZipReader {
    archives: Vec<Mutex<ZipArchive<File>>>,
    next_archive: AtomicUsize,
    image_names: Vec<String>,
    entry_indices: HashMap<String, usize>,
    entry_sizes: Mutex<HashMap<String, u64>>,
}

impl ZipReader {
    pub fn new(path: PathBuf) -> Result<Self> {
        let t0 = Instant::now();
        let file = File::open(&path).with_context(|| format!("Failed to open ZIP: {:?}", path))?;
        let archive =
            ZipArchive::new(file).with_context(|| format!("Failed to parse ZIP: {:?}", path))?;

        let mut image_names = Vec::new();
        let mut entry_indices = HashMap::new();
        let entry_sizes = Mutex::new(HashMap::new());

        for (i, name) in archive.file_names().enumerate() {
            if is_supported_image_name(name) {
                let name = name.to_string();
                image_names.push(name.clone());
                entry_indices.entry(name.clone()).or_insert(i);
            }
        }

        image_names
            .sort_by(|a, b| crate::util::sorting::natural_cmp_ci(Path::new(a), Path::new(b)));

        let entry_count = archive.len();
        let desired_pool_size = if entry_count >= ZIP_HANDLE_POOL_EXPAND_ENTRY_THRESHOLD {
            num_cpus::get()
                .saturating_mul(2)
                .clamp(1, ZIP_HANDLE_POOL_MAX)
        } else {
            1
        };

        let mut archives = Vec::with_capacity(desired_pool_size);
        archives.push(Mutex::new(archive));

        for _ in 1..desired_pool_size {
            let file =
                File::open(&path).with_context(|| format!("Failed to open ZIP: {:?}", path))?;
            let extra = ZipArchive::new(file)
                .with_context(|| format!("Failed to parse ZIP for pool: {:?}", path))?;
            archives.push(Mutex::new(extra));
        }

        tracing::info!(
            "[ZipReader] Scanned {:?} in {}ms (images={})",
            path.file_name(),
            t0.elapsed().as_millis(),
            image_names.len()
        );

        Ok(Self {
            archives,
            next_archive: AtomicUsize::new(0),
            image_names,
            entry_indices,
            entry_sizes,
        })
    }

    fn select_archive_index(&self) -> usize {
        let len = self.archives.len().max(1);
        self.next_archive.fetch_add(1, Ordering::Relaxed) % len
    }

    fn log_archive_lock_wait(wait_ms: f32, archive_idx: usize, name: &str, purpose: &str) {
        if wait_ms >= 0.1 {
            tracing::debug!(
                "[ZipReader][LockWait] archive_mutex={:.2}ms handle={} file={} ({})",
                wait_ms,
                archive_idx,
                name,
                purpose
            );
        }
    }

    fn with_locked_archive<T, F>(&self, name: &str, purpose: &str, mut f: F) -> Result<T>
    where
        F: FnMut(&mut ZipArchive<File>) -> Result<T>,
    {
        let len = self.archives.len().max(1);
        let start = self.select_archive_index();

        for offset in 0..len {
            let idx = (start + offset) % len;
            let lock_t0 = Instant::now();
            match self.archives[idx].try_lock() {
                Ok(mut archive) => {
                    let wait_ms = lock_t0.elapsed().as_secs_f32() * 1000.0;
                    Self::log_archive_lock_wait(wait_ms, idx, name, purpose);
                    return f(&mut archive);
                }
                Err(std::sync::TryLockError::Poisoned(_)) => {
                    return Err(anyhow::anyhow!("ZIP archive mutex poisoned"));
                }
                Err(std::sync::TryLockError::WouldBlock) => {}
            }
        }

        let lock_t0 = Instant::now();
        let mut archive = self.archives[start]
            .lock()
            .map_err(|_| anyhow::anyhow!("ZIP archive mutex poisoned"))?;
        let wait_ms = lock_t0.elapsed().as_secs_f32() * 1000.0;
        Self::log_archive_lock_wait(wait_ms, start, name, purpose);
        f(&mut archive)
    }
}

impl ArchiveReader for ZipReader {
    fn list_images(&self) -> Vec<String> {
        self.image_names.clone()
    }

    fn read_file(&self, name: &str) -> Result<Vec<u8>> {
        let index = self
            .entry_indices
            .get(name)
            .copied()
            .with_context(|| format!("File not found in ZIP index: {}", name))?;
        self.with_locked_archive(name, "read_file", |archive| {
            let mut inner_file = archive.by_index(index)?;
            let mut buffer = Vec::new();
            inner_file.read_to_end(&mut buffer)?;
            Ok(buffer)
        })
    }

    fn read_file_partial(&self, name: &str, limit: usize) -> Result<Vec<u8>> {
        let index = self
            .entry_indices
            .get(name)
            .copied()
            .with_context(|| format!("File not found in ZIP index: {}", name))?;
        self.with_locked_archive(name, "read_partial", |archive| {
            let mut entry = archive.by_index(index)?;
            let mut header = vec![0u8; limit];
            let n = entry.read(&mut header)?;
            header.truncate(n);
            Ok(header)
        })
    }

    fn file_size_bytes(&self, name: &str) -> Option<u64> {
        let cache_lock_t0 = Instant::now();
        if let Ok(sizes) = self.entry_sizes.lock() {
            if let Some(size) = sizes.get(name).copied() {
                let lock_wait_ms = cache_lock_t0.elapsed().as_secs_f32() * 1000.0;
                if lock_wait_ms >= 0.1 {
                    tracing::debug!(
                        "[ZipReader][LockWait] size_cache_mutex={:.2}ms file={}",
                        lock_wait_ms,
                        name
                    );
                }
                return Some(size);
            }
        }

        let index = self.entry_indices.get(name).copied()?;
        let size = self
            .with_locked_archive(name, "file_size", |archive| {
                let size = archive.by_index(index)?.size();
                Ok(size)
            })
            .ok()?;
        if let Ok(mut sizes) = self.entry_sizes.lock() {
            sizes.insert(name.to_string(), size);
        }
        Some(size)
    }

    fn get_dimensions_fast(&self, name: &str) -> Option<(u32, u32)> {
        let header = self.read_file_partial(name, METADATA_HEADER_SIZE).ok()?;
        let meta = crate::document::format_probe::probe_image_metadata(&header)?;
        Some((meta.width, meta.height))
    }
}

pub struct SingleImageReader {
    path: PathBuf,
}

impl SingleImageReader {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ArchiveReader for SingleImageReader {
    fn list_images(&self) -> Vec<String> {
        vec![
            self.path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                .to_string_lossy()
                .to_string(),
        ]
    }

    fn read_file(&self, _name: &str) -> Result<Vec<u8>> {
        Ok(fs::read(&self.path)?)
    }

    fn read_file_partial(&self, _name: &str, limit: usize) -> Result<Vec<u8>> {
        let mut file = File::open(&self.path)?;
        let mut header = vec![0u8; limit];
        let n = file.read(&mut header)?;
        header.truncate(n);
        Ok(header)
    }

    fn file_size_bytes(&self, _name: &str) -> Option<u64> {
        fs::metadata(&self.path).ok().map(|m| m.len())
    }

    fn get_dimensions_fast(&self, _name: &str) -> Option<(u32, u32)> {
        let header = self.read_file_partial("", METADATA_HEADER_SIZE).ok()?;
        let meta = crate::document::format_probe::probe_image_metadata(&header)?;
        Some((meta.width, meta.height))
    }
}
