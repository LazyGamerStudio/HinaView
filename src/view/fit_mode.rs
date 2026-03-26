// src/view/fit_mode.rs

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FitMode {
    FitScreen,
    #[allow(dead_code)]
    FitWidth,
    #[allow(dead_code)]
    FitHeight,
    #[allow(dead_code)]
    Fixed(f32),
}

impl Eq for FitMode {}
