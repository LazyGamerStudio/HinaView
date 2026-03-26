use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookmarkSource {
    AutoRecent,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkEntry {
    pub id: u64,
    pub source: BookmarkSource,
    pub archive_name: String,
    pub file_name: String,
    pub path: PathBuf,
    pub page_index: usize,
    pub page_name: String,
    pub saved_at_ms: u64,
}
