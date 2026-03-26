// src/pipeline/decoders/gif.rs
use super::{DecodedFrame, FrameStream, ImageDecoder};
use crate::types::DecodedImage;
use anyhow::{Result, anyhow};
use gif::{Decoder, DisposalMethod};
use std::io::Cursor;
use std::time::Duration;

pub struct GifDecoder;

impl GifDecoder {
    pub fn new() -> Self {
        Self
    }
}

impl ImageDecoder for GifDecoder {
    fn decode(&self, data: &[u8], _mip: crate::types::MipLevel) -> Result<DecodedImage> {
        let mut decoder = Decoder::new(Cursor::new(data))?;

        // Get dimensions and palette FIRST
        let width = decoder.width() as u32;
        let height = decoder.height() as u32;
        let palette = decoder.palette()?.to_vec();

        // Read the first frame
        let frame = decoder
            .read_next_frame()?
            .ok_or_else(|| anyhow!("GIF has no frames"))?;

        // Clone frame data to avoid borrow conflicts
        let frame_buffer = frame.buffer.to_vec();
        let frame_left = frame.left;
        let frame_top = frame.top;
        let frame_width = frame.width;
        let frame_height = frame.height;
        let transparent = frame.transparent;

        // Composite frame onto canvas (handles partial frames)
        let mut canvas = vec![0u8; (width * height * 4) as usize];
        composite_frame(GifFrameContext {
            canvas: &mut canvas,
            canvas_width: width,
            canvas_height: height,
            frame_buffer: &frame_buffer,
            frame_left,
            frame_top,
            frame_width,
            frame_height,
            palette: &palette,
            transparent,
        });

        Ok(DecodedImage {
            width,
            height,
            original_width: width,
            original_height: height,
            pixels: canvas,
            icc_profile: None,
            exif: None,
        })
    }
}

struct GifFrameContext<'a> {
    pub canvas: &'a mut [u8],
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub frame_buffer: &'a [u8],
    pub frame_left: u16,
    pub frame_top: u16,
    pub frame_width: u16,
    pub frame_height: u16,
    pub palette: &'a [u8],
    pub transparent: Option<u8>,
}

/// Composite a GIF frame onto an RGBA canvas
fn composite_frame(ctx: GifFrameContext) {
    let frame_width = ctx.frame_width as usize;
    let frame_height = ctx.frame_height as usize;
    let frame_left = ctx.frame_left as usize;
    let frame_top = ctx.frame_top as usize;

    for y in 0..frame_height {
        for x in 0..frame_width {
            let canvas_x = frame_left + x;
            let canvas_y = frame_top + y;

            if canvas_x >= ctx.canvas_width as usize || canvas_y >= ctx.canvas_height as usize {
                continue;
            }

            let palette_index = ctx.frame_buffer[y * frame_width + x];
            let canvas_idx = (canvas_y * ctx.canvas_width as usize + canvas_x) * 4;

            // Handle transparency
            if Some(palette_index) == ctx.transparent {
                // Skip transparent pixels (keep canvas as-is)
                continue;
            }

            // Convert palette index to RGBA
            let pal_idx = (palette_index as usize) * 3;
            if pal_idx + 2 < ctx.palette.len() {
                ctx.canvas[canvas_idx] = ctx.palette[pal_idx];
                ctx.canvas[canvas_idx + 1] = ctx.palette[pal_idx + 1];
                ctx.canvas[canvas_idx + 2] = ctx.palette[pal_idx + 2];
                ctx.canvas[canvas_idx + 3] = 255;
            }
        }
    }
}

/// Check if data is a GIF file
pub fn is_gif(data: &[u8]) -> bool {
    data.len() >= 6 && (data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a"))
}

/// Get GIF info (width, height, frame count)
pub fn get_info(data: &[u8]) -> Option<(u32, u32, usize)> {
    let mut decoder = Decoder::new(Cursor::new(data)).ok()?;

    let width = decoder.width() as u32;
    let height = decoder.height() as u32;

    // Count frames
    let mut frame_count = 0;
    while let Ok(Some(_)) = decoder.read_next_frame() {
        frame_count += 1;
    }

    Some((width, height, frame_count))
}

/// Create an animated GIF stream
pub fn create_stream(data: Vec<u8>) -> Result<Box<dyn FrameStream>> {
    let decoder = Decoder::new(Cursor::new(data.clone()))?;

    let width = decoder.width() as u32;
    let height = decoder.height() as u32;
    let palette = decoder
        .palette()
        .map_err(|e| anyhow::anyhow!("Failed to get palette: {}", e))?
        .to_vec();

    Ok(Box::new(GifAnimStream {
        decoder,
        data,
        palette,
        width,
        height,
        canvas: vec![0u8; (width * height * 4) as usize],
        dispose_method: DisposalMethod::Any,
        is_at_start: true,
    }))
}

pub struct GifAnimStream {
    decoder: Decoder<Cursor<Vec<u8>>>,
    data: Vec<u8>,
    palette: Vec<u8>,
    width: u32,
    height: u32,
    canvas: Vec<u8>,
    dispose_method: DisposalMethod,
    is_at_start: bool,
}

// Safety: GifAnimStream is confined to a single worker thread
unsafe impl Send for GifAnimStream {}
unsafe impl Sync for GifAnimStream {}

impl FrameStream for GifAnimStream {
    fn next_frame(&mut self) -> Option<DecodedFrame> {
        loop {
            match self.decoder.read_next_frame() {
                Ok(Some(frame)) => {
                    let is_first = self.is_at_start;
                    self.is_at_start = false;

                    // GIF delay is in centiseconds (1/100 second), convert to milliseconds
                    let delay_ms = (frame.delay as u64) * 10;
                    let dispose = frame.dispose;
                    let transparent = frame.transparent;

                    // Apply disposal method from previous frame
                    match self.dispose_method {
                        DisposalMethod::Any => {}
                        DisposalMethod::Keep => {}
                        DisposalMethod::Background => {
                            self.canvas.fill(0);
                        }
                        DisposalMethod::Previous => {
                            self.canvas.fill(0);
                        }
                    }

                    let frame_buffer = frame.buffer.to_vec();
                    let frame_left = frame.left;
                    let frame_top = frame.top;
                    let frame_width = frame.width;
                    let frame_height = frame.height;

                    let palette = self
                        .decoder
                        .palette()
                        .map(|p| p.to_vec())
                        .unwrap_or_else(|_| self.palette.clone());

                    composite_frame(GifFrameContext {
                        canvas: &mut self.canvas,
                        canvas_width: self.width,
                        canvas_height: self.height,
                        frame_buffer: &frame_buffer,
                        frame_left,
                        frame_top,
                        frame_width,
                        frame_height,
                        palette: &palette,
                        transparent,
                    });

                    self.dispose_method = dispose;
                    let pixels = self.canvas.clone();

                    return Some(DecodedFrame {
                        pixels,
                        delay: Duration::from_millis(delay_ms),
                        is_first_frame: is_first,
                    });
                }
                Ok(None) => {
                    // Loop: reset decoder
                    // If we already tried to reset and failed to get a frame, stop to avoid infinite loop
                    if self.is_at_start && self.dispose_method == DisposalMethod::Any {
                        return None;
                    }

                    self.decoder = Decoder::new(Cursor::new(self.data.clone())).ok()?;
                    self.palette = self.decoder.palette().ok()?.to_vec();
                    self.canvas.fill(0);
                    self.is_at_start = true;
                    self.dispose_method = DisposalMethod::Any;
                    // continue loop to get the first frame
                }
                Err(_) => return None,
            }
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
