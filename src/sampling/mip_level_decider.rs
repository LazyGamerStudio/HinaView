// src/sampling/mip_level_decider.rs
use crate::types::MipLevel;

/// Decides the appropriate MipLevel based on zoom level and animation status.
///
/// # Strategy
/// - If animated, always use Full to ensure frame stream stability.
/// - Otherwise, lower zoom = lower resolution needed.
///
/// # Arguments
/// * `target_zoom` - Current zoom level (1.0 = 100%)
/// * `is_animated` - Whether the image is animated (GIF, WebP)
///
/// # Returns
/// Optimal MipLevel for the current context.
pub fn decide_mip_level(target_zoom: f32, is_animated: bool) -> MipLevel {
    if is_animated {
        return MipLevel::Full;
    }

    // Map target_zoom to the closest n/8 MipLevel.
    // We use thresholds roughly in the middle of each n/8 step to find the best fit.
    match target_zoom {
        z if z < 0.1875 => MipLevel::Eighth,        // < 1.5/8 → 1/8
        z if z < 0.3125 => MipLevel::Quarter,       // < 2.5/8 → 2/8
        z if z < 0.4375 => MipLevel::ThreeEighths,  // < 3.5/8 → 3/8
        z if z < 0.5625 => MipLevel::Half,          // < 4.5/8 → 4/8
        z if z < 0.6875 => MipLevel::FiveEighths,   // < 5.5/8 → 5/8
        z if z < 0.8125 => MipLevel::ThreeQuarters, // < 6.5/8 → 6/8
        z if z < 0.9375 => MipLevel::SevenEighths,  // < 7.5/8 → 7/8
        _ => MipLevel::Full,                        // >= 7.5/8 → 8/8
    }
}
