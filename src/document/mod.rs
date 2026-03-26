// src/document/mod.rs
pub mod archive;
pub mod archive_navigator;
pub mod format_probe;
pub mod logical_spread;
pub mod opening;
pub mod page_meta;
pub mod spread_builder;

pub use crate::pipeline::types::ArchiveReader;
pub use logical_spread::LogicalSpread;
pub use page_meta::PageMeta;

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

pub struct Document {
    pub id: u64,
    pub path: PathBuf,
    pub pages: Vec<PageMeta>,
    pub spreads: Vec<LogicalSpread>,
    pub reader: Arc<dyn ArchiveReader + Send + Sync>,
}

impl Document {
    /// Fast open: scans filenames, but defers metadata extraction to background workers.
    /// Only the initial page's metadata is extracted synchronously for immediate display.
    pub fn open_fast(path: PathBuf, initial_page_name: Option<String>) -> Result<(Self, usize)> {
        let reader = opening::create_reader(&path)?;
        let (pages, initial_index) =
            opening::build_pages_with_initial_metadata(&reader, initial_page_name.as_deref())?;
        info!(
            "[Document] 📂 Scanned {} files from {:?}",
            pages.len(),
            path
        );

        let doc_id = opening::generate_doc_id(&path);

        let document = opening::assemble_document(doc_id, path, pages, reader);
        Ok((document, initial_index))
    }

    /// High-level entry point used by the application layer.
    pub fn open_with_initial(path: PathBuf) -> Result<(Self, crate::types::PageId)> {
        let (open_path, initial_name) = opening::resolve_open_target(path);
        let (doc, initial) = Self::open_fast(open_path, initial_name)?;
        Ok((doc, initial as crate::types::PageId))
    }

    /// Re-calculates logical spreads based on latest page metadata.
    /// This is used when image dimensions are found/updated during runtime (e.g. after decoding).
    pub fn rebuild_spreads(&mut self, mode: crate::view::LayoutMode) {
        self.spreads = spread_builder::build_spreads(&self.pages, mode);
    }
}
