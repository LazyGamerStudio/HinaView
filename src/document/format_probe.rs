// src/document/format_probe.rs
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ExifSummary {
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub f_stop: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<String>,
    pub datetime: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Folder,
    Zip,
    Image,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
    Gif,
    Bmp,
    #[allow(dead_code)]
    Avif,
    Heif,
    Jxl,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    #[allow(dead_code)]
    pub format: ImageFormat,
    pub is_animated: bool,
}

/// Main entry point for detecting file container format (Folder, Zip, etc.)
pub fn probe_format(path: &Path) -> FileFormat {
    if path.is_dir() {
        return FileFormat::Folder;
    }

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return FileFormat::Unknown,
    };

    let mut header = [0u8; 16];
    let n = file.read(&mut header).unwrap_or(0);

    if n < 4 {
        return FileFormat::Unknown;
    }

    if &header[0..4] == b"PK\x03\x04" {
        return FileFormat::Zip;
    }

    // Quick image check
    if is_image_header(&header[..n]) {
        return FileFormat::Image;
    }

    // Fallback to extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();

        // ZIP formats
        if ext_str == "zip" || ext_str == "cbz" {
            return FileFormat::Zip;
        }

        // Image formats without reliable magic numbers (TGA, etc.)
        if matches!(
            ext_str.as_str(),
            "tga" | "exr" | "hdr" | "pbm" | "pgm" | "ppm" | "pnm"
        ) {
            return FileFormat::Image;
        }
    }

    FileFormat::Unknown
}

/// Unified entry point for extracting image metadata from a data buffer.
/// This is the SINGLE SOURCE OF TRUTH for image dimensions and formats.
pub fn probe_image_metadata(data: &[u8]) -> Option<ImageMetadata> {
    if data.len() < 4 {
        return None;
    }

    // 1. Detect Image Type
    let is_avif = data.len() > 12 && (&data[4..12] == b"ftypavif" || &data[4..12] == b"ftypavis");
    let is_heif = data.len() > 12
        && &data[4..8] == b"ftyp"
        && (&data[8..12] == b"heic"
            || &data[8..12] == b"heix"
            || &data[8..12] == b"mif1"
            || &data[8..12] == b"msf1");

    let mut format = ImageFormat::Unknown;
    let mut width = 0;
    let mut height = 0;

    if is_avif || is_heif {
        // Lightweight ISOBMFF box parser to extract width and height
        if let Some((w, h)) = parse_isobmff_dimensions(data) {
            format = if is_avif {
                ImageFormat::Avif
            } else {
                ImageFormat::Heif
            };
            width = w;
            height = h;
        } else if is_avif {
            // Fallback to crabby_avif for AVIF
            use crabby_avif::decoder::Decoder;
            let mut decoder = Decoder::default();
            unsafe {
                // SAFETY: `data` is a valid byte slice that remains alive for the parser call.
                let _ = decoder.set_io_raw(data.as_ptr(), data.len());
            }
            if decoder.parse().is_ok() {
                format = ImageFormat::Avif;
                width = decoder.image().map(|i| i.width).unwrap_or(0);
                height = decoder.image().map(|i| i.height).unwrap_or(0);
            }
        }
    } else {
        if let Ok(it) = imagesize::image_type(data) {
            format = match it {
                imagesize::ImageType::Jpeg => ImageFormat::Jpeg,
                imagesize::ImageType::Png => ImageFormat::Png,
                imagesize::ImageType::Webp => ImageFormat::Webp,
                imagesize::ImageType::Gif => ImageFormat::Gif,
                imagesize::ImageType::Bmp => ImageFormat::Bmp,
                imagesize::ImageType::Heif(_) => ImageFormat::Heif,
                imagesize::ImageType::Jxl => ImageFormat::Jxl,
                _ => ImageFormat::Unknown,
            };
        }
        if let Ok(sz) = imagesize::blob_size(data) {
            width = sz.width as u32;
            height = sz.height as u32;
        }
    }

    if format == ImageFormat::Unknown || width == 0 || height == 0 {
        return None;
    }

    let mut is_animated = format == ImageFormat::Gif;

    // For WebP, we still use FFI to check for animation if needed
    if format == ImageFormat::Webp
        && let Some(ffi_meta) = crate::pipeline::decoders::webp_ffi::get_info(data)
    {
        is_animated = ffi_meta.2;
    }

    Some(ImageMetadata {
        width,
        height,
        format,
        is_animated,
    })
}

pub fn extract_exif_summary(data: &[u8]) -> Option<ExifSummary> {
    use exif::{In, Reader, Tag};
    use std::io::Cursor;

    let mut cursor = Cursor::new(data);

    // First try read_from_container (handles JPEG/HEIF/PNG containers)
    // Then try read_raw (handles raw TIFF header chunks from AVIF/WebP)
    let exif_res = Reader::new().read_from_container(&mut cursor).or_else(|_| {
        cursor.set_position(0);
        Reader::new().read_raw(data.to_vec())
    });

    let exif = exif_res.ok()?;

    let make = exif
        .get_field(Tag::Make, In::PRIMARY)
        .map(|f| f.display_value().with_unit(&exif).to_string());
    let model = exif
        .get_field(Tag::Model, In::PRIMARY)
        .map(|f| f.display_value().with_unit(&exif).to_string());
    let camera = match (make, model) {
        (Some(a), Some(b)) => Some(format!("{} {}", a.trim(), b.trim())),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    let lens = exif
        .get_field(Tag::LensModel, In::PRIMARY)
        .map(|f| f.display_value().with_unit(&exif).to_string());
    let f_stop = exif
        .get_field(Tag::FNumber, In::PRIMARY)
        .map(|f| f.display_value().with_unit(&exif).to_string());
    let shutter_speed = exif
        .get_field(Tag::ExposureTime, In::PRIMARY)
        .map(|f| f.display_value().with_unit(&exif).to_string());
    let iso = exif
        .get_field(Tag::ISOSpeed, In::PRIMARY)
        .or_else(|| exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY))
        .map(|f| f.display_value().with_unit(&exif).to_string());
    let datetime = exif
        .get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .or_else(|| exif.get_field(Tag::DateTime, In::PRIMARY))
        .map(|f| f.display_value().with_unit(&exif).to_string());

    let summary = ExifSummary {
        camera,
        lens,
        f_stop,
        shutter_speed,
        iso,
        datetime,
    };

    if summary.camera.is_none()
        && summary.lens.is_none()
        && summary.f_stop.is_none()
        && summary.shutter_speed.is_none()
        && summary.iso.is_none()
        && summary.datetime.is_none()
    {
        None
    } else {
        Some(summary)
    }
}

pub fn probe_icc_profile_name(data: &[u8]) -> Option<String> {
    if data.starts_with(&[0xFF, 0xD8]) {
        return parse_jpeg_icc(data);
    }
    if data.starts_with(b"\x89PNG\r\n\x1A\n") {
        return parse_png_icc(data);
    }
    // WebP check
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        if data.windows(4).any(|w| w == b"ICCP") {
            return Some("Embedded ICC".to_string());
        }
        return None;
    }
    // If it starts with 'acsp' or looks like a raw ICC chunk (starts with size)
    // Raw ICC profile data check: 36-39 is 'acsp'
    if data.len() >= 40 && &data[36..40] == b"acsp" {
        return Some("Embedded ICC Profile".to_string());
    }
    None
}

fn parse_jpeg_icc(data: &[u8]) -> Option<String> {
    let mut i = 2usize;
    while i + 4 < data.len() {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = data[i + 1];
        if marker == 0xDA || marker == 0xD9 {
            break;
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            i += 2;
            continue;
        }
        let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        if seg_len < 2 || i + 2 + seg_len > data.len() {
            break;
        }
        let payload = &data[i + 4..i + 2 + seg_len];
        if marker == 0xE2 && payload.starts_with(b"ICC_PROFILE\0") {
            return Some("ICC Profile".to_string());
        }
        i += 2 + seg_len;
    }
    None
}

fn parse_png_icc(data: &[u8]) -> Option<String> {
    let mut i = 8usize;
    while i + 12 <= data.len() {
        let len = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        if i + 12 + len > data.len() {
            break;
        }
        let chunk_type = &data[i + 4..i + 8];
        if chunk_type == b"iCCP" {
            let chunk_data = &data[i + 8..i + 8 + len];
            if let Some(null_pos) = chunk_data.iter().position(|b| *b == 0) {
                let name = String::from_utf8_lossy(&chunk_data[..null_pos]).to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
            return Some("ICC Profile".to_string());
        }
        i += 12 + len;
    }
    None
}

fn is_image_header(data: &[u8]) -> bool {
    // Check magic numbers for all supported image formats
    if data.len() < 2 {
        return false;
    }

    // JPEG: FF D8
    // PNG: 89 50 4E 47
    // GIF: 47 49 46 38
    // BMP: 42 4D
    // WebP: 52 49 46 46 (RIFF) + 57 45 42 50 (WEBP)
    // TIFF: 49 49 (II) or 4D 4D (MM)
    // DDS: 44 44 53 20 (DDS )
    // EXR: 76 2F 31 01
    // HDR: 23 3F 52 41
    // PNM: 50 (P) + [3456] (P4-P6)

    // AVIF: ....ftypavif or ....ftypavis
    let is_avif = data.len() > 12 && (&data[4..12] == b"ftypavif" || &data[4..12] == b"ftypavis");

    // HEIF: ....ftypheic, ....ftypmif1, etc.
    let is_heif = data.len() > 12
        && &data[4..8] == b"ftyp"
        && (&data[8..12] == b"heic"
            || &data[8..12] == b"heix"
            || &data[8..12] == b"mif1"
            || &data[8..12] == b"msf1");

    // JXL: FF 0A (Codestream) or ....JXL  (Container)
    let is_jxl = (data.len() >= 2 && data[0] == 0xFF && data[1] == 0x0A)
        || (data.len() >= 8 && &data[4..8] == b"JXL ");

    data.starts_with(b"\x89PNG")           // PNG
        || data.starts_with(b"\xFF\xD8")   // JPEG
        || data.starts_with(b"RIFF")       // WebP
        || data.starts_with(b"GIF8")       // GIF
        || data.starts_with(b"BM")         // BMP
        || is_avif
        || is_heif
        || is_jxl
        || data.starts_with(b"II")         // TIFF (little endian)
        || data.starts_with(b"MM")         // TIFF (big endian)
        || data.starts_with(b"DDS ")       // DDS
        || data.starts_with(b"\x76\x2F\x31\x01")  // EXR
        || data.starts_with(b"#?RADIANCE") // HDR
        || data.starts_with(b"P") // PNM (P1-P6)
}

/// Lightweight ISOBMFF parser to find 'ispe' box and extract AVIF dimensions.
fn parse_isobmff_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    let mut i = 0;
    while i + 8 <= data.len() {
        let box_size =
            u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        let box_type = &data[i + 4..i + 8];

        if box_size < 8 {
            break;
        } // Invalid box size

        if box_type == b"meta" {
            // Inside 'meta', skip 4 bytes (version/flags) and recurse
            return parse_isobmff_dimensions(&data[i + 12..(i + box_size).min(data.len())]);
        } else if box_type == b"iprp" || box_type == b"ipco" {
            // Recurse into container boxes
            return parse_isobmff_dimensions(&data[i + 8..(i + box_size).min(data.len())]);
        } else if box_type == b"ispe" {
            // Found Image Spatial Extents box
            // structure: 4 bytes version/flags, 4 bytes width, 4 bytes height
            if i + 16 <= data.len() {
                let width =
                    u32::from_be_bytes([data[i + 12], data[i + 13], data[i + 14], data[i + 15]]);
                let height =
                    u32::from_be_bytes([data[i + 16], data[i + 17], data[i + 18], data[i + 19]]);
                return Some((width, height));
            }
        }

        i += box_size;
    }
    None
}
