// src/ui/file_association_icons.rs
// File association icons embedded in the binary

use egui::{ColorImage, ImageData, TextureHandle};
use image::GenericImageView;

/// Get an embedded icon for a file extension
/// Returns the icon data as raw ICO bytes
pub fn get_icon_for_extension(ext: &str) -> Option<&'static [u8]> {
    match ext.trim_start_matches('.').to_lowercase().as_str() {
        "webp" => Some(include_bytes!("../../icon_win/webp.ico")),
        "avif" => Some(include_bytes!("../../icon_win/avif.ico")),
        "heif" => Some(include_bytes!("../../icon_win/heif.ico")),
        "heic" => Some(include_bytes!("../../icon_win/heic.ico")),
        "jxl" => Some(include_bytes!("../../icon_win/jxl.ico")),
        "jpg" | "jpeg" => Some(include_bytes!("../../icon_win/jpeg.ico")),
        "png" => Some(include_bytes!("../../icon_win/png.ico")),
        "gif" => Some(include_bytes!("../../icon_win/gif.ico")),
        "bmp" => Some(include_bytes!("../../icon_win/bmp.ico")),
        "tiff" | "tif" => Some(include_bytes!("../../icon_win/tiff.ico")),
        "tga" => Some(include_bytes!("../../icon_win/tga.ico")),
        "dds" => Some(include_bytes!("../../icon_win/dds.ico")),
        "exr" => Some(include_bytes!("../../icon_win/exr.ico")),
        "hdr" => Some(include_bytes!("../../icon_win/hdr.ico")),
        "pnm" => Some(include_bytes!("../../icon_win/pnm.ico")),
        "ico" => Some(include_bytes!("../../icon_win/ico.ico")),
        "cbz" => Some(include_bytes!("../../icon_win/cbz.ico")),
        _ => None,
    }
}

/// Load an ICO file and extract the 24x24 icon as RGBA
/// Uses the image crate for reliable ICO decoding (handles PNG and BMP formats)
pub fn load_icon_from_ico(ico_data: &[u8]) -> Option<Vec<u8>> {
    tracing::info!("ICO data size: {} bytes", ico_data.len());

    // Use image crate to decode ICO - it handles both PNG-compressed and BMP formats
    match image::load_from_memory_with_format(ico_data, image::ImageFormat::Ico) {
        Ok(img) => {
            let (orig_w, orig_h) = img.dimensions();
            tracing::info!("ICO decoded: {}x{}", orig_w, orig_h);

            // Resize to 16x16 for pixel-perfect rendering
            let resized = img.resize_exact(16, 16, image::imageops::FilterType::Lanczos3);
            let rgba = resized.into_rgba8();
            tracing::info!("ICO resized to 16x16, RGBA size: {} bytes", rgba.len());
            Some(rgba.to_vec())
        }
        Err(e) => {
            tracing::error!("ICO decode failed: {}", e);
            None
        }
    }
}

/// Load and cache all icons as texture handles
#[allow(dead_code)]
pub struct IconCache {
    textures: std::collections::HashMap<String, TextureHandle>,
}

#[allow(dead_code)]
impl IconCache {
    pub fn new() -> Self {
        Self {
            textures: std::collections::HashMap::new(),
        }
    }

    pub fn get_or_load(&mut self, ctx: &egui::Context, ext: &str) -> Option<&TextureHandle> {
        use std::collections::hash_map::Entry;

        match self.textures.entry(ext.to_lowercase()) {
            Entry::Occupied(entry) => Some(entry.into_mut()),
            Entry::Vacant(entry) => {
                if let Some(ico_data) = get_icon_for_extension(ext) {
                    if let Some(rgba) = load_icon_from_ico(ico_data) {
                        let image = ColorImage::from_rgba_unmultiplied([16, 16], &rgba);
                        let texture = ctx.load_texture(
                            &format!("icon_{}", ext),
                            ImageData::Color(image.into()),
                            Default::default(),
                        );
                        return Some(entry.insert(texture));
                    }
                }
                None
            }
        }
    }
}
