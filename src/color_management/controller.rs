// src/color_management/controller.rs

use super::display_profile::detect_display_profile;
use super::profile::ColorProfile;

pub struct ColorManagementController {
    display: ColorProfile,
}

impl ColorManagementController {
    pub fn new() -> Self {
        // 1. Try automatic detection
        let detected = detect_display_profile();

        // 2. Fallback to name-based or sRGB
        let display = if let Some((name, data)) = detected {
            ColorProfile::from_icc(&data, Some(name)).unwrap_or_else(ColorProfile::srgb)
        } else {
            // Try environment variable or default
            let env_profile = std::env::var("HINAVIEW_DISPLAY_PROFILE")
                .ok()
                .filter(|v| !v.trim().is_empty());

            let display_name = env_profile.unwrap_or_else(|| "sRGB IEC61966-2.1".to_string());
            ColorProfile::from_name(display_name)
        };

        Self { display }
    }

    pub fn display_profile_name(&self) -> &str {
        &self.display.name
    }

    #[allow(dead_code)]
    pub fn display_profile(&self) -> &ColorProfile {
        &self.display
    }

    /// Returns (conversion_matrix, icc_gamma) based on source profile name
    pub fn get_params_for_source_name(&self, source_name: Option<&str>) -> ([[f32; 4]; 3], f32) {
        let source_profile = source_name.map(ColorProfile::from_name);
        self.get_params_for_source(source_profile.as_ref())
    }

    /// Returns (conversion_matrix, icc_gamma)
    pub fn get_params_for_source(
        &self,
        source_profile: Option<&ColorProfile>,
    ) -> ([[f32; 4]; 3], f32) {
        let (matrix, src_gamma) = if let Some(src) = source_profile {
            (src.calculate_conversion_matrix(&self.display), src.gamma)
        } else {
            // Assume sRGB source if no profile
            let srgb = ColorProfile::srgb();
            (srgb.calculate_conversion_matrix(&self.display), 2.2)
        };

        // Convert 3x3 to [[f32; 4]; 3] for WGSL alignment (16-byte columns)
        let mut aligned_matrix = [[0.0; 4]; 3];
        for i in 0..3 {
            for j in 0..3 {
                aligned_matrix[i][j] = matrix[i][j];
            }
        }

        let icc_gamma = (src_gamma / self.display.gamma).clamp(0.1, 10.0);

        (aligned_matrix, icc_gamma)
    }

    pub fn gamma_correction_for_source(&self, source_profile_name: Option<&str>) -> f32 {
        let src_gamma = source_profile_name
            .map(ColorProfile::from_name)
            .map(|p| p.gamma)
            .unwrap_or(2.2);
        (src_gamma / self.display.gamma).clamp(0.1, 10.0)
    }
}
