// src/document/opening.rs
use super::PageMeta;
use super::archive::{FolderReader, METADATA_HEADER_SIZE, SingleImageReader, ZipReader};
use super::format_probe::{FileFormat, probe_format, probe_image_metadata};
use crate::document::LogicalSpread;
use crate::document::{ArchiveReader, Document};
use crate::util::formats::is_supported_image_path;
use anyhow::Result;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const INITIAL_METADATA_DEEP_HEADER_SIZE: usize = 131_072;

pub fn generate_doc_id(path: &Path) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    let normalized = crate::util::formats::normalize_path(path);
    normalized.hash(&mut hasher);
    hasher.finish()
}

pub fn create_reader(path: &Path) -> Result<Arc<dyn ArchiveReader + Send + Sync>> {
    let format = probe_format(path);
    match format {
        FileFormat::Folder => Ok(Arc::new(FolderReader::new(path.to_path_buf()))),
        FileFormat::Zip => Ok(Arc::new(ZipReader::new(path.to_path_buf())?)),
        FileFormat::Image => Ok(Arc::new(SingleImageReader::new(path.to_path_buf()))),
        FileFormat::Unknown => Err(anyhow::anyhow!("Unsupported file format.")),
    }
}

pub fn resolve_open_target(path: PathBuf) -> (PathBuf, Option<String>) {
    let path_ref: &Path = path.as_path();

    if path_ref.is_dir() {
        return (path, None);
    }

    if let Some(ext) = path_ref
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
    {
        if ext == "zip" || ext == "cbz" {
            return (path, None);
        }

        if is_supported_image_path(path_ref)
            && let Some(parent) = path_ref.parent()
        {
            let file_name = path_ref
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            return (parent.to_path_buf(), Some(file_name));
        }
    }

    (path, None)
}

pub fn build_pages_with_initial_metadata(
    reader: &Arc<dyn ArchiveReader + Send + Sync>,
    initial_page_name: Option<&str>,
) -> Result<(Vec<PageMeta>, usize)> {
    let image_names = reader.list_images();
    if image_names.is_empty() {
        return Err(anyhow::anyhow!("[Document] No images found"));
    }

    let initial_index = if let Some(name) = initial_page_name {
        image_names.iter().position(|p| p == name).unwrap_or(0)
    } else {
        0
    };

    let mut pages: Vec<PageMeta> = image_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let format_label = std::path::Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_uppercase())
                .unwrap_or_else(|| "Unknown".to_string());

            PageMeta {
                index,
                name: name.clone(),
                format_label,
                file_size_bytes: None,
                width: 0,
                height: 0,
                metadata_probe_failed: false,
                is_wide: false,
                is_animated: false,
                icc_profile: None,
                exif_camera: None,
                exif_lens: None,
                exif_f_stop: None,
                exif_shutter: None,
                exif_iso: None,
                exif_datetime: None,
            }
        })
        .collect();

    if let Some(page) = pages.get_mut(initial_index)
        && let Ok(header) = reader.read_file_partial(&page.name, METADATA_HEADER_SIZE)
    {
        let mut initial_meta = probe_image_metadata(&header);
        if initial_meta.is_none()
            && let Ok(deep_header) =
                reader.read_file_partial(&page.name, INITIAL_METADATA_DEEP_HEADER_SIZE)
        {
            initial_meta = probe_image_metadata(&deep_header);
        }

        if let Some(meta) = initial_meta {
            page.width = meta.width;
            page.height = meta.height;
            page.is_wide = page.width > page.height;
            page.is_animated = meta.is_animated;
        }
    }

    Ok((pages, initial_index))
}

pub fn build_initial_spreads(pages: &[PageMeta]) -> Vec<LogicalSpread> {
    super::spread_builder::build_spreads(pages, crate::view::LayoutMode::Single)
}

pub fn assemble_document(
    doc_id: u64,
    path: PathBuf,
    pages: Vec<PageMeta>,
    reader: Arc<dyn ArchiveReader + Send + Sync>,
) -> Document {
    let spreads = build_initial_spreads(&pages);
    Document {
        id: doc_id,
        path,
        pages,
        spreads,
        reader,
    }
}
