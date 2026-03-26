use super::ImageDecoder;
use crate::types::{DecodedImage, MipLevel};
use anyhow::{Result, anyhow};
use crabby_avif::decoder::Decoder;
use crabby_avif::reformat::rgb;

pub struct AvifDecoder;

impl AvifDecoder {
    pub fn new() -> Self {
        Self
    }
}

impl ImageDecoder for AvifDecoder {
    fn decode(&self, data: &[u8], _mip: MipLevel) -> Result<DecodedImage> {
        let mut decoder = Decoder::default();

        // Optimize for speed as AVIF is heavy and serialized.
        let total_cores = num_cpus::get();
        let max_threads = if total_cores <= 4 {
            total_cores.saturating_sub(1).max(1)
        } else {
            total_cores.saturating_sub(2).max(1)
        };
        decoder.settings.max_threads = max_threads as u32;

        // SAFETY: `data` is a valid byte slice that remains alive for the duration of the call,
        // and crabby-avif only borrows the provided memory while parsing/decoding.
        unsafe {
            decoder
                .set_io_raw(data.as_ptr(), data.len())
                .map_err(|e| anyhow!("Failed to set AVIF IO: {:?}", e))?;
        }

        decoder
            .parse()
            .map_err(|e| anyhow!("Failed to parse AVIF: {:?}", e))?;
        decoder
            .next_image()
            .map_err(|e| anyhow!("Failed to decode AVIF image: {:?}", e))?;

        let image = decoder
            .image()
            .ok_or_else(|| anyhow!("No image found in AVIF"))?;

        let width = image.width;
        let height = image.height;

        // Convert YUV to RGBA8
        let mut rgb = rgb::Image::create_from_yuv(image);
        rgb.format = rgb::Format::Rgba;
        rgb.depth = 8;
        rgb.allocate()
            .map_err(|e| anyhow!("Failed to allocate RGB buffer: {:?}", e))?;

        rgb.convert_from_yuv(image)
            .map_err(|e| anyhow!("Failed to convert AVIF to RGBA: {:?}", e))?;

        // Get pixels from the allocated buffer
        let pixels = match rgb.pixels {
            Some(crabby_avif::utils::pixels::Pixels::Buffer(data)) => data,
            _ => return Err(anyhow!("Failed to get RGB pixels from buffer")),
        };

        // Extract ICC and EXIF metadata (Lazy Extraction)
        let icc_profile = if !image.icc.is_empty() {
            // Check for embedded ICC profile name or use a default label
            crate::document::format_probe::probe_icc_profile_name(&image.icc)
                .or_else(|| Some("Embedded ICC".to_string()))
        } else {
            None
        };

        let exif = if !image.exif.is_empty() {
            // AVIF EXIF data usually starts with the TIFF header directly
            crate::document::format_probe::extract_exif_summary(&image.exif)
        } else {
            None
        };

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
