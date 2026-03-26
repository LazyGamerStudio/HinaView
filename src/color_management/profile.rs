// src/color_management/profile.rs

use super::lcms2_ffi::*;
use std::os::raw::c_void;
use tracing::warn;

#[derive(Clone, Debug)]
pub struct ColorProfile {
    pub name: String,
    /// Matrix to convert profile RGB to XYZ PCS (D50)
    pub to_xyz_matrix: [[f32; 3]; 3],
    pub gamma: f32,
}

impl ColorProfile {
    pub fn srgb() -> Self {
        Self {
            name: "sRGB".to_string(),
            // sRGB D50 Matrix (from brucelindbloom.com)
            to_xyz_matrix: [
                [0.4360747, 0.3850649, 0.1430804],
                [0.2225045, 0.7168786, 0.0606169],
                [0.0139291, 0.0970870, 0.7141736],
            ],
            gamma: 2.2,
        }
    }

    pub fn from_name(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut profile = Self::srgb();
        profile.name = name;
        profile.gamma = infer_gamma(&profile.name);
        profile
    }

    pub fn from_icc(icc_data: &[u8], name: Option<String>) -> Option<Self> {
        if icc_data.is_empty() {
            return None;
        }

        unsafe {
            // SAFETY: `icc_data` is a stable in-memory byte slice for the duration of this call,
            // and LittleCMS only reads from the provided buffer.
            let h_profile =
                cmsOpenProfileFromMem(icc_data.as_ptr() as *const c_void, icc_data.len() as u32);
            if h_profile.is_null() {
                return None;
            }

            let result = Self::from_h_profile(h_profile, name);
            // SAFETY: `h_profile` was created by cmsOpenProfileFromMem above and is still valid.
            cmsCloseProfile(h_profile);
            result
        }
    }

    pub unsafe fn from_h_profile(h_profile: cmsHPROFILE, name: Option<String>) -> Option<Self> {
        let mut to_xyz = [[0.0f32; 3]; 3];

        // Try to read red, green, blue colorants (XYZ matrix)
        let (r, g, b) = unsafe {
            // SAFETY: `h_profile` is expected to be a valid LittleCMS profile handle supplied
            // by the caller, and cmsReadTag returns borrowed pointers owned by that profile.
            (
                cmsReadTag(h_profile, cmsSigRedColorantTag) as *mut cmsCIEXYZ,
                cmsReadTag(h_profile, cmsSigGreenColorantTag) as *mut cmsCIEXYZ,
                cmsReadTag(h_profile, cmsSigBlueColorantTag) as *mut cmsCIEXYZ,
            )
        };

        if !r.is_null() && !g.is_null() && !b.is_null() {
            unsafe {
                // SAFETY: The colorant pointers were checked for null above and point to
                // cmsCIEXYZ values owned by the live profile handle.
                to_xyz[0][0] = (*r).X as f32;
                to_xyz[1][0] = (*r).Y as f32;
                to_xyz[2][0] = (*r).Z as f32;

                to_xyz[0][1] = (*g).X as f32;
                to_xyz[1][1] = (*g).Y as f32;
                to_xyz[2][1] = (*g).Z as f32;

                to_xyz[0][2] = (*b).X as f32;
                to_xyz[1][2] = (*b).Y as f32;
                to_xyz[2][2] = (*b).Z as f32;
            }
        } else {
            // Fallback for non-matrix profiles (simplified)
            warn!("ICC profile does not contain RGB colorant tags, using sRGB matrix");
            to_xyz = Self::srgb().to_xyz_matrix;
        }

        // Estimate gamma from Red TRC
        let mut gamma = 2.2;
        // SAFETY: The tag pointer is borrowed from the valid profile handle.
        let r_trc = unsafe { cmsReadTag(h_profile, cmsSigRedTRCTag) as cmsToneCurve };
        if !r_trc.is_null() {
            // SAFETY: `r_trc` is a non-null tone curve pointer returned by cmsReadTag above.
            gamma = unsafe { cmsEstimateGamma(r_trc, 0.01) as f32 };
            if gamma < 0.1 || gamma > 5.0 {
                gamma = 2.2;
            }
        }

        Some(Self {
            name: name.unwrap_or_else(|| "Unknown".to_string()),
            to_xyz_matrix: to_xyz,
            gamma,
        })
    }

    /// Calculate a matrix that converts from `self` to `target`.
    pub fn calculate_conversion_matrix(&self, target: &ColorProfile) -> [[f32; 3]; 3] {
        // [M_total] = [M_target]^-1 * [M_self]
        let m_self = self.to_xyz_matrix;
        let m_target_inv = invert_matrix(target.to_xyz_matrix).unwrap_or(identity_matrix());

        multiply_matrices(m_target_inv, m_self)
    }
}

pub fn infer_gamma(name: &str) -> f32 {
    let n = name.to_ascii_lowercase();
    if n.contains("adobe rgb") || n.contains("display p3") || n.contains("p3") {
        2.2
    } else if n.contains("rec.709") || n.contains("bt.709") {
        2.4
    } else {
        2.2
    }
}

fn identity_matrix() -> [[f32; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn multiply_matrices(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut res = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            res[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    res
}

fn invert_matrix(m: [[f32; 3]; 3]) -> Option<[[f32; 3]; 3]> {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);

    if det.abs() < 1e-6 {
        return None;
    }

    let inv_det = 1.0 / det;
    let mut res = [[0.0; 3]; 3];

    res[0][0] = (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det;
    res[0][1] = (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det;
    res[0][2] = (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det;

    res[1][0] = (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det;
    res[1][1] = (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det;
    res[1][2] = (m[1][0] * m[0][2] - m[0][0] * m[1][2]) * inv_det;

    res[2][0] = (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det;
    res[2][1] = (m[2][0] * m[0][1] - m[0][0] * m[2][1]) * inv_det;
    res[2][2] = (m[0][0] * m[1][1] - m[1][0] * m[0][1]) * inv_det;

    Some(res)
}
