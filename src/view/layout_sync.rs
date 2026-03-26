use crate::camera::Camera;
use crate::document::Document;
use crate::layout::{LayoutResult, compute_layout};
use crate::types::LayoutMode;
use crate::types::PageId;
use crate::view::{FitMode, RotationQuarter, ViewState};

fn maybe_swap_dims(
    dims: Option<(f32, f32)>,
    rotation: RotationQuarter,
    allow_rotation: bool,
) -> Option<(f32, f32)> {
    dims.map(|(w, h)| {
        if allow_rotation && rotation.is_transposed() {
            (h, w)
        } else {
            (w, h)
        }
    })
}

pub fn center_camera_on_page(
    layout: Option<&LayoutResult>,
    view: &mut ViewState,
    camera: &mut Camera,
    page: PageId,
) {
    let layout = match layout {
        Some(layout) => layout,
        None => return,
    };

    if let Some(placement) = layout.placements.iter().find(|p| p.page_index == page) {
        if matches!(view.layout_mode, crate::types::LayoutMode::Dual { .. }) {
            let y = placement.position[1];
            if let Some(partner) = layout
                .placements
                .iter()
                .find(|p| p.page_index != page && (p.position[1] - y).abs() < 0.1)
            {
                let min_x = placement.position[0].min(partner.position[0]);
                let min_y = placement.position[1].min(partner.position[1]);
                let max_x = (placement.position[0] + placement.size[0])
                    .max(partner.position[0] + partner.size[0]);
                let max_y = (placement.position[1] + placement.size[1])
                    .max(partner.position[1] + partner.size[1]);
                let center_x = (min_x + max_x) * 0.5;
                let center_y = (min_y + max_y) * 0.5;
                view.pan = [center_x, center_y];
                refresh_camera(view, camera, Some(layout), Some(page));
                return;
            }

            // Single-page spread in dual mode (1|x or x|1):
            // center camera to spread center, not page center.
            let page_center_x = placement.size[0].mul_add(0.5, placement.position[0]);
            let centered_single = page_center_x.abs() < 0.1;
            let center_x = if centered_single {
                page_center_x
            } else if placement.position[0] >= 0.0 {
                placement.position[0]
            } else {
                placement.position[0] + placement.size[0]
            };
            let center_y = placement.size[1].mul_add(0.5, placement.position[1]);
            view.pan = [center_x, center_y];
            refresh_camera(view, camera, Some(layout), Some(page));
            return;
        }

        let center_x = placement.size[0].mul_add(0.5, placement.position[0]);
        let center_y = placement.size[1].mul_add(0.5, placement.position[1]);
        view.pan = [center_x, center_y];
        refresh_camera(view, camera, Some(layout), Some(page));
    }
}

fn target_bounds_for_mode(
    document: &Document,
    page: PageId,
    layout_mode: LayoutMode,
    rotation: RotationQuarter,
) -> Option<(f32, f32)> {
    match layout_mode {
        LayoutMode::VerticalScroll => {
            let baseline_w = document
                .pages
                .iter()
                .find_map(|p| (p.width > 0).then_some(p.width as f32))
                .unwrap_or(1.0);
            let page_meta = document.pages.get(page)?;
            if page_meta.width == 0 || page_meta.height == 0 {
                return None;
            }
            let src_w = page_meta.width as f32;
            let src_h = page_meta.height as f32;
            let scale = baseline_w / src_w.max(1.0);
            Some((baseline_w, src_h * scale))
        }
        LayoutMode::Dual { .. } => {
            let spread = document
                .spreads
                .iter()
                .find(|s| s.left == Some(page) || s.right == Some(page))?;
            let single_page_spread = spread.left.is_some() ^ spread.right.is_some();

            let left = spread
                .left
                .and_then(|idx| document.pages.get(idx))
                .map(|m| (m.width as f32, m.height as f32));
            let right = spread
                .right
                .and_then(|idx| document.pages.get(idx))
                .map(|m| (m.width as f32, m.height as f32));

            // Rotation is disabled in dual mode.
            let left = maybe_swap_dims(left, rotation, false);
            let right = maybe_swap_dims(right, rotation, false);

            let target_h = left
                .map(|(_, h)| h)
                .unwrap_or(0.0)
                .max(right.map(|(_, h)| h).unwrap_or(0.0));
            if target_h <= 0.0 {
                return None;
            }

            let left_scaled_w = left
                .map(|(w, h)| if h > 0.0 { w * (target_h / h) } else { w })
                .unwrap_or(0.0);
            let right_scaled_w = right
                .map(|(w, h)| if h > 0.0 { w * (target_h / h) } else { w })
                .unwrap_or(0.0);

            if single_page_spread {
                let only_idx = spread.left.or(spread.right)?;
                let only_meta = document.pages.get(only_idx)?;
                if only_meta.is_wide || only_meta.is_animated {
                    // Wide/animated single spread is intentionally centered as a single page.
                    // Do not apply virtual empty-slot width here.
                    let (pw, ph) = maybe_swap_dims(
                        Some((only_meta.width as f32, only_meta.height as f32)),
                        rotation,
                        false,
                    )?;
                    if pw > 0.0 && ph > 0.0 {
                        return Some((pw, ph));
                    }
                }
            }

            // Keep virtual width for empty slot (x|1 / 1|x).
            let left_slot_w = if left_scaled_w > 0.0 {
                left_scaled_w
            } else {
                right_scaled_w
            };
            let right_slot_w = if right_scaled_w > 0.0 {
                right_scaled_w
            } else {
                left_scaled_w
            };

            Some((left_slot_w + right_slot_w, target_h))
        }
        _ => {
            let page_meta = document.pages.get(page)?;
            if page_meta.width == 0 || page_meta.height == 0 {
                return None;
            }
            maybe_swap_dims(
                Some((page_meta.width as f32, page_meta.height as f32)),
                rotation,
                true,
            )
        }
    }
}

pub fn calculate_target_zoom(
    document: Option<&Document>,
    fit_mode: FitMode,
    layout_mode: LayoutMode,
    rotation: RotationQuarter,
    page: PageId,
    window_size: (u32, u32),
) -> f32 {
    if let FitMode::Fixed(zoom) = fit_mode {
        return zoom;
    }

    if let Some(doc) = document
        && let Some((pw, ph)) = target_bounds_for_mode(doc, page, layout_mode, rotation)
    {
        let (ww, wh) = (window_size.0 as f32, window_size.1 as f32);

        return match fit_mode {
            FitMode::FitWidth => ww / pw,
            FitMode::FitHeight => wh / ph,
            FitMode::FitScreen => {
                let img_aspect = pw / ph;
                let screen_aspect = ww / wh;
                if img_aspect > screen_aspect {
                    ww / pw
                } else {
                    wh / ph
                }
            }
            FitMode::Fixed(z) => z,
        };
    }

    1.0
}

pub fn update_zoom_for_current_page(
    document: Option<&Document>,
    current_page: Option<PageId>,
    view: &mut ViewState,
    camera: &mut Camera,
    window_size: (u32, u32),
) {
    if let Some(doc) = document
        && !doc.pages.is_empty()
    {
        let fit_target_index = current_page.unwrap_or(0);
        let zoom = calculate_target_zoom(
            Some(doc),
            view.fit_mode,
            view.layout_mode,
            view.rotation,
            fit_target_index,
            window_size,
        );

        view.zoom = zoom;
        camera.zoom = zoom;
    }
}

pub fn rebuild_layout(document: &mut Document, view: &ViewState) -> LayoutResult {
    document.rebuild_spreads(view.layout_mode);
    compute_layout(document, view)
}

pub fn refresh_camera(
    view: &mut ViewState,
    camera: &mut Camera,
    layout: Option<&LayoutResult>,
    current_page: Option<PageId>,
) {
    camera.zoom = view.zoom;

    if let (Some(layout), Some(current_page)) = (layout, current_page) {
        if matches!(view.layout_mode, LayoutMode::VerticalScroll) {
            view.image_offset[0] = 0.0;
            // Webtoon mode: keep horizontal center fixed, but do not clamp Y.
            camera.pan = glam::Vec2::new(view.pan[0], view.pan[1] + view.image_offset[1]);
            return;
        }
        if let Some(placement) = layout
            .placements
            .iter()
            .find(|p| p.page_index == current_page)
        {
            let limit_x = placement.size[0] * 0.5;
            let limit_y = placement.size[1] * 0.5;
            let (limit_x, limit_y) = if matches!(view.layout_mode, LayoutMode::Dual { .. }) {
                (limit_x, limit_y)
            } else if view.rotation.is_transposed() {
                (limit_y, limit_x)
            } else {
                (limit_x, limit_y)
            };

            // Clamp the actual stored offset values
            view.image_offset[0] = view.image_offset[0].clamp(-limit_x, limit_x);
            view.image_offset[1] = view.image_offset[1].clamp(-limit_y, limit_y);
        }
    }

    camera.pan = glam::Vec2::new(
        view.pan[0] + view.image_offset[0],
        view.pan[1] + view.image_offset[1],
    );
}
