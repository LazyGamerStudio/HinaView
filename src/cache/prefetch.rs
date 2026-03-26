// src/cache/prefetch.rs
use crate::types::PageId;

/// Direction of navigation for prefetching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefetchDirection {
    /// Previous page (left arrow / navigate backward)
    Previous,
    /// Next page (right arrow / navigate forward)
    Next,
}

/// Computes the list of pages to prefetch based on navigation direction.
///
/// # Strategy
/// - Prefetch only in the direction the user is navigating
/// - Prefetch 3 pages ahead (configurable)
/// - Pages are ordered by priority (closest first)
///
/// # Arguments
/// * `current_index` - Current page index
/// * `direction` - Direction of navigation (Previous or Next)
/// * `prefetch_count` - Number of pages to prefetch (default: 3)
/// * `total_pages` - Total number of pages in document
///
/// # Returns
/// Vec of page indices to prefetch, ordered by priority (highest priority first)
///
/// # Examples
/// ```
/// // Current: page 5, Total: 10, Direction: Next, Count: 3
/// // Result: [6, 7, 8] (next 3 pages)
///
/// // Current: page 5, Total: 10, Direction: Previous, Count: 3
/// // Result: [4, 3, 2] (previous 3 pages)
/// ```
pub fn compute_prefetch_pages(
    current_index: PageId,
    direction: PrefetchDirection,
    prefetch_count: usize,
    total_pages: usize,
) -> Vec<PageId> {
    if total_pages == 0 {
        return Vec::new();
    }

    let mut result: Vec<PageId> = Vec::with_capacity(prefetch_count);

    match direction {
        PrefetchDirection::Next => {
            // Prefetch next pages: current+1, current+2, current+3, ...
            for i in 1..=prefetch_count {
                let page_idx = current_index + i;
                if page_idx < total_pages {
                    result.push(page_idx);
                }
            }
        }
        PrefetchDirection::Previous => {
            // Prefetch previous pages: current-1, current-2, current-3, ...
            for i in 1..=prefetch_count {
                if current_index >= i {
                    result.push(current_index - i);
                }
            }
        }
    }

    result
}

/// A priority-aware sliding window result.
/// Contains page indices and their protection priorities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlidingWindowPriority {
    pub page_id: PageId,
    pub priority: usize, // 0 = highest priority
}

/// Computes the sliding window of pages around current index with priorities.
///
/// # Priority Strategy (Lower value = Higher Priority)
/// 1. Current page: Priority 0 (Most protected)
/// 2. Forward window (relative to navigation): Priority 1 to N
/// 3. Backward window (relative to navigation): Priority N+1 to 2N
///
/// # Arguments
/// * `current_index` - Current page index
/// * `direction` - Navigation direction (Next/Previous)
/// * `prefetch_count` - Window size (N)
/// * `total_pages` - Total document length
pub fn compute_sliding_window_priorities(
    current_index: PageId,
    direction: PrefetchDirection,
    prefetch_count: usize,
    total_pages: usize,
) -> Vec<SlidingWindowPriority> {
    if total_pages == 0 {
        return Vec::new();
    }

    let mut results = Vec::with_capacity(1 + prefetch_count * 2);

    // 1. Current page (Priority 0)
    results.push(SlidingWindowPriority {
        page_id: current_index,
        priority: 0,
    });

    // Determine forward and backward steps based on navigation direction
    let (forward_steps, backward_steps) = match direction {
        PrefetchDirection::Next => (true, false), // Forward is +
        PrefetchDirection::Previous => (false, true), // Forward is -
    };

    // 2. Forward window (Priority 1 to N)
    for i in 1..=prefetch_count {
        let page_idx = if forward_steps {
            current_index.checked_add(i)
        } else {
            current_index.checked_sub(i)
        };

        if let Some(idx) = page_idx {
            if idx < total_pages {
                results.push(SlidingWindowPriority {
                    page_id: idx,
                    priority: i,
                });
            }
        }
    }

    // 3. Backward window (Priority N+1 to 2N)
    for i in 1..=prefetch_count {
        let page_idx = if backward_steps {
            current_index.checked_add(i)
        } else {
            current_index.checked_sub(i)
        };

        if let Some(idx) = page_idx {
            if idx < total_pages {
                results.push(SlidingWindowPriority {
                    page_id: idx,
                    priority: prefetch_count + i,
                });
            }
        }
    }

    results
}

/// Default prefetch count: 3 pages
#[allow(dead_code)]
pub const DEFAULT_PREFETCH_COUNT: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_next() {
        let result = compute_prefetch_pages(5, PrefetchDirection::Next, DEFAULT_PREFETCH_COUNT, 10);
        assert_eq!(result, vec![6, 7, 8]);
    }

    #[test]
    fn test_prefetch_previous() {
        let result =
            compute_prefetch_pages(5, PrefetchDirection::Previous, DEFAULT_PREFETCH_COUNT, 10);
        assert_eq!(result, vec![4, 3, 2]);
    }

    #[test]
    fn test_prefetch_boundary_end() {
        // Near the end, should stop at last page
        let result = compute_prefetch_pages(8, PrefetchDirection::Next, DEFAULT_PREFETCH_COUNT, 10);
        assert_eq!(result, vec![9]); // Only page 9 exists
    }

    #[test]
    fn test_prefetch_boundary_start() {
        // At the start, previous should return empty
        let result =
            compute_prefetch_pages(0, PrefetchDirection::Previous, DEFAULT_PREFETCH_COUNT, 10);
        assert!(result.is_empty());
    }

    #[test]
    fn test_prefetch_custom_count() {
        let result = compute_prefetch_pages(5, PrefetchDirection::Next, 5, 20);
        assert_eq!(result, vec![6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_prefetch_empty_document() {
        let result = compute_prefetch_pages(0, PrefetchDirection::Next, DEFAULT_PREFETCH_COUNT, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_sliding_window_priorities() {
        // Current: 5, Count: 3, Direction: Next, Total: 20
        // Window: [5](0), [6,7,8](1,2,3), [4,3,2](4,5,6)
        let result = compute_sliding_window_priorities(5, PrefetchDirection::Next, 3, 20);

        assert_eq!(result.len(), 7);
        assert_eq!(
            result[0],
            SlidingWindowPriority {
                page_id: 5,
                priority: 0
            }
        ); // Current
        assert_eq!(
            result[1],
            SlidingWindowPriority {
                page_id: 6,
                priority: 1
            }
        ); // Forward 1
        assert_eq!(
            result[2],
            SlidingWindowPriority {
                page_id: 7,
                priority: 2
            }
        ); // Forward 2
        assert_eq!(
            result[3],
            SlidingWindowPriority {
                page_id: 8,
                priority: 3
            }
        ); // Forward 3
        assert_eq!(
            result[4],
            SlidingWindowPriority {
                page_id: 4,
                priority: 4
            }
        ); // Backward 1
        assert_eq!(
            result[5],
            SlidingWindowPriority {
                page_id: 3,
                priority: 5
            }
        ); // Backward 2
        assert_eq!(
            result[6],
            SlidingWindowPriority {
                page_id: 2,
                priority: 6
            }
        ); // Backward 3
    }

    #[test]
    fn test_sliding_window_priorities_previous() {
        // Current: 5, Count: 3, Direction: Previous, Total: 20
        // Window: [5](0), [4,3,2](1,2,3), [6,7,8](4,5,6)
        let result = compute_sliding_window_priorities(5, PrefetchDirection::Previous, 3, 20);

        assert_eq!(result.len(), 7);
        assert_eq!(
            result[0],
            SlidingWindowPriority {
                page_id: 5,
                priority: 0
            }
        ); // Current
        assert_eq!(
            result[1],
            SlidingWindowPriority {
                page_id: 4,
                priority: 1
            }
        ); // Forward 1 (Previous direction)
        assert_eq!(
            result[2],
            SlidingWindowPriority {
                page_id: 3,
                priority: 2
            }
        ); // Forward 2
        assert_eq!(
            result[3],
            SlidingWindowPriority {
                page_id: 2,
                priority: 3
            }
        ); // Forward 3
        assert_eq!(
            result[4],
            SlidingWindowPriority {
                page_id: 6,
                priority: 4
            }
        ); // Backward 1 (Next direction)
        assert_eq!(
            result[5],
            SlidingWindowPriority {
                page_id: 7,
                priority: 5
            }
        ); // Backward 2
        assert_eq!(
            result[6],
            SlidingWindowPriority {
                page_id: 8,
                priority: 6
            }
        ); // Backward 3
    }
}
