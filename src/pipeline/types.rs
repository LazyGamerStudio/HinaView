// src/pipeline/types.rs
use anyhow::Result;

/// Trait for archive readers (ZIP, folder, single image).
/// Moved here to avoid pipeline → document dependency.
pub trait ArchiveReader {
    fn list_images(&self) -> Vec<String>;
    fn read_file(&self, name: &str) -> Result<Vec<u8>>;
    fn read_file_partial(&self, name: &str, limit: usize) -> Result<Vec<u8>>;
    fn file_size_bytes(&self, name: &str) -> Option<u64>;

    /// Fast dimension extraction from header only (no full decode).
    /// Returns None if format is not recognized or header is corrupted.
    #[allow(dead_code)]
    fn get_dimensions_fast(&self, name: &str) -> Option<(u32, u32)>;
}
