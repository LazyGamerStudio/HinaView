// src/sampling/halftone.rs
use fast_image_resize::images::Image;
use fast_image_resize::{ResizeAlg, Resizer};

/// Detects if an image has halftone patterns (common in manga scans).
/// Returns a score where higher values indicate more halftone.
/// threshold ≈ 25 is recommended for detection.
pub fn detect_halftone_score(pixels: &[u8], width: u32, height: u32) -> f32 {
    if width < 128 || height < 128 {
        return 0.0;
    }

    // 1. Create a 128x128 thumbnail for fast analysis
    let Ok(src_img) = Image::from_vec_u8(
        width,
        height,
        pixels.to_vec(),
        fast_image_resize::PixelType::U8x4,
    ) else {
        return 0.0;
    };

    let mut dst_img = Image::new(128, 128, fast_image_resize::PixelType::U8x4);
    let mut resizer = Resizer::new();

    // Use Nearest for maximum speed for this internal analysis
    if resizer
        .resize(
            &src_img,
            &mut dst_img,
            Some(&fast_image_resize::ResizeOptions::new().resize_alg(ResizeAlg::Nearest)),
        )
        .is_err()
    {
        return 0.0;
    }

    let thumb_pixels = dst_img.buffer();

    // 2. Grayscale (Luma) conversion and Laplacian
    // Using 126x126 inner area to avoid boundary checks
    let mut laplacian_values = Vec::with_capacity(126 * 126);
    let mut sum: f32 = 0.0;

    for y in 1..127 {
        for x in 1..127 {
            // Laplacian Kernel:
            // -1 -1 -1
            // -1  8 -1
            // -1 -1 -1

            let center_idx = (y * 128 + x) * 4;
            let luma = |idx: usize| {
                let r = thumb_pixels[idx] as f32;
                let g = thumb_pixels[idx + 1] as f32;
                let b = thumb_pixels[idx + 2] as f32;
                0.299 * r + 0.587 * g + 0.114 * b
            };

            let val = 8.0 * luma(center_idx)
                - luma(((y - 1) * 128 + (x - 1)) * 4)
                - luma(((y - 1) * 128 + x) * 4)
                - luma(((y - 1) * 128 + (x + 1)) * 4)
                - luma((y * 128 + (x - 1)) * 4)
                - luma((y * 128 + (x + 1)) * 4)
                - luma(((y + 1) * 128 + (x - 1)) * 4)
                - luma(((y + 1) * 128 + x) * 4)
                - luma(((y + 1) * 128 + (x + 1)) * 4);

            laplacian_values.push(val);
            sum += val;
        }
    }

    let mean = sum / (laplacian_values.len() as f32);
    let mut variance_sum = 0.0;
    for &v in &laplacian_values {
        let diff = v - mean;
        variance_sum += diff * diff;
    }

    let variance = variance_sum / (laplacian_values.len() as f32);

    // Map variance to a more readable score (log scale often helps with variance)
    // For Laplacian variance, 1000~5000 is common for edges, higher for textures/halftones.
    // We normalize this to the requested 0~100 scale approximately.
    (variance / 100.0).sqrt()
}
