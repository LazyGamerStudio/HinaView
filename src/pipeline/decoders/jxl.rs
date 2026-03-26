// src/pipeline/decoders/jxl.rs
use super::ImageDecoder;
use crate::types::{DecodedImage, MipLevel};
use anyhow::{Result, anyhow};
use std::ffi::c_void;

// Use libjxl-sys bindings
use libjxl_sys::bindings::*;

lazy_static::lazy_static! {
    static ref JXL_PARALLEL_RUNNER: JxlThreadRunner = JxlThreadRunner::new();
}

struct JxlThreadRunner {
    pub runner: *mut std::ffi::c_void,
}

unsafe impl Send for JxlThreadRunner {}
unsafe impl Sync for JxlThreadRunner {}

impl JxlThreadRunner {
    fn new() -> Self {
        unsafe {
            // SAFETY: We request a process-global thread runner once and store the raw handle for
            // the lifetime of the program.
            let total_cores = num_cpus::get();
            // Since heavy decoding is serialized (one at a time), we utilize most cores
            // for maximum speed while leaving enough for UI and other background tasks.
            // Requirement: max_cores - 1 for up to 4 cores, max_cores - 2 for more than 4 cores.
            let num_threads = if total_cores <= 4 {
                total_cores.saturating_sub(1).max(1)
            } else {
                total_cores.saturating_sub(2).max(1)
            };
            let runner = JxlThreadParallelRunnerCreate(std::ptr::null(), num_threads);
            Self { runner }
        }
    }
}

pub struct JxlDecoder;

impl JxlDecoder {
    pub fn new() -> Self {
        Self
    }
}

impl ImageDecoder for JxlDecoder {
    fn decode(&self, data: &[u8], _mip: MipLevel) -> Result<DecodedImage> {
        unsafe {
            // SAFETY: The decoder handle returned here is owned within this function and destroyed
            // on every exit path below.
            // Create decoder
            let decoder = JxlDecoderCreate(std::ptr::null());
            if decoder.is_null() {
                return Err(anyhow!("Failed to create JXL decoder"));
            }

            // Use global parallel runner
            if JxlDecoderSetParallelRunner(
                decoder,
                Some(JxlThreadParallelRunner),
                JXL_PARALLEL_RUNNER.runner,
            ) != JxlDecoderStatus_JXL_DEC_SUCCESS
            {
                JxlDecoderDestroy(decoder);
                return Err(anyhow!("Failed to set JXL parallel runner"));
            }

            // Subscribe to events we need
            let events = JxlDecoderStatus_JXL_DEC_BASIC_INFO
                | JxlDecoderStatus_JXL_DEC_COLOR_ENCODING
                | JxlDecoderStatus_JXL_DEC_FULL_IMAGE;

            if JxlDecoderSubscribeEvents(decoder, events as i32) != JxlDecoderStatus_JXL_DEC_SUCCESS
            {
                JxlDecoderDestroy(decoder);
                return Err(anyhow!("Failed to subscribe to JXL events"));
            }

            // Set input data
            if JxlDecoderSetInput(decoder, data.as_ptr(), data.len())
                != JxlDecoderStatus_JXL_DEC_SUCCESS
            {
                JxlDecoderDestroy(decoder);
                return Err(anyhow!("Failed to set JXL input"));
            }

            JxlDecoderCloseInput(decoder);

            let mut basic_info = std::mem::zeroed::<JxlBasicInfo>();
            let mut width = 0u32;
            let mut height = 0u32;
            let mut pixels: Option<Vec<u8>> = None;

            let final_result = loop {
                let result = JxlDecoderProcessInput(decoder);

                if result == JxlDecoderStatus_JXL_DEC_SUCCESS {
                    break Ok(());
                } else if result == JxlDecoderStatus_JXL_DEC_ERROR {
                    break Err(anyhow!("JXL decoder error"));
                } else if result == JxlDecoderStatus_JXL_DEC_NEED_MORE_INPUT {
                    break Err(anyhow!("JXL needs more input (truncated file?)"));
                } else if result == JxlDecoderStatus_JXL_DEC_BASIC_INFO {
                    if JxlDecoderGetBasicInfo(decoder, &mut basic_info)
                        == JxlDecoderStatus_JXL_DEC_SUCCESS
                    {
                        width = basic_info.xsize;
                        height = basic_info.ysize;
                    } else {
                        break Err(anyhow!("Failed to get JXL basic info"));
                    }
                } else if result == JxlDecoderStatus_JXL_DEC_COLOR_ENCODING {
                    // Continue
                } else if result == JxlDecoderStatus_JXL_DEC_NEED_IMAGE_OUT_BUFFER {
                    if width == 0 || height == 0 {
                        break Err(anyhow!("No dimensions from basic info"));
                    }

                    let pixel_count = (width * height) as usize;
                    let buffer_size = pixel_count * 4;
                    let mut buffer = vec![0u8; buffer_size];

                    let format = JxlPixelFormat {
                        num_channels: 4,
                        data_type: JxlDataType_JXL_TYPE_UINT8,
                        endianness: JxlEndianness_JXL_NATIVE_ENDIAN,
                        align: 1,
                    };

                    if JxlDecoderSetImageOutBuffer(
                        decoder,
                        &format,
                        buffer.as_mut_ptr() as *mut c_void,
                        buffer_size,
                    ) != JxlDecoderStatus_JXL_DEC_SUCCESS
                    {
                        break Err(anyhow!("Failed to set JXL output buffer"));
                    }

                    pixels = Some(buffer);
                } else if result == JxlDecoderStatus_JXL_DEC_FULL_IMAGE {
                    if let Some(pixels_buf) = pixels.take() {
                        JxlDecoderDestroy(decoder);

                        return Ok(DecodedImage {
                            width,
                            height,
                            original_width: width,
                            original_height: height,
                            pixels: pixels_buf,
                            icc_profile: None,
                            exif: None,
                        });
                    } else {
                        break Err(anyhow!("No pixel buffer for full image"));
                    }
                }
            };

            JxlDecoderDestroy(decoder);

            final_result.and_then(|_| {
                if width == 0 || height == 0 {
                    Err(anyhow!("Failed to decode JXL: no dimensions"))
                } else {
                    Err(anyhow!("JXL decoding incomplete"))
                }
            })
        }
    }
}
