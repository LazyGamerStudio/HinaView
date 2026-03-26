// src/layout/mod.rs
pub mod dual_layout;
pub mod placement;
pub mod scroll_layout;
pub mod single_layout;

use crate::document::Document;
use crate::view::{LayoutMode, ViewState};
pub use placement::PagePlacement;

pub struct LayoutResult {
    pub placements: Vec<PagePlacement>,
    #[allow(dead_code)]
    pub total_width: f32,
    #[allow(dead_code)]
    pub total_height: f32,
}

/// Computes the layout placements for all pages based on the current document and view state.
pub fn compute_layout(document: &Document, view: &ViewState) -> LayoutResult {
    match view.layout_mode {
        LayoutMode::Single => single_layout::compute(document, view),
        LayoutMode::Dual { .. } => dual_layout::compute(document, view),
        LayoutMode::VerticalScroll => scroll_layout::compute(document, view),
    }
}
