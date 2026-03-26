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

/// Represents an opened digital document, such as an image collection or a compressed archive.
///
/// The `Document` struct serves as the primary data model for active media, managing
/// a list of individual pages, their logical grouping into spreads for display,
/// and the underlying archive reader for data access.
pub struct Document {
    /// A unique identifier for the document, typically derived from its filesystem path.
    pub id: u64,
    /// The absolute path to the document file or directory.
    pub path: PathBuf,
    /// Metadata for each individual page (file) within the document.
    pub pages: Vec<PageMeta>,
    /// Logical layout groupings for display (e.g., single page or dual spread).
    pub spreads: Vec<LogicalSpread>,
    /// Thread-safe reader for extracting data from the document source.
    pub reader: Arc<dyn ArchiveReader + Send + Sync>,
}

impl Document {
    /// Performs an efficient document opening by deferring full metadata extraction.
    ///
    /// This method scans the directory or archive structure to identify files but
    /// only extracts metadata for the initial page synchronously. This ensures
    /// the UI remains responsive and can display the first page immediately.
    ///
    /// # Arguments
    /// * `path` - Path to the file or directory to open.
    /// * `initial_page_name` - Optional filename to start viewing from.
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

    /// Provides a high-level entry point for opening a document from a path.
    ///
    /// This function handles path resolution for internal archive navigation and
    /// returns both the initialized `Document` and the starting page index.
    pub fn open_with_initial(path: PathBuf) -> Result<(Self, crate::types::PageId)> {
        let (open_path, initial_name) = opening::resolve_open_target(path);
        let (doc, initial) = Self::open_fast(open_path, initial_name)?;
        Ok((doc, initial as crate::types::PageId))
    }

    /// Recalculates the logical spreads based on the latest available page metadata.
    ///
    /// This is used to update the document's layout when image dimensions are
    /// retrieved during background decoding or when the layout mode is toggled.
    pub fn rebuild_spreads(&mut self, mode: crate::view::LayoutMode) {
        self.spreads = spread_builder::build_spreads(&self.pages, mode);
    }

    /// Removes a page from the document and rebuilds spreads.
    ///
    /// # Arguments
    /// * `index` - Index of the page to remove.
    pub fn remove_page(&mut self, index: usize, mode: crate::view::LayoutMode) {
        if index < self.pages.len() {
            self.pages.remove(index);
            // Updating internal indices
            for (i, page) in self.pages.iter_mut().enumerate() {
                page.index = i;
            }
            self.rebuild_spreads(mode);
        }
    }
}
