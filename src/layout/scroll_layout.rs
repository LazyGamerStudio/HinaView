// src/layout/scroll_layout.rs
use super::{LayoutResult, PagePlacement};
use crate::document::Document;
use crate::view::ViewState;

pub fn compute(document: &Document, _view: &ViewState) -> LayoutResult {
    let mut placements = Vec::new();
    let gap = 0.0f32;
    let mut current_y = 0.0f32;
    let baseline_w = document
        .pages
        .iter()
        .find_map(|p| (p.width > 0).then_some(p.width as f32))
        .unwrap_or(1.0);

    // Webtoon mode: page-direct vertical stack (ignore spreads), center X, gap=0.
    for (page_idx, meta) in document.pages.iter().enumerate() {
        let src_w = (meta.width as f32).max(1.0);
        let src_h = (meta.height as f32).max(1.0);
        let scale = baseline_w / src_w;
        let w = baseline_w;
        let h = src_h * scale;
        placements.push(PagePlacement {
            page_index: page_idx,
            position: [-w * 0.5, current_y - h * 0.5],
            size: [w, h],
        });
        current_y -= h + gap;
    }

    LayoutResult {
        placements,
        total_width: 0.0,
        total_height: current_y.abs(),
    }
}
