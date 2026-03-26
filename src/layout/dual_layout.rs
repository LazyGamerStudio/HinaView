// src/layout/dual_layout.rs
use super::{LayoutResult, PagePlacement};
use crate::document::Document;
use crate::view::ViewState;

pub fn compute(document: &Document, _view: &ViewState) -> LayoutResult {
    let mut placements = Vec::new();
    let mut current_y = 0.0;
    let mut total_width: f32 = 0.0;
    let gap = 40.0;

    for spread in &document.spreads {
        let mut max_h: f32 = 0.0;
        let left_size = spread
            .left
            .and_then(|idx| document.pages.get(idx))
            .map(|m| (m.width as f32, m.height as f32));
        let right_size = spread
            .right
            .and_then(|idx| document.pages.get(idx))
            .map(|m| (m.width as f32, m.height as f32));
        let left_is_wide_or_animated = spread
            .left
            .and_then(|idx| document.pages.get(idx))
            .map(|m| m.is_wide || m.is_animated)
            .unwrap_or(false);
        let right_is_wide_or_animated = spread
            .right
            .and_then(|idx| document.pages.get(idx))
            .map(|m| m.is_wide || m.is_animated)
            .unwrap_or(false);

        let target_h = left_size
            .map(|(_, h)| h)
            .unwrap_or(0.0)
            .max(right_size.map(|(_, h)| h).unwrap_or(0.0));

        let left_scaled = left_size.map(|(w, h)| {
            if h > 0.0 && target_h > 0.0 {
                let scale = target_h / h;
                (w * scale, target_h)
            } else {
                (w, h)
            }
        });
        let right_scaled = right_size.map(|(w, h)| {
            if h > 0.0 && target_h > 0.0 {
                let scale = target_h / h;
                (w * scale, target_h)
            } else {
                (w, h)
            }
        });

        // Single wide/animated page spread should be centered, not slotted as x|1/1|x.
        let single_page = spread.left.is_some() ^ spread.right.is_some();
        let single_should_center =
            single_page && (left_is_wide_or_animated || right_is_wide_or_animated);

        if single_should_center {
            if let Some(left_idx) = spread.left
                && let Some((w, h)) = left_scaled
            {
                placements.push(PagePlacement {
                    page_index: left_idx,
                    position: [-w * 0.5, current_y],
                    size: [w, h],
                });
                max_h = max_h.max(h);
                total_width = total_width.max(w);
            }
            if let Some(right_idx) = spread.right
                && let Some((w, h)) = right_scaled
            {
                placements.push(PagePlacement {
                    page_index: right_idx,
                    position: [-w * 0.5, current_y],
                    size: [w, h],
                });
                max_h = max_h.max(h);
                total_width = total_width.max(w);
            }

            current_y += max_h + gap;
            continue;
        }

        // Keep virtual slot width when one side is empty so 1|x / x|1 remains visible.
        let left_slot_w = left_scaled
            .map(|(w, _)| w)
            .or_else(|| right_scaled.map(|(w, _)| w))
            .unwrap_or(0.0);
        let right_slot_w = right_scaled
            .map(|(w, _)| w)
            .or_else(|| left_scaled.map(|(w, _)| w))
            .unwrap_or(0.0);
        let spread_width = left_slot_w + right_slot_w;
        let left_slot_x = -spread_width * 0.5;
        let right_slot_x = left_slot_x + left_slot_w;

        // Place left page
        if let Some(left_idx) = spread.left
            && let Some((w, h)) = left_scaled
        {
            placements.push(PagePlacement {
                page_index: left_idx,
                position: [left_slot_x, current_y],
                size: [w, h],
            });
            max_h = max_h.max(h);
        }

        // Place right page
        if let Some(right_idx) = spread.right
            && let Some((w, h)) = right_scaled
        {
            placements.push(PagePlacement {
                page_index: right_idx,
                position: [right_slot_x, current_y],
                size: [w, h],
            });
            max_h = max_h.max(h);
        }

        current_y += max_h + gap;
        total_width = total_width.max(spread_width);
    }

    LayoutResult {
        placements,
        total_width,
        total_height: current_y,
    }
}
