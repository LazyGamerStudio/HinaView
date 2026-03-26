use crate::cache::TextureManager;
use crate::pipeline::DecodeScheduler;
use crate::types::PageId;
use crate::view::NavigationController;
use crate::view::animation_controller::AnimationController;

fn is_spread_ready(
    nav: &NavigationController,
    texture_manager: &TextureManager,
    page_id: PageId,
) -> bool {
    match nav.view.layout_mode {
        crate::types::LayoutMode::Dual { .. } => {
            if let Some(doc) = &nav.document {
                if let Some(spread) = doc
                    .spreads
                    .iter()
                    .find(|s| s.left == Some(page_id) || s.right == Some(page_id))
                {
                    let left_ready = spread.left.map_or(true, |id| texture_manager.has_page(id));
                    let right_ready = spread.right.map_or(true, |id| texture_manager.has_page(id));
                    return left_ready && right_ready;
                }
            }
            texture_manager.has_page(page_id)
        }
        _ => texture_manager.has_page(page_id),
    }
}

pub fn finalize_navigation_state_controller(
    nav: &mut NavigationController,
    texture_manager: &mut TextureManager,
    _scheduler: &mut DecodeScheduler,
    window_size: (u32, u32),
    animation_controller: &mut AnimationController,
) -> bool {
    // 1. Chasing Commit: If we have a pending target, try to advance current_page
    // to the furthest ready page in the direction of pending_page.
    if let (Some(current_id), Some(pending_id)) = (nav.current_page, nav.pending_page) {
        if current_id != pending_id {
            let mut furthest_ready = None;
            let (start, end) = if current_id < pending_id {
                (current_id + 1, pending_id)
            } else {
                (pending_id, current_id.saturating_sub(1))
            };

            // Identify the best available intermediate page already in GPU cache.
            // In Dual mode, we MUST ensure the entire spread is ready before committing.
            for &page_id in texture_manager.textures.keys() {
                if page_id >= start
                    && page_id <= end
                    && is_spread_ready(nav, texture_manager, page_id)
                {
                    match furthest_ready {
                        None => furthest_ready = Some(page_id),
                        Some(prev) => {
                            if current_id < pending_id {
                                if page_id > prev {
                                    furthest_ready = Some(page_id);
                                }
                            } else {
                                if page_id < prev {
                                    furthest_ready = Some(page_id);
                                }
                            }
                        }
                    }
                }
            }

            if let Some(ready_id) = furthest_ready {
                nav.current_page = Some(ready_id);
                if ready_id == pending_id {
                    nav.pending_page = None;
                }

                nav.update_zoom_for_current_page(window_size);
                nav.center_camera_on_page(ready_id);
                nav.prefetch_after_first_present = true;

                let visible = nav.get_visible_pages(window_size);
                animation_controller.retain_visible(&visible);
                return true;
            }
        }
    }

    // 2. Exact Match Commit: Fallback for initial load or single-step jumps.
    if let Some(pending_id) = nav.pending_page
        && is_spread_ready(nav, texture_manager, pending_id)
    {
        nav.current_page = Some(pending_id);
        nav.pending_page = None;

        nav.update_zoom_for_current_page(window_size);
        nav.center_camera_on_page(pending_id);
        nav.prefetch_after_first_present = true;

        let visible = nav.get_visible_pages(window_size);
        animation_controller.retain_visible(&visible);
        return true;
    }

    false
}
