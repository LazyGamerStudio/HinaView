use crate::view::NavigationController;

pub struct WebtoonScrollContext<'a> {
    pub nav: &'a mut NavigationController,
    pub holds: (bool, bool, bool, bool), // left, right, up, down
    pub scroll_speed: f32,
    pub dt_sec: f32,
    pub window_height: f32,
}

pub fn process_webtoon_scroll_controller(ctx: WebtoonScrollContext) -> bool {
    let zoom = ctx.nav.view.zoom.max(0.0001);
    let speed_px_per_sec = ctx.scroll_speed;
    let delta_world = (speed_px_per_sec * ctx.dt_sec) / zoom;
    let (left, right, up, down) = ctx.holds;

    let hold_x = match (left, right) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };
    let hold_y = match (up, down) {
        (true, false) => 1.0,
        (false, true) => -1.0,
        _ => 0.0,
    };

    if !matches!(
        ctx.nav.view.layout_mode,
        crate::types::LayoutMode::VerticalScroll
    ) {
        if hold_x == 0.0 && hold_y == 0.0 {
            return false;
        }
        ctx.nav.view.image_offset[0] += hold_x * delta_world;
        ctx.nav.view.image_offset[1] += hold_y * delta_world;
        ctx.nav.refresh_camera();
        return true;
    }

    let mut changed = false;
    let hold_dir = hold_y;

    if hold_dir != 0.0 {
        ctx.nav.webtoon_scroll_target_y = None;
        ctx.nav.view.image_offset[1] += hold_dir * delta_world;
        ctx.nav.refresh_camera();
        changed = true;
    }

    if hold_dir == 0.0
        && let Some(target) = ctx.nav.webtoon_scroll_target_y
    {
        let current = ctx.nav.view.image_offset[1];
        let diff = target - current;
        let epsilon = 0.5 / ctx.nav.view.zoom.max(0.0001);
        if diff.abs() <= epsilon {
            ctx.nav.view.image_offset[1] = target;
            ctx.nav.webtoon_scroll_target_y = None;
        } else {
            ctx.nav.view.image_offset[1] = current + diff * 0.24;
        }
        ctx.nav.refresh_camera();
        changed = true;
    }

    let page_changed = sync_current_page_for_webtoon(ctx.nav);
    let pan_y = ctx.nav.camera.pan.y;
    let threshold = (ctx.window_height / ctx.nav.view.zoom.max(0.0001)) * 0.1;
    let should_request = page_changed
        || changed
        || ctx
            .nav
            .webtoon_last_request_pan_y
            .map_or(true, |last| (pan_y - last).abs() >= threshold);

    if should_request {
        // This request logic should be handled by the caller or we should pass more state.
        // But for now, returning whether it changed is enough to trigger a refresh in App.
        ctx.nav.webtoon_last_request_pan_y = Some(pan_y);
    }

    changed || page_changed
}

pub fn queue_webtoon_scroll_page_delta_controller(
    nav: &mut NavigationController,
    delta: i32,
    is_pressed: bool,
) {
    if !is_pressed || delta == 0 {
        return;
    }
    if nav.webtoon_scroll_target_y.is_some() {
        return;
    }

    let Some(layout) = nav.layout.as_ref() else {
        return;
    };
    if layout.placements.is_empty() {
        return;
    }

    let mut ordered: Vec<usize> = layout.placements.iter().map(|p| p.page_index).collect();
    ordered.sort_unstable();
    ordered.dedup();
    if ordered.is_empty() {
        return;
    }

    let base_page = nav.current_page.or(nav.target_page).unwrap_or(ordered[0]);
    let current_idx = ordered.iter().position(|&p| p == base_page).unwrap_or(0);
    let next_idx = (current_idx as i32 + delta).clamp(0, ordered.len() as i32 - 1) as usize;
    let target_page = ordered[next_idx];
    queue_webtoon_scroll_to_page_controller(nav, target_page, true, true);
}

pub fn queue_webtoon_scroll_to_page_controller(
    nav: &mut NavigationController,
    page: usize,
    is_pressed: bool,
    honor_existing: bool,
) {
    if !is_pressed {
        return;
    }
    if honor_existing && nav.webtoon_scroll_target_y.is_some() {
        return;
    }
    let Some(layout) = nav.layout.as_ref() else {
        return;
    };
    let Some(placement) = layout.placements.iter().find(|p| p.page_index == page) else {
        return;
    };
    let target_center_y = placement.position[1] + placement.size[1] * 0.5;
    nav.webtoon_scroll_target_y = Some(target_center_y - nav.view.pan[1]);
    nav.target_page = Some(page);
}

fn sync_current_page_for_webtoon(nav: &mut NavigationController) -> bool {
    let Some(layout) = nav.layout.as_ref() else {
        return false;
    };
    if layout.placements.is_empty() {
        return false;
    }

    let cy = nav.camera.pan.y;
    let mut best_page = None;
    let mut best_dist = f32::MAX;
    for placement in &layout.placements {
        let center_y = placement.position[1] + placement.size[1] * 0.5;
        let dist = (center_y - cy).abs();
        if dist < best_dist {
            best_dist = dist;
            best_page = Some(placement.page_index);
        }
    }

    let Some(best_page) = best_page else {
        return false;
    };
    if nav.current_page == Some(best_page) {
        return false;
    }

    nav.current_page = Some(best_page);
    nav.pending_page = None;
    nav.target_page = Some(best_page);
    true
}
