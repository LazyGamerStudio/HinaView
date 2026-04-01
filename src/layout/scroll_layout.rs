// src/layout/scroll_layout.rs
use super::{LayoutResult, PagePlacement};
use crate::document::Document;
use crate::view::ViewState;

pub fn compute(document: &Document, _view: &ViewState) -> LayoutResult {
    let mut placements = Vec::new();
    let gap = 0.0f32;
    let mut next_top_y: Option<f32> = None;
    let mut min_y = 0.0f32;
    let mut max_y = 0.0f32;
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
        let top_y = next_top_y.unwrap_or(h * 0.5);
        let bottom_y = top_y - h;

        placements.push(PagePlacement {
            page_index: page_idx,
            position: [-w * 0.5, bottom_y],
            size: [w, h],
        });

        min_y = min_y.min(bottom_y);
        max_y = max_y.max(top_y);
        next_top_y = Some(bottom_y - gap);
    }

    LayoutResult {
        placements,
        total_width: baseline_w,
        total_height: max_y - min_y,
    }
}
