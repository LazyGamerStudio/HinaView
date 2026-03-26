// src/view/page_navigator.rs
// Pure logical page target calculations for navigation.
// Separated from NavigationController to adhere to SRP.

use crate::document::Document;
use crate::types::LayoutMode;
use crate::types::PageId;
use crate::view::navigation_types::{NavState, NavigationDirection};
use std::time::Instant;

/// Result of a navigation step calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationStepResult {
    pub target_page: PageId,
    pub direction: NavigationDirection,
    pub changed: bool,
}

/// Pure navigation logic for calculating page targets.
/// This struct contains no state related to rendering, camera, or layout.
pub struct PageNavigator {
    /// Current page index
    pub current_page: Option<PageId>,
    /// Pending page (waiting for transition)
    pub pending_page: Option<PageId>,
    /// Target page (accumulator for keyboard events)
    pub target_page: Option<PageId>,
    /// Last navigation direction
    pub last_navigation_direction: Option<NavigationDirection>,
    /// FSM state for high-speed navigation
    pub state: NavState,
    /// When the current fast navigation session started
    pub fast_nav_start_instant: Option<Instant>,
    /// Accumulator for gradual acceleration (0.0 to 1.0+)
    pub fast_nav_accumulator: f32,
}

impl PageNavigator {
    pub fn new() -> Self {
        Self {
            current_page: None,
            pending_page: None,
            target_page: None,
            last_navigation_direction: None,
            state: NavState::Idle,
            fast_nav_start_instant: None,
            fast_nav_accumulator: 0.0,
        }
    }

    /// Calculate the target page for a spread (dual mode) navigation step.
    pub fn spread_step_target(
        &self,
        document: &Document,
        current: PageId,
        delta: i32,
        layout_mode: LayoutMode,
    ) -> Option<PageId> {
        let LayoutMode::Dual { rtl, .. } = layout_mode else {
            return None;
        };

        let current_spread_idx = document
            .spreads
            .iter()
            .position(|s| s.left == Some(current) || s.right == Some(current))?;
        let next_spread_idx = (current_spread_idx as i32 + delta.signum())
            .clamp(0, document.spreads.len() as i32 - 1) as usize;
        if next_spread_idx == current_spread_idx {
            return Some(current);
        }

        let spread = document.spreads.get(next_spread_idx)?;
        let target = if rtl {
            spread.right.or(spread.left)
        } else {
            spread.left.or(spread.right)
        }?;
        Some(target)
    }

    /// Process a navigation step and return the result.
    ///
    /// # Arguments
    /// * `document` - Current document for spread calculations
    /// * `layout_mode` - Current layout mode (Single, Dual, VerticalScroll)
    /// * `delta` - Navigation direction (-1 for previous, +1 for next)
    /// * `is_repeat` - Whether this is a repeated key hold event
    /// * `is_pressed` - Whether the key is currently pressed
    ///
    /// # Returns
    /// `Some(NavigationStepResult)` if navigation occurred, `None` otherwise
    pub fn navigate_step(
        &mut self,
        document: Option<&Document>,
        layout_mode: LayoutMode,
        delta: i32,
        is_repeat: bool,
        is_pressed: bool,
    ) -> Option<NavigationStepResult> {
        let (doc_len, current) = match (document, self.current_page) {
            (Some(doc), Some(curr)) if !doc.pages.is_empty() => (doc.pages.len(), curr),
            _ => return None,
        };

        if !is_pressed {
            let was_fast = matches!(self.state, NavState::FastNavigating);
            if was_fast {
                tracing::debug!("[Nav] Fast Navigation STOPPED");
            }
            self.state = NavState::Idle;
            self.fast_nav_start_instant = None;
            self.fast_nav_accumulator = 0.0;
            return None;
        }

        // --- Fast Navigation Logic (Accumulate target_page with Acceleration) ---
        if is_repeat {
            if crate::view::navigation_fsm::should_enter_fast_navigation(
                is_repeat,
                matches!(self.state, NavState::Idle),
            ) {
                tracing::debug!("[Nav] Fast Navigation STARTED");
                self.state = NavState::FastNavigating;
                self.fast_nav_start_instant = Some(Instant::now());
                self.fast_nav_accumulator = 0.0;
            }

            // Calculate acceleration: 0.05 to 1.0 using ln(1+t) curve over 5 seconds
            // This provides fast initial acceleration while tapering off towards the maximum.
            let acceleration = if let Some(start) = self.fast_nav_start_instant {
                let t = start.elapsed().as_secs_f32();
                // ln(1+t) / ln(6) results in 0.0 at t=0 and 1.0 at t=5
                ((1.0 + t).ln() / (6.0f32).ln()).clamp(0.05, 1.0)
            } else {
                1.0
            };

            self.fast_nav_accumulator += acceleration;

            if self.fast_nav_accumulator < 1.0 {
                return None;
            }
            self.fast_nav_accumulator -= 1.0;

            let base =
                self.target_page
                    .unwrap_or(crate::view::navigation_fsm::fast_nav_start_page(
                        self.pending_page,
                        current,
                    ));
            let next_target = crate::view::navigation_fsm::clamp_target_page(base, delta, doc_len);

            if next_target != base {
                let direction = self.infer_direction(current, next_target);
                self.target_page = Some(next_target);

                return Some(NavigationStepResult {
                    target_page: next_target,
                    direction,
                    changed: true,
                });
            }
            return None;
        }

        // Reset acceleration state on single step navigation
        self.fast_nav_start_instant = None;
        self.fast_nav_accumulator = 0.0;

        if self.pending_page.is_some() && matches!(self.state, NavState::Idle) {
            return None;
        }

        let step_base = self.pending_page.unwrap_or(current);
        let next = if matches!(layout_mode, LayoutMode::Dual { .. }) {
            document
                .and_then(|doc| self.spread_step_target(doc, step_base, delta, layout_mode))
                .unwrap_or(step_base)
        } else {
            crate::view::navigation_fsm::clamp_target_page(step_base, delta, doc_len)
        };

        if next != step_base {
            let direction = self.infer_direction(step_base, next);
            self.target_page = Some(next);
            return Some(NavigationStepResult {
                target_page: next,
                direction,
                changed: true,
            });
        }

        None
    }

    /// Navigate to a specific page.
    pub fn navigate(
        &mut self,
        target_page: PageId,
        current_layout_mode: LayoutMode,
    ) -> NavigationStepResult {
        self.target_page = Some(target_page);

        let direction = self.infer_direction_from_current(target_page);

        let is_same_page = self.current_page == Some(target_page);
        if !is_same_page {
            if !matches!(current_layout_mode, LayoutMode::VerticalScroll) {
                // Image offset reset will be handled by the caller
            }
            self.last_navigation_direction = Some(direction);
        }

        NavigationStepResult {
            target_page,
            direction,
            changed: true,
        }
    }

    /// Infer navigation direction from two page indices.
    fn infer_direction(&self, from: PageId, to: PageId) -> NavigationDirection {
        if to < from {
            NavigationDirection::Previous
        } else {
            NavigationDirection::Next
        }
    }

    /// Infer navigation direction from current page to target.
    fn infer_direction_from_current(&self, target: PageId) -> NavigationDirection {
        if let Some(current) = self.current_page {
            self.infer_direction(current, target)
        } else {
            self.last_navigation_direction
                .unwrap_or(NavigationDirection::Next)
        }
    }

    /// Check if currently in fast navigation mode.
    pub fn is_fast_navigating(&self) -> bool {
        matches!(self.state, NavState::FastNavigating)
    }

    /// Get the current effective page (current or pending).
    pub fn effective_page(&self) -> Option<PageId> {
        self.pending_page.or(self.current_page)
    }

    /// Commit the pending page to current.
    pub fn commit_pending(&mut self) -> Option<PageId> {
        self.pending_page.take().map(|page| {
            self.current_page = Some(page);
            page
        })
    }

    /// Set the pending page.
    pub fn set_pending(&mut self, page: PageId) {
        self.pending_page = Some(page);
    }

    /// Clear navigation state (used when document changes).
    pub fn clear(&mut self) {
        self.current_page = None;
        self.pending_page = None;
        self.target_page = None;
        self.last_navigation_direction = None;
        self.state = NavState::Idle;
        self.fast_nav_start_instant = None;
        self.fast_nav_accumulator = 0.0;
    }
}

impl Default for PageNavigator {
    fn default() -> Self {
        Self::new()
    }
}
