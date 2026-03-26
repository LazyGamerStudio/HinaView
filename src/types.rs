// src/types.rs

use crate::document::format_probe::ExifSummary;

pub type PageId = usize;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum MipLevel {
    Eighth,        // 1/8 (0.125)
    Quarter,       // 2/8 (0.250)
    ThreeEighths,  // 3/8 (0.375)
    Half,          // 4/8 (0.500)
    FiveEighths,   // 5/8 (0.625)
    ThreeQuarters, // 6/8 (0.750)
    SevenEighths,  // 7/8 (0.875)
    Full,          // 8/8 (1.000)
}

#[derive(Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub original_width: u32,
    pub original_height: u32,
    pub pixels: Vec<u8>, // RGBA8
    /// ICC profile name extracted during decode (lazy extraction)
    pub icc_profile: Option<String>,
    /// EXIF summary extracted during decode (lazy extraction)
    pub exif: Option<ExifSummary>,
}

/// Layout mode representing user intent.
/// Moved from view::layout_mode to avoid document → view dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Single,
    #[allow(dead_code)]
    Dual {
        rtl: bool,
        first_page_offset: bool,
    },
    #[allow(dead_code)]
    VerticalScroll,
}

/// Represents the geometry of a single tile within the original image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
