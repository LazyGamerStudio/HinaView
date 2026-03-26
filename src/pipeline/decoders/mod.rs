use crate::types::DecodedImage;
use anyhow::Result;
use std::time::Duration;

pub struct DecodedFrame {
    pub pixels: Vec<u8>,
    pub delay: Duration,
    pub is_first_frame: bool,
}

pub trait FrameStream: Send + Sync {
    fn next_frame(&mut self) -> Option<DecodedFrame>;
    fn dimensions(&self) -> (u32, u32);
}

pub trait ImageDecoder: Send + Sync {
    /// Decode a single-frame image
    /// Optional mip can be used for faster, lower-resolution decoding (hardware scaling)
    fn decode(&self, data: &[u8], mip: crate::types::MipLevel) -> Result<DecodedImage>;

    /// Check if the image is animated (default: false)
    #[allow(dead_code)]
    fn is_animated(&self, _data: &[u8]) -> bool {
        false
    }
}

pub mod avif;
pub mod fallback;
pub mod gif;
pub mod heif;
pub mod jxl;
pub mod turbo_jpeg;
pub mod webp_ffi;
