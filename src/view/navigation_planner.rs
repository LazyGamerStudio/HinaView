use crate::cache::{PrefetchDirection, compute_prefetch_pages};
use crate::pipeline::JobPriority;
use crate::types::PageId;
use crate::view::navigation_types::NavigationDirection;

pub fn prefetch_plan(
    current_page: PageId,
    direction: NavigationDirection,
    total_pages: usize,
    prefetch_count: usize,
) -> Vec<(PageId, JobPriority)> {
    let prefetch_dir = match direction {
        NavigationDirection::Next => PrefetchDirection::Next,
        NavigationDirection::Previous => PrefetchDirection::Previous,
    };

    let prefetch_indices =
        compute_prefetch_pages(current_page, prefetch_dir, prefetch_count, total_pages);

    prefetch_indices
        .into_iter()
        .enumerate()
        .map(|(distance, page_idx)| {
            let priority = match distance {
                0 => JobPriority::PREFETCH_CLOSE,
                1 => JobPriority::PREFETCH_MEDIUM,
                _ => JobPriority::PREFETCH_FAR,
            };
            (page_idx, priority)
        })
        .collect()
}
