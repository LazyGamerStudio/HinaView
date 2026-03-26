// src/layout/single_layout.rs
use super::{LayoutResult, PagePlacement};
use crate::document::Document;
use crate::view::ViewState;

/// Single-page layout.
///
/// WORLD space convention:
/// - +X: right, +Y: up, unit: logical pixels.
/// - `PagePlacement.position` is bottom-left corner of the page quad.
///
/// Layout policy:
/// - First page is **centered at (0, 0)**.
/// - Subsequent spreads are stacked **downwards** (negative Y) with a fixed gap.
pub fn compute(document: &Document, _view: &ViewState) -> LayoutResult {
    let mut placements = Vec::new();
    let gap = 20.0;

    // Position the first page centered at (0, 0).
    // `position` is the BOTTOM-LEFT corner, so for center at (0,0): position = (-w/2, -h/2).
    let mut current_y = 0.0;

    for spread in &document.spreads {
        if let Some(page_idx) = spread.left.or(spread.right)
            && let Some(meta) = document.pages.get(page_idx)
        {
            let w = meta.width as f32;
            let h = meta.height as f32;

            placements.push(PagePlacement {
                page_index: page_idx,
                // Center horizontally and vertically
                // First image center at (0, 0), subsequent images stack downward
                position: [-w / 2.0, current_y - h / 2.0],
                size: [w, h],
            });
            current_y -= h + gap;
        }
    }

    LayoutResult {
        placements,
        total_width: 0.0,
        total_height: current_y.abs(),
    }
}
