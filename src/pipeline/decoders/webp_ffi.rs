use super::{DecodedFrame, FrameStream};
use crate::types::DecodedImage;
use anyhow::{Result, anyhow};
use libwebp_sys::*;
use std::ptr;
use std::slice;
use std::time::Duration;
use tracing::{debug, error};

pub struct WebpDecoder;

impl WebpDecoder {
    pub fn new() -> Self {
        Self
    }
}

pub struct WebpAnimStream {
    decoder: *mut WebPAnimDecoder,
    last_timestamp: i32,
    width: u32,
    height: u32,
    frame_count: usize,
    total_frames: usize,
    is_at_start: bool,
    _source_data: Vec<u8>,
}

// Safety: The raw pointer `decoder` is confined to this struct, and `next_frame` requires `&mut self`.
// As long as `WebpAnimStream` is moved as a whole and accessed exclusively (which `FrameStream` implies via `&mut self`),
// it should be safe to send across threads.
unsafe impl Send for WebpAnimStream {}
unsafe impl Sync for WebpAnimStream {}

impl FrameStream for WebpAnimStream {
    fn next_frame(&mut self) -> Option<DecodedFrame> {
        loop {
            let mut buf_ptr: *mut u8 = ptr::null_mut();
            let mut timestamp: i32 = 0;

            unsafe {
                // SAFETY: `self.decoder` is a live decoder owned by this stream, and WebP fills
                // the out-pointers for the duration of the call.
                // Get the next frame.
                // Returns true (1) on success, false (0) if no more frames.
                if WebPAnimDecoderGetNext(self.decoder, &mut buf_ptr, &mut timestamp) != 0 {
                    // Safety check: ensure buffer pointer is valid
                    if buf_ptr.is_null() {
                        error!("[WebP] Decoder returned null buffer");
                        return None;
                    }

                    let is_first = self.is_at_start;
                    self.is_at_start = false;

                    let delay_ms = if self.last_timestamp < 0 {
                        // First frame, delay is from 0 to timestamp
                        timestamp
                    } else {
                        timestamp - self.last_timestamp
                    };

                    // Clamp delay to be reasonable (>= 10ms)
                    let delay_ms = delay_ms.max(10) as u64;

                    self.last_timestamp = timestamp;
                    self.frame_count = self.frame_count.wrapping_add(1);

                    // Log only on first frame or loop
                    if self.frame_count == 1 {
                        debug!(
                            "[WebP] Stream started: {}x{}, {} frames",
                            self.width, self.height, self.total_frames
                        );
                    }

                    // Copy pixels
                    let size = (self.width * self.height * 4) as usize;
                    // SAFETY: `buf_ptr` was checked non-null above and WebP returns an RGBA buffer
                    // sized exactly width * height * 4 for the current frame.
                    let pixels = slice::from_raw_parts(buf_ptr, size).to_vec();

                    return Some(DecodedFrame {
                        pixels,
                        delay: Duration::from_millis(delay_ms),
                        is_first_frame: is_first,
                    });
                } else {
                    // Loop: Reset decoder
                    // If we just reset and still get no frames, stop to avoid infinite loop
                    if self.is_at_start && self.frame_count == 0 {
                        error!("[WebP] Failed to get any frames even after reset");
                        return None;
                    }

                    // SAFETY: `self.decoder` remains valid for the lifetime of the stream.
                    WebPAnimDecoderReset(self.decoder);
                    self.last_timestamp = 0;
                    self.frame_count = 0;
                    self.is_at_start = true;

                    // continue loop to get the first frame
                }
            }
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for WebpAnimStream {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: `self.decoder` is owned by this stream and must be destroyed exactly once.
            WebPAnimDecoderDelete(self.decoder);
        }
    }
}

impl super::ImageDecoder for WebpDecoder {
    fn decode(&self, data: &[u8], mip: crate::types::MipLevel) -> Result<DecodedImage> {
        unsafe {
            // SAFETY: `config` is zero-initialized before being passed to libwebp initialization.
            let mut config: WebPDecoderConfig = std::mem::zeroed();
            if !WebPInitDecoderConfig(&mut config) {
                return Err(anyhow!("Failed to initialize WebP decoder config"));
            }

            // Get original dimensions
            if WebPGetFeatures(data.as_ptr(), data.len(), &mut config.input)
                != VP8StatusCode::VP8_STATUS_OK
            {
                return Err(anyhow!("Invalid WebP header or features"));
            }

            let orig_width = config.input.width as u32;
            let orig_height = config.input.height as u32;

            // Apply scaling if mip is not Full
            if mip != crate::types::MipLevel::Full {
                let scale = match mip {
                    crate::types::MipLevel::SevenEighths => 0.875f32,
                    crate::types::MipLevel::ThreeQuarters => 0.75f32,
                    crate::types::MipLevel::FiveEighths => 0.625f32,
                    crate::types::MipLevel::Half => 0.5f32,
                    crate::types::MipLevel::ThreeEighths => 0.375f32,
                    crate::types::MipLevel::Quarter => 0.25f32,
                    crate::types::MipLevel::Eighth => 0.125f32,
                    crate::types::MipLevel::Full => 1.0f32,
                };

                config.options.use_scaling = 1;
                config.options.scaled_width = (orig_width as f32 * scale).ceil() as i32;
                config.options.scaled_height = (orig_height as f32 * scale).ceil() as i32;
            }

            // Set output format to RGBA
            config.output.colorspace = WEBP_CSP_MODE::MODE_RGBA;

            // Enable multi-threading for faster decoding (since WebP is now classified as heavy)
            config.options.use_threads = 1;

            // Perform the decode
            if WebPDecode(data.as_ptr(), data.len(), &mut config) != VP8StatusCode::VP8_STATUS_OK {
                return Err(anyhow!("WebP decode failed"));
            }

            let out_width = config.output.width as u32;
            let out_height = config.output.height as u32;

            // Transfer ownership of pixels to a Vec
            let pixels_size = (out_width * out_height * 4) as usize;
            // SAFETY: libwebp populated `config.output` after a successful decode, and the RGBA
            // buffer contains exactly width * height * 4 bytes.
            let pixels = slice::from_raw_parts(config.output.u.RGBA.rgba, pixels_size).to_vec();

            // Free the internal buffer
            // SAFETY: `config.output` owns the decoder output buffer returned by WebPDecode.
            WebPFreeDecBuffer(&mut config.output);

            // Extract ICC and EXIF from raw WebP data
            let icc_profile = crate::document::format_probe::probe_icc_profile_name(data);
            let exif = crate::document::format_probe::extract_exif_summary(data);

            Ok(DecodedImage {
                width: out_width,
                height: out_height,
                original_width: orig_width,
                original_height: orig_height,
                pixels,
                icc_profile,
                exif,
            })
        }
    }

    fn is_animated(&self, data: &[u8]) -> bool {
        is_animated(data)
    }
}

pub fn is_animated(data: &[u8]) -> bool {
    // Check extended format features bit
    // VP8X chunk:
    // bytes 0-3: 'RIFF'
    // bytes 4-7: size
    // bytes 8-11: 'WEBP'
    // bytes 12-15: 'VP8X'
    // byte 20: flags (bit 1 is Animation)

    if data.len() < 30 {
        return false;
    }

    // Check signature
    if &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return false;
    }

    if &data[12..16] == b"VP8X" {
        // Animation bit is 2nd bit of byte 20 (0x02)
        // Flags are at offset 20 in file (offset 8 in VP8X chunk content)
        // VP8X header is 10 bytes (tag 4 + size 4 + flags 4 + dims 6...)
        // Wait, standard VP8X header structure:
        // 0-3: VP8X
        // 4-7: Chunk Size (10)
        // 8-11: Flags (1 byte) + Reserved (3 bytes)
        // Flags byte is at index 20 of the FILE.
        return (data[20] & 0x02) != 0;
    }

    false
}

/// Extract WebP image info (width, height, is_animated) using FFI.
/// This is the single source of truth for WebP metadata, delegated from format_probe.
pub fn get_info(data: &[u8]) -> Option<(u32, u32, bool)> {
    if data.len() < 12 {
        return None;
    }

    unsafe {
        let mut width: i32 = 0;
        let mut height: i32 = 0;

        // SAFETY: `data` is a valid byte slice and width/height pointers are stack locals.
        if WebPGetInfo(data.as_ptr(), data.len(), &mut width, &mut height) == 0 {
            return None;
        }

        let animated = is_animated(data);

        Some((width as u32, height as u32, animated))
    }
}

pub fn create_stream(data: Vec<u8>) -> Result<Box<dyn FrameStream>> {
    unsafe {
        // SAFETY: `data` is stored in the resulting stream, so the byte buffer remains alive for
        // the lifetime of the decoder that borrows it through `webp_data`.
        let webp_data = WebPData {
            bytes: data.as_ptr(),
            size: data.len(),
        };

        // SAFETY: `options` is zero-initialized before libwebp fills it.
        let mut options: WebPAnimDecoderOptions = std::mem::zeroed();
        if WebPAnimDecoderOptionsInit(&mut options) == 0 {
            return Err(anyhow!("Failed to init WebP anim options"));
        }

        // Use RGBA color mode
        options.color_mode = WEBP_CSP_MODE::MODE_RGBA;

        // Create decoder
        let decoder = WebPAnimDecoderNew(&webp_data, &options);
        if decoder.is_null() {
            return Err(anyhow!("Failed to create WebPAnimDecoder"));
        }

        // SAFETY: `info` is zero-initialized before libwebp writes animation metadata into it.
        let mut info: WebPAnimInfo = std::mem::zeroed();
        if WebPAnimDecoderGetInfo(decoder, &mut info) == 0 {
            WebPAnimDecoderDelete(decoder);
            return Err(anyhow!("Failed to get animation info"));
        }

        Ok(Box::new(WebpAnimStream {
            decoder,
            last_timestamp: 0,
            width: info.canvas_width,
            height: info.canvas_height,
            frame_count: 0,
            total_frames: info.frame_count as usize,
            is_at_start: true,
            _source_data: data,
        }))
    }
}
