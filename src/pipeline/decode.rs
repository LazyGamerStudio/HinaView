use super::decoders::{
    ImageDecoder, avif::AvifDecoder, fallback::FallbackDecoder, gif::GifDecoder, heif::HeifDecoder,
    jxl::JxlDecoder, turbo_jpeg::TurboJpegDecoder, webp_ffi::WebpDecoder,
};
use crate::types::DecodedImage;
use anyhow::Result;

lazy_static::lazy_static! {
    pub(crate) static ref HEAVY_FORMAT_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

/// Decodes raw image bytes into a DecodedImage structure.
pub fn decode_bytes(
    data: &[u8],
    mip: crate::types::MipLevel,
) -> (Result<DecodedImage>, &'static str) {
    // Check WebP magic number (RIFF....WEBP)
    let is_webp = data.len() > 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP";

    if is_webp {
        let decoder = WebpDecoder::new();
        match decoder.decode(data, mip) {
            Ok(img) => return (Ok(img), "WebP-FFI"),
            Err(e) => {
                tracing::warn!("Webp FFI failed, falling back to image crate: {}", e);
            }
        }
    }

    // Check AVIF magic number (....ftypavif or ....ftypavis)
    let is_avif = data.len() > 12 && (&data[4..12] == b"ftypavif" || &data[4..12] == b"ftypavis");

    if is_avif {
        let decoder = AvifDecoder::new();
        match decoder.decode(data, mip) {
            Ok(img) => return (Ok(img), "CrabbyAVIF"),
            Err(e) => {
                tracing::warn!("CrabbyAVIF failed, falling back to image crate: {}", e);
            }
        }
    }

    // Check HEIF magic number (....ftypheic, ....ftypheix, ....ftypmif1, etc.)
    let is_heif = data.len() > 12
        && &data[4..8] == b"ftyp"
        && (&data[8..12] == b"heic"
            || &data[8..12] == b"heix"
            || &data[8..12] == b"mif1"
            || &data[8..12] == b"msf1");

    if is_heif {
        let decoder = HeifDecoder;
        match decoder.decode(data, mip) {
            Ok(img) => return (Ok(img), "LibHeif"),
            Err(e) => {
                tracing::warn!("LibHeif failed, falling back to image crate: {}", e);
            }
        }
    }

    // Check JPEG XL magic number (priority: before JPEG/AVIF)
    // JPEG XL codestream: ff 0a
    // JPEG XL container (ISOBMFF): 00 00 00 0c 4a 58 4c 20 0d 0a 87 0a
    let is_jxl_codestream = data.len() >= 2 && data[0] == 0xff && data[1] == 0x0a;
    // More lenient JXL container check - just check "JXL " signature at offset 4
    let is_jxl_container = data.len() >= 8 && &data[4..8] == b"JXL ";
    let is_jxl = is_jxl_codestream || is_jxl_container;

    if is_jxl {
        let decoder = JxlDecoder::new();
        match decoder.decode(data, mip) {
            Ok(img) => {
                return (Ok(img), "JXL");
            }
            Err(e) => {
                tracing::warn!("JPEG XL decoder failed: {}, falling back to image crate", e);
            }
        }
    }

    // Check JPEG magic number (FF D8 FF)
    let is_jpeg = data.len() > 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF;

    if is_jpeg {
        let decoder = TurboJpegDecoder::new();
        match decoder.decode(data, mip) {
            Ok(img) => return (Ok(img), "TurboJPEG"),
            Err(e) => {
                tracing::warn!("TurboJPEG failed, falling back to image crate: {}", e);
            }
        }
    }

    // Check GIF magic number (GIF87a or GIF89a)
    let is_gif = data.len() >= 6 && (data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a"));

    if is_gif {
        let decoder = GifDecoder::new();
        match decoder.decode(data, mip) {
            Ok(img) => return (Ok(img), "Gif"),
            Err(e) => {
                tracing::warn!("GIF decoder failed, falling back to image crate: {}", e);
            }
        }
    }

    // Use fallback decoder (image crate)
    let decoder = FallbackDecoder::new();
    (decoder.decode(data, mip), "Fallback")
}

/// Returns true if the image format is considered "heavy" (JXL, AVIF, HEIC, WebP).
/// These formats typically use multi-threading internally and benefit from serialized execution.
pub fn is_heavy_format(data: &[u8]) -> bool {
    let is_webp = data.len() > 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP";
    let is_avif = data.len() > 12 && (&data[4..12] == b"ftypavif" || &data[4..12] == b"ftypavis");
    let is_heif = data.len() > 12
        && &data[4..8] == b"ftyp"
        && (&data[8..12] == b"heic"
            || &data[8..12] == b"heix"
            || &data[8..12] == b"mif1"
            || &data[8..12] == b"msf1");
    let is_jxl_codestream = data.len() >= 2 && data[0] == 0xff && data[1] == 0x0a;
    let is_jxl_container = data.len() >= 8 && &data[4..8] == b"JXL ";
    let is_jxl = is_jxl_codestream || is_jxl_container;

    is_webp || is_avif || is_heif || is_jxl
}
