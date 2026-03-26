use crate::types::PageId;

pub fn fast_nav_start_page(pending_page: Option<PageId>, current_page: PageId) -> PageId {
    pending_page.unwrap_or(current_page)
}

pub fn should_enter_fast_navigation(is_repeat: bool, is_idle: bool) -> bool {
    is_repeat && is_idle
}

pub fn clamp_target_page(current_page: PageId, delta: i32, total_pages: usize) -> PageId {
    let max_idx = total_pages.saturating_sub(1) as i32;
    (current_page as i32 + delta).clamp(0, max_idx) as PageId
}
