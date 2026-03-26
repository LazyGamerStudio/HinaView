// src/view/view_state.rs
use super::fit_mode::FitMode;
use super::layout_mode::LayoutMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationQuarter {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl RotationQuarter {
    /// Rotates the orientation 90 degrees clockwise.
    pub fn rotate_cw(&mut self) {
        *self = match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        };
    }

    /// Rotates the orientation 90 degrees counter-clockwise.
    pub fn rotate_ccw(&mut self) {
        *self = match self {
            Self::Deg0 => Self::Deg270,
            Self::Deg90 => Self::Deg0,
            Self::Deg180 => Self::Deg90,
            Self::Deg270 => Self::Deg180,
        };
    }

    /// Returns true if the orientation results in transposed (swapped) width and height.
    pub fn is_transposed(self) -> bool {
        matches!(self, Self::Deg90 | Self::Deg270)
    }
}

pub struct ViewState {
    pub zoom: f32,
    pub pan: [f32; 2],
    pub image_offset: [f32; 2],
    pub layout_mode: LayoutMode,
    pub rotation: RotationQuarter,
    #[allow(dead_code)]
    pub fit_mode: FitMode,
}
