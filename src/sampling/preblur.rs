/// Apply a lightweight separable gaussian-like blur on RGBA8 pixels.
/// This is used as a prefilter before mip downscaling to reduce aliasing.
pub fn pre_gaussian_rgba(
    pixels: &[u8],
    width: u32,
    height: u32,
    passes: usize,
    center_weight: u32,
) -> Vec<u8> {
    if passes == 0 || width == 0 || height == 0 {
        return pixels.to_vec();
    }
    let len = pixels.len();
    if len != (width as usize) * (height as usize) * 4 {
        return pixels.to_vec();
    }

    let mut src = pixels.to_vec();
    let mut tmp = vec![0u8; len];
    let mut dst = vec![0u8; len];

    let total_weight = center_weight + 2;

    for _ in 0..passes {
        horizontal_weighted(
            &src,
            &mut tmp,
            width as usize,
            height as usize,
            center_weight,
            total_weight,
        );
        vertical_weighted(
            &tmp,
            &mut dst,
            width as usize,
            height as usize,
            center_weight,
            total_weight,
        );
        std::mem::swap(&mut src, &mut dst);
    }

    src
}

fn horizontal_weighted(
    src: &[u8],
    dst: &mut [u8],
    width: usize,
    height: usize,
    center_weight: u32,
    total_weight: u32,
) {
    for y in 0..height {
        for x in 0..width {
            let xl = x.saturating_sub(1);
            let xr = (x + 1).min(width - 1);
            let base = (y * width + x) * 4;
            let l = (y * width + xl) * 4;
            let r = (y * width + xr) * 4;

            for c in 0..4 {
                let v =
                    src[l + c] as u32 + (src[base + c] as u32) * center_weight + src[r + c] as u32;
                dst[base + c] = (v / total_weight) as u8;
            }
        }
    }
}

fn vertical_weighted(
    src: &[u8],
    dst: &mut [u8],
    width: usize,
    height: usize,
    center_weight: u32,
    total_weight: u32,
) {
    for y in 0..height {
        let yu = y.saturating_sub(1);
        let yd = (y + 1).min(height - 1);
        for x in 0..width {
            let base = (y * width + x) * 4;
            let u = (yu * width + x) * 4;
            let d = (yd * width + x) * 4;
            for c in 0..4 {
                let v =
                    src[u + c] as u32 + (src[base + c] as u32) * center_weight + src[d + c] as u32;
                dst[base + c] = (v / total_weight) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pre_gaussian_rgba;

    #[test]
    fn preblur_preserves_length() {
        let width = 8;
        let height = 4;
        let pixels = vec![128u8; width * height * 4];
        let out = pre_gaussian_rgba(&pixels, width as u32, height as u32, 2, 2);
        assert_eq!(out.len(), pixels.len());
    }

    #[test]
    fn preblur_zero_pass_returns_copy() {
        let pixels = vec![1u8, 2, 3, 4];
        let out = pre_gaussian_rgba(&pixels, 1, 1, 0, 2);
        assert_eq!(out, pixels);
    }
}
