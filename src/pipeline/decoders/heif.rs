// src/pipeline/decoders/heif.rs
use crate::pipeline::decoders::ImageDecoder;
use crate::types::{DecodedImage, MipLevel};
use anyhow::{Result, anyhow};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;

// --- libheif FFI Bindings ---

#[repr(C)]
struct heif_context(c_void);
#[repr(C)]
struct heif_image_handle(c_void);
#[repr(C)]
struct heif_image(c_void);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct heif_error {
    pub code: c_int,
    pub subcode: c_int,
    pub message: *const c_char,
}

const HEIF_ERROR_OK: c_int = 0;
const HEIF_COLORSPACE_RGB: c_int = 1;
const HEIF_CHROMA_INTERLEAVED_RGBA: c_int = 11;
const HEIF_CHANNEL_INTERLEAVED: c_int = 10;

unsafe extern "C" {
    fn heif_context_alloc() -> *mut heif_context;
    fn heif_context_free(ctx: *mut heif_context);
    fn heif_context_read_from_memory_without_copy(
        ctx: *mut heif_context,
        data: *const c_void,
        size: usize,
        userdata: *const c_void,
    ) -> heif_error;
    fn heif_context_get_primary_image_handle(
        ctx: *mut heif_context,
        handle: *mut *mut heif_image_handle,
    ) -> heif_error;
    fn heif_image_handle_release(handle: *mut heif_image_handle);
    fn heif_image_handle_get_width(handle: *mut heif_image_handle) -> c_int;
    fn heif_image_handle_get_height(handle: *mut heif_image_handle) -> c_int;
    fn heif_decode_image(
        handle: *mut heif_image_handle,
        img: *mut *mut heif_image,
        colorspace: c_int,
        chroma: c_int,
        options: *const c_void,
    ) -> heif_error;
    fn heif_image_release(img: *mut heif_image);
    fn heif_image_get_plane_readonly(
        img: *mut heif_image,
        channel: c_int,
        stride: *mut c_int,
    ) -> *const u8;
}

pub struct HeifDecoder;

impl ImageDecoder for HeifDecoder {
    fn decode(&self, data: &[u8], _mip: MipLevel) -> Result<DecodedImage> {
        let (decoded_width, decoded_height, decoded_pixels) = unsafe {
            let decoded_width;
            let decoded_height;
            let decoded_pixels;

            let ctx = heif_context_alloc();
            if ctx.is_null() {
                return Err(anyhow!("Failed to allocate libheif context"));
            }

            let error = heif_context_read_from_memory_without_copy(
                ctx,
                data.as_ptr() as *const _,
                data.len(),
                ptr::null(),
            );

            if error.code != HEIF_ERROR_OK {
                heif_context_free(ctx);
                return Err(anyhow!("Failed to read HEIF data"));
            }

            let mut handle = ptr::null_mut();
            let error = heif_context_get_primary_image_handle(ctx, &mut handle);

            if error.code != HEIF_ERROR_OK {
                heif_context_free(ctx);
                return Err(anyhow!("Failed to get primary image handle"));
            }

            decoded_width = heif_image_handle_get_width(handle) as u32;
            decoded_height = heif_image_handle_get_height(handle) as u32;

            let mut img = ptr::null_mut();
            let error = heif_decode_image(
                handle,
                &mut img,
                HEIF_COLORSPACE_RGB,
                HEIF_CHROMA_INTERLEAVED_RGBA,
                ptr::null(),
            );

            if error.code != HEIF_ERROR_OK {
                heif_image_handle_release(handle);
                heif_context_free(ctx);
                return Err(anyhow!("Failed to decode HEIF image"));
            }

            let mut stride = 0;
            let pixels_ptr =
                heif_image_get_plane_readonly(img, HEIF_CHANNEL_INTERLEAVED, &mut stride);

            if pixels_ptr.is_null() {
                heif_image_release(img);
                heif_image_handle_release(handle);
                heif_context_free(ctx);
                return Err(anyhow!("Failed to get pixels from HEIF image"));
            }

            if stride <= 0 {
                heif_image_release(img);
                heif_image_handle_release(handle);
                heif_context_free(ctx);
                return Err(anyhow!("Invalid HEIF stride: {}", stride));
            }

            let pixel_bytes = (stride as usize)
                .checked_mul(decoded_height as usize)
                .ok_or_else(|| anyhow!("HEIF image size overflow"))?;

            // SAFETY: `pixels_ptr` is non-null, `stride` is validated positive, and the copied
            // length is checked with `checked_mul` to avoid overflow before creating the slice.
            decoded_pixels = slice::from_raw_parts(pixels_ptr, pixel_bytes).to_vec();

            // Cleanup
            heif_image_release(img);
            heif_image_handle_release(handle);
            heif_context_free(ctx);
            (decoded_width, decoded_height, decoded_pixels)
        };

        Ok(DecodedImage {
            width: decoded_width,
            height: decoded_height,
            original_width: decoded_width,
            original_height: decoded_height,
            pixels: decoded_pixels,
            icc_profile: None,
            exif: None,
        })
    }
}
