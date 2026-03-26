// src/util/formats.rs
use std::path::Path;

/// Supported image file extensions for file association.
/// Each extension includes the leading dot (e.g., ".webp").
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    ".webp", ".avif", ".heif", ".heic", ".jxl", ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".tiff",
    ".tif", ".tga", ".dds", ".exr", ".hdr", ".pnm", ".ico", ".cbz",
];
/// Returns true if the file extension is a supported archive format (ZIP, CBZ).
pub fn is_supported_archive_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    let ext_lower = ext.to_lowercase();
    matches!(ext_lower.as_str(), "zip" | "cbz")
}

/// Returns true if the path points to a supported image file.
/// Also filters out system hidden files and metadata folders like __MACOSX.
pub fn is_supported_image_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };

    // Filter out hidden files (starting with dot) and system folders
    if file_name.starts_with('.') {
        return false;
    }

    // Filter out __MACOSX or similar system-internal paths
    let path_str = path.to_string_lossy();
    if path_str.contains("__MACOSX") {
        return false;
    }

    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    let ext_lower = ext.to_lowercase();
    matches!(
        ext_lower.as_str(),
        "jpg"
            | "jpeg"
            | "png"
            | "webp"
            | "gif"
            | "avif"
            | "heic"
            | "heif"
            | "jxl"
            | "bmp"
            | "tiff"
            | "tif"
            | "tga"
            | "dds"
            | "exr"
            | "hdr"
            | "ico"
            | "pbm"
            | "pgm"
            | "ppm"
            | "pnm"
    )
}

/// String-based fast path for archive entry names (ZIP internal paths).
pub fn is_supported_image_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.contains("__MACOSX") {
        return false;
    }
    if name.ends_with('/') || name.ends_with('\\') {
        return false;
    }

    let file_name = name.rsplit(['/', '\\']).next().unwrap_or(name);
    if file_name.starts_with('.') {
        return false;
    }

    let Some((_, ext)) = file_name.rsplit_once('.') else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "jpg"
            | "jpeg"
            | "png"
            | "webp"
            | "gif"
            | "avif"
            | "heic"
            | "heif"
            | "jxl"
            | "bmp"
            | "tiff"
            | "tif"
            | "tga"
            | "dds"
            | "exr"
            | "hdr"
            | "ico"
            | "pbm"
            | "pgm"
            | "ppm"
            | "pnm"
    )
}

/// Formats byte size into human readable string (KB, MB, GB).
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f32 as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f32 as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f32 as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Robustly normalizes a path into a standard string for consistent hashing and comparison.
/// Handles Windows verbatim prefixes (\\?\), slash directions, and case-insensitivity.
pub fn normalize_path(path: &Path) -> String {
    let s = path.to_string_lossy();

    #[cfg(windows)]
    {
        // 1. Remove the verbatim prefix (\\?\)
        let s = s.trim_start_matches(r"\\?\");
        // 2. Uniform slashes to forward slashes
        let s = s.replace('\\', "/");
        // 3. Lowercase for case-insensitivity
        s.to_lowercase()
    }

    #[cfg(not(windows))]
    {
        s.to_string()
    }
}
