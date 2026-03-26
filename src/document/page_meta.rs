// src/document/page_meta.rs

#[derive(Debug, Clone)]
pub struct PageMeta {
    pub index: usize,
    pub name: String,
    pub format_label: String,
    pub file_size_bytes: Option<u64>,
    pub width: u32,
    pub height: u32,
    pub metadata_probe_failed: bool,
    pub is_wide: bool,
    pub is_animated: bool,
    pub icc_profile: Option<String>,
    pub exif_camera: Option<String>,
    pub exif_lens: Option<String>,
    pub exif_f_stop: Option<String>,
    pub exif_shutter: Option<String>,
    pub exif_iso: Option<String>,
    pub exif_datetime: Option<String>,
}
