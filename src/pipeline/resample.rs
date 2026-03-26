// src/pipeline/resample.rs
use crate::types::{DecodedImage, MipLevel};
use fast_image_resize::images::Image;
use fast_image_resize::{ResizeAlg, ResizeOptions, Resizer};
use tracing::{debug, warn};

/// Applies high-performance resampling using the advanced pipeline.
/// 1. Progressive Box Downscale for large images (> 4096px)
/// 2. Halftone detection for conditional pre-filtering and algorithm switching
/// 3. Gaussian pre-blur (only for halftone images)
/// 4. Final resize with Mitchell (for halftone) or Lanczos3 (for normal)
pub fn apply_mip(image: DecodedImage, mip: MipLevel) -> DecodedImage {
    if mip == MipLevel::Full {
        return image;
    }

    let DecodedImage {
        width: image_width,
        height: image_height,
        original_width: orig_width,
        original_height: orig_height,
        pixels: original_pixels,
        icc_profile,
        exif,
    } = image;

    let scale = match mip {
        MipLevel::Full => 1.0,
        MipLevel::SevenEighths => 0.875,
        MipLevel::ThreeQuarters => 0.75,
        MipLevel::FiveEighths => 0.625,
        MipLevel::Half => 0.5,
        MipLevel::ThreeEighths => 0.375,
        MipLevel::Quarter => 0.25,
        MipLevel::Eighth => 0.125,
    };

    // Use original dimensions to calculate the TRUE target size for this mip level
    let target_width = (orig_width as f32 * scale).ceil() as u32;
    let target_height = (orig_height as f32 * scale).ceil() as u32;

    // Check if the image is already at or smaller than the target mip size.
    if image_width <= target_width && image_height <= target_height {
        return DecodedImage {
            width: image_width,
            height: image_height,
            original_width: orig_width,
            original_height: orig_height,
            pixels: original_pixels,
            icc_profile,
            exif,
        };
    }

    // Ensure minimum dimensions
    if target_width == 0 || target_height == 0 {
        return DecodedImage {
            width: image_width,
            height: image_height,
            original_width: orig_width,
            original_height: orig_height,
            pixels: original_pixels,
            icc_profile,
            exif,
        };
    }

    let start_total = std::time::Instant::now();
    let mut resizer = Resizer::new();
    let mut current_pixels = original_pixels;
    let mut current_width = image_width;
    let mut current_height = image_height;

    // 1. Progressive Box Downscale for Large Images
    // Optimized for massive downscaling (e.g. 8000px -> 1000px)
    if (current_width > 4096 || current_height > 4096) && scale < 0.5 {
        let mut loop_count = 0;
        loop {
            let next_w = (current_width / 2).max(target_width);
            let next_h = (current_height / 2).max(target_height);

            // Stop if we can't shrink effectively or we are near target
            if next_w >= current_width || next_w < target_width * 2 {
                break;
            }

            let src_img = match Image::from_vec_u8(
                current_width,
                current_height,
                current_pixels.clone(),
                fast_image_resize::PixelType::U8x4,
            ) {
                Ok(img) => img,
                Err(e) => {
                    warn!(
                        "[Resample] Progressive stage image creation failed: {}, returning original",
                        e
                    );
                    return DecodedImage {
                        width: current_width,
                        height: current_height,
                        original_width: orig_width,
                        original_height: orig_height,
                        pixels: current_pixels,
                        icc_profile,
                        exif,
                    };
                }
            };

            let mut dst_img = Image::new(next_w, next_h, fast_image_resize::PixelType::U8x4);

            // Use ResizeAlg::Nearest (which maps to optimized Box/Point in SIMD)
            if let Err(e) = resizer.resize(
                &src_img,
                &mut dst_img,
                Some(&ResizeOptions::new().resize_alg(ResizeAlg::Nearest)),
            ) {
                warn!(
                    "[Resample] Progressive stage resize failed: {}, returning intermediate",
                    e
                );
                return DecodedImage {
                    width: current_width,
                    height: current_height,
                    original_width: orig_width,
                    original_height: orig_height,
                    pixels: src_img.into_vec(),
                    icc_profile,
                    exif,
                };
            }

            current_pixels = dst_img.into_vec();
            current_width = next_w;
            current_height = next_h;
            loop_count += 1;
        }
        if loop_count > 0 {
            debug!(
                "[Resample] Progressive Box: {} steps -> {}x{}",
                loop_count, current_width, current_height
            );
        }
    }

    // 2. Halftone Detection (using current intermediate image)
    let halftone_score =
        crate::sampling::detect_halftone_score(&current_pixels, current_width, current_height);
    let is_halftone = halftone_score > 25.0;

    // 3. Conditional Pre-Gaussian Blur
    // Only applied if halftone patterns are detected to avoid blurring clean illustrations.
    let prefiltered_pixels = if is_halftone && scale < 0.7 {
        let blur_start = std::time::Instant::now();
        // Dynamic center weight: stronger blur for higher halftone scores.
        let center_weight = if halftone_score > 40.0 { 2 } else { 4 };

        let result = crate::sampling::preblur::pre_gaussian_rgba(
            &current_pixels,
            current_width,
            current_height,
            1, // Single pass is usually enough as a prefilter
            center_weight,
        );
        let blur_ms = blur_start.elapsed().as_secs_f32() * 1000.0;
        debug!(
            "[Resample] Prefilter Applied (score={:.1}): src={}x{} in {:.1}ms",
            halftone_score, current_width, current_height, blur_ms
        );
        result
    } else {
        current_pixels.clone()
    };

    // 4. Algorithm Switch (Mitchell for Halftone, Lanczos3 for Normal)
    let alg = if is_halftone {
        ResizeAlg::Convolution(fast_image_resize::FilterType::Mitchell)
    } else {
        ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3)
    };

    // Final resize to target_width, target_height
    let src_final = match Image::from_vec_u8(
        current_width,
        current_height,
        prefiltered_pixels,
        fast_image_resize::PixelType::U8x4,
    ) {
        Ok(img) => img,
        Err(e) => {
            warn!(
                "[Resample] Final stage image creation failed: {}, returning intermediate",
                e
            );
            return DecodedImage {
                width: current_width,
                height: current_height,
                original_width: orig_width,
                original_height: orig_height,
                pixels: current_pixels,
                icc_profile,
                exif,
            };
        }
    };

    let mut dst_final = Image::new(
        target_width,
        target_height,
        fast_image_resize::PixelType::U8x4,
    );

    // Final high-quality resize
    match resizer.resize(
        &src_final,
        &mut dst_final,
        Some(&ResizeOptions::new().resize_alg(alg)),
    ) {
        Ok(_) => {
            let total_ms = start_total.elapsed().as_secs_f32() * 1000.0;
            if total_ms > 10.0 {
                debug!(
                    "[Resample] Complete: {}x{} -> {}x{} in {:.1}ms ({} Mode, Score={:.1})",
                    orig_width,
                    orig_height,
                    target_width,
                    target_height,
                    total_ms,
                    if is_halftone { "Halftone" } else { "Clean" },
                    halftone_score
                );
            }
            DecodedImage {
                width: target_width,
                height: target_height,
                original_width: orig_width,
                original_height: orig_height,
                pixels: dst_final.into_vec(),
                icc_profile,
                exif,
            }
        }
        Err(e) => {
            warn!(
                "[Resample] Final resize failed: {}, returning intermediate",
                e
            );
            DecodedImage {
                width: current_width,
                height: current_height,
                original_width: orig_width,
                original_height: orig_height,
                pixels: src_final.into_vec(),
                icc_profile,
                exif,
            }
        }
    }
}
