use super::ImageDecoder;
use crate::types::DecodedImage;
use anyhow::{Result, anyhow};
use std::cell::RefCell;
use turbojpeg_sys as sys;

pub struct TurboJpegDecoder;

thread_local! {
    static TJ_HANDLE: RefCell<Option<sys::tjhandle>> = const { RefCell::new(None) };
}

impl TurboJpegDecoder {
    pub fn new() -> Self {
        Self
    }
}

impl Drop for TurboJpegDecoder {
    fn drop(&mut self) {
        TJ_HANDLE.with(|handle_cell| {
            if let Some(handle) = handle_cell.borrow_mut().take() {
                // SAFETY: `handle` was created by tjInitDecompress and is owned by this thread-local.
                unsafe { sys::tjDestroy(handle) };
            }
        });
    }
}

impl ImageDecoder for TurboJpegDecoder {
    fn decode(&self, data: &[u8], mip: crate::types::MipLevel) -> Result<DecodedImage> {
        TJ_HANDLE.with(|handle_cell| {
            let mut handle_opt = handle_cell.borrow_mut();
            if handle_opt.is_none() {
                // SAFETY: Initializes a new TurboJPEG decompressor handle owned by this thread-local.
                let handle = unsafe { sys::tjInitDecompress() };
                if handle.is_null() {
                    return Err(anyhow!("Failed to initialize TurboJPEG decompressor"));
                }
                *handle_opt = Some(handle);
            }

            let handle = handle_opt.unwrap();

            unsafe {
                // SAFETY: `handle` is initialized above and remains valid for the duration of
                // these TurboJPEG calls. Output buffers are allocated to the exact requested size.
                let mut width: i32 = 0;
                let mut height: i32 = 0;
                let mut subsamp: i32 = 0;

                if sys::tjDecompressHeader2(
                    handle,
                    data.as_ptr() as *mut _,
                    data.len() as _,
                    &mut width,
                    &mut height,
                    &mut subsamp,
                ) != 0
                {
                    let err = std::ffi::CStr::from_ptr(sys::tjGetErrorStr()).to_string_lossy();
                    return Err(anyhow!("TurboJPEG header read failed: {}", err));
                }

                // Map MipLevel to TurboJPEG hardware scaling factors
                // libjpeg-turbo supports n/8 scaling (1/8, 1/4, 3/8, 1/2, 5/8, 3/4, 7/8, 1/1)
                let (scale_n, scale_d) = match mip {
                    crate::types::MipLevel::Full => (1, 1),
                    crate::types::MipLevel::SevenEighths => (7, 8),
                    crate::types::MipLevel::ThreeQuarters => (3, 4), // 6/8
                    crate::types::MipLevel::FiveEighths => (5, 8),
                    crate::types::MipLevel::Half => (1, 2), // 4/8
                    crate::types::MipLevel::ThreeEighths => (3, 8),
                    crate::types::MipLevel::Quarter => (1, 4), // 2/8
                    crate::types::MipLevel::Eighth => (1, 8),
                };

                // Calculate scaled dimensions (TJSCALED macro logic)
                let scaled_width = (width * scale_n + scale_d - 1) / scale_d;
                let scaled_height = (height * scale_n + scale_d - 1) / scale_d;

                // Allocate buffer (We don't reuse the vector here yet because DecodedImage takes ownership)
                // However, we could potentially pass a reused buffer and clone it, or use a custom allocator.
                // For now, let's just use FASTDCT and scaling.
                let mut pixels = vec![0u8; (scaled_width * scaled_height * 4) as usize];

                if sys::tjDecompress2(
                    handle,
                    data.as_ptr() as *mut _,
                    data.len() as _,
                    pixels.as_mut_ptr(),
                    scaled_width,
                    scaled_width * 4,
                    scaled_height,
                    sys::TJPF_TJPF_RGBA,
                    sys::TJFLAG_FASTDCT as i32,
                ) != 0
                {
                    let err = std::ffi::CStr::from_ptr(sys::tjGetErrorStr()).to_string_lossy();
                    return Err(anyhow!("TurboJPEG decompression failed: {}", err));
                }

                // Extract EXIF and ICC from raw JPEG data (lazy extraction)
                let exif = crate::document::format_probe::extract_exif_summary(data);
                let icc_profile = crate::document::format_probe::probe_icc_profile_name(data);

                Ok(DecodedImage {
                    width: scaled_width as u32,
                    height: scaled_height as u32,
                    original_width: width as u32,
                    original_height: height as u32,
                    pixels,
                    icc_profile,
                    exif,
                })
            }
        })
    }
}
