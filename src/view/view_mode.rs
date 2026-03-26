// src/view/view_mode.rs

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    FitScreen,
    #[allow(dead_code)]
    FitWidth,
    #[allow(dead_code)]
    FitHeight,
    #[allow(dead_code)]
    Fixed(f32),
}

impl Eq for ViewMode {}
