use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterParams {
    pub bypass_color: bool,
    pub bypass_median: bool,
    pub bypass_fsr: bool,
    pub bypass_detail: bool,
    pub bypass_levels: bool,
    pub bright: f32,
    pub contrast: f32,
    pub gamma: f32,
    pub exposure: f32,
    pub fsr_enabled: bool,
    pub fsr_sharpness: f32, // RCAS sharpness
    pub median_enabled: bool,
    pub median_strength: f32,
    pub median_stride: f32,
    pub blur_radius: f32,
    pub unsharp_amount: f32,
    pub unsharp_threshold: f32,
    pub levels_in_black: f32,
    pub levels_in_white: f32,
    pub levels_gamma: f32,
    pub levels_out_black: f32,
    pub levels_out_white: f32,
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            bypass_color: false,
            bypass_median: false,
            bypass_fsr: false,
            bypass_detail: false,
            bypass_levels: false,
            bright: 0.0,
            contrast: 1.0,
            gamma: 1.0,
            exposure: 0.0,
            fsr_enabled: false,
            fsr_sharpness: 0.0, // Default RCAS sharpness
            median_enabled: false,
            median_strength: 0.0,
            median_stride: 1.0,
            blur_radius: 0.0,
            unsharp_amount: 0.0,
            unsharp_threshold: 0.05,
            levels_in_black: 0.0,
            levels_in_white: 1.0,
            levels_gamma: 1.0,
            levels_out_black: 0.0,
            levels_out_white: 1.0,
        }
    }
}
