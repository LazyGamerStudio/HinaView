use super::ImageDecoder;
use crate::types::DecodedImage;
use anyhow::Result;
use image::GenericImageView;

pub struct FallbackDecoder;

impl FallbackDecoder {
    pub fn new() -> Self {
        Self
    }
}

impl ImageDecoder for FallbackDecoder {
    fn decode(&self, data: &[u8], _mip: crate::types::MipLevel) -> Result<DecodedImage> {
        let img = image::load_from_memory(data)?;
        let (width, height) = img.dimensions();

        // Convert to RGBA8 format
        let rgba = img.to_rgba8();
        let pixels = rgba.into_raw();

        let exif = crate::document::format_probe::extract_exif_summary(data);
        let icc_profile = crate::document::format_probe::probe_icc_profile_name(data);

        Ok(DecodedImage {
            width,
            height,
            original_width: width,
            original_height: height,
            pixels,
            icc_profile,
            exif,
        })
    }
}
