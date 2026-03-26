// src/view/navigation_controller.rs

use crate::camera::Camera;
use crate::document::Document;
use crate::layout::LayoutResult;
use crate::types::PageId;
use crate::view::navigation_types::{NavState, NavigationDirection};
use crate::view::{FitMode, LayoutMode, PageNavigator, RotationQuarter, ViewState};

pub struct NavigationController {
    pub document: Option<Document>,
    pub view: ViewState,
    pub camera: Camera,
    pub layout: Option<LayoutResult>,
    // Embedded navigator for pure logic
    pub navigator: PageNavigator,
    // Legacy fields for backward compatibility - mirror navigator fields
    pub current_page: Option<PageId>,
    pub pending_page: Option<PageId>,
    pub target_page: Option<PageId>,
    pub last_navigation_direction: Option<NavigationDirection>,
    pub state: NavState,
    pub prefetch_after_first_present: bool,
    pub webtoon_scroll_target_y: Option<f32>,
    pub webtoon_last_request_pan_y: Option<f32>,
}

impl NavigationController {
    pub fn new() -> Self {
        Self {
            document: None,
            view: ViewState {
                zoom: 1.0,
                pan: [0.0, 0.0],
                image_offset: [0.0, 0.0],
                layout_mode: LayoutMode::Single,
                rotation: RotationQuarter::Deg0,
                fit_mode: FitMode::FitScreen,
            },
            camera: Camera::new(),
            layout: None,
            navigator: PageNavigator::new(),
            current_page: None,
            pending_page: None,
            target_page: None,
            last_navigation_direction: None,
            state: NavState::Idle,
            prefetch_after_first_present: false,
            webtoon_scroll_target_y: None,
            webtoon_last_request_pan_y: None,
        }
    }

    /// Sync legacy fields from navigator
    fn sync_from_navigator(&mut self) {
        self.current_page = self.navigator.current_page;
        self.pending_page = self.navigator.pending_page;
        self.target_page = self.navigator.target_page;
        self.last_navigation_direction = self.navigator.last_navigation_direction;
        self.state = self.navigator.state.clone();
    }

    /// Sync navigator from legacy fields
    fn sync_to_navigator(&mut self) {
        self.navigator.current_page = self.current_page;
        self.navigator.pending_page = self.pending_page;
        self.navigator.target_page = self.target_page;
        self.navigator.last_navigation_direction = self.last_navigation_direction;
        self.navigator.state = self.state.clone();
    }

    pub fn navigate_step(&mut self, delta: i32, is_repeat: bool, is_pressed: bool) {
        // Sync legacy fields to navigator before navigation
        self.sync_to_navigator();

        let layout_mode = self.view.layout_mode;
        let result = self.navigator.navigate_step(
            self.document.as_ref(),
            layout_mode,
            delta,
            is_repeat,
            is_pressed,
        );

        let Some(result) = result else {
            self.sync_from_navigator();
            return;
        };

        // Apply navigation result to both navigator and legacy fields
        self.navigator.target_page = Some(result.target_page);
        self.navigator.last_navigation_direction = Some(result.direction);
        self.target_page = Some(result.target_page);
        self.last_navigation_direction = Some(result.direction);

        let is_same_page = self.current_page == Some(result.target_page);
        if !is_same_page {
            if !matches!(self.view.layout_mode, LayoutMode::VerticalScroll) {
                self.view.image_offset = [0.0, 0.0];
            }
        }

        if self.current_page.is_none() {
            self.current_page = Some(result.target_page);
            self.navigator.current_page = Some(result.target_page);
            self.center_camera_on_page(result.target_page);
        } else {
            self.pending_page = Some(result.target_page);
            self.navigator.pending_page = Some(result.target_page);
        }
    }

    pub fn navigate(&mut self, target_page: PageId) {
        let result = self.navigator.navigate(target_page, self.view.layout_mode);

        self.navigator.target_page = Some(result.target_page);
        self.navigator.last_navigation_direction = Some(result.direction);
        self.target_page = Some(result.target_page);
        self.last_navigation_direction = Some(result.direction);

        let is_same_page = self.current_page == Some(target_page);
        if !is_same_page {
            if !matches!(self.view.layout_mode, LayoutMode::VerticalScroll) {
                self.view.image_offset = [0.0, 0.0];
            }
        }

        if self.current_page.is_none() {
            self.current_page = Some(target_page);
            self.navigator.current_page = Some(target_page);
            self.center_camera_on_page(target_page);
        } else {
            self.pending_page = Some(target_page);
            self.navigator.pending_page = Some(target_page);
        }
    }

    /// Proactively evict GPU cache entries far from the current page during fast navigation.
    /// This prevents VRAM overflow when user rapidly navigates through many pages.
    #[allow(dead_code)]
    pub fn evict_gpu_cache_far_pages(
        &self,
        texture_manager: &mut crate::cache::TextureManager,
        max_pages_to_keep: usize,
    ) {
        let Some(current) = self.current_page else {
            return;
        };

        let half = max_pages_to_keep / 2;
        let min_keep = current.saturating_sub(half);
        let max_keep = current.saturating_add(half);

        let to_remove: Vec<PageId> = texture_manager
            .textures
            .keys()
            .filter(|&&id| id < min_keep || id > max_keep)
            .cloned()
            .collect();

        if !to_remove.is_empty() {
            for page_id in to_remove {
                texture_manager.remove_page(page_id);
            }
        }

        let cache_usage_mb = texture_manager.gpu_cache_memory_mb();
        let cache_max_mb = texture_manager.gpu_cache_max_mb();
        if cache_usage_mb > cache_max_mb * 90 / 100 {
            tracing::debug!(
                "[GPU Cache] High memory pressure after eviction: {}MB / {}MB",
                cache_usage_mb,
                cache_max_mb
            );
        }
    }

    pub fn maybe_run_deferred_prefetch(
        &mut self,
        texture_manager: &crate::cache::TextureManager,
    ) -> bool {
        if !self.prefetch_after_first_present {
            return false;
        }

        let Some(current_page) = self.current_page else {
            self.prefetch_after_first_present = false;
            return false;
        };

        if !texture_manager.has_page(current_page) {
            return false;
        }

        self.prefetch_after_first_present = false;
        true
    }

    pub fn get_prefetch_plan(
        &self,
        prefetch_count: usize,
    ) -> Vec<(PageId, crate::pipeline::JobPriority)> {
        let (Some(current_page), Some(doc)) = (self.current_page, &self.document) else {
            return Vec::new();
        };

        let direction = self
            .last_navigation_direction
            .unwrap_or(NavigationDirection::Next);

        crate::view::navigation_planner::prefetch_plan(
            current_page,
            direction,
            doc.pages.len(),
            prefetch_count,
        )
        .into_iter()
        .collect()
    }

    pub fn center_camera_on_page(&mut self, page: PageId) {
        crate::view::layout_sync::center_camera_on_page(
            self.layout.as_ref(),
            &mut self.view,
            &mut self.camera,
            page,
        );
    }

    pub fn update_zoom_for_current_page(&mut self, window_size: (u32, u32)) {
        crate::view::layout_sync::update_zoom_for_current_page(
            self.document.as_ref(),
            self.current_page,
            &mut self.view,
            &mut self.camera,
            window_size,
        );
    }

    pub fn refresh_layout(&mut self, window_size: (u32, u32)) {
        if let Some(doc) = &mut self.document {
            self.layout = Some(crate::view::layout_sync::rebuild_layout(doc, &self.view));
        }

        self.update_zoom_for_current_page(window_size);

        if let Some(curr) = self.current_page {
            if matches!(self.view.layout_mode, LayoutMode::VerticalScroll) {
                self.refresh_camera();
            } else {
                self.center_camera_on_page(curr);
            }
        } else {
            self.refresh_camera();
        }
    }

    pub fn refresh_camera(&mut self) {
        crate::view::layout_sync::refresh_camera(
            &mut self.view,
            &mut self.camera,
            self.layout.as_ref(),
            self.current_page,
        );
    }

    pub fn is_fast_navigating(&self) -> bool {
        self.navigator.is_fast_navigating()
    }

    pub fn get_visible_pages(&self, window_size: (u32, u32)) -> Vec<PageId> {
        let mut visible_pages = Vec::new();
        let Some(current_page) = self.current_page else {
            return visible_pages;
        };

        visible_pages.push(current_page);

        if let Some(pending) = self.pending_page {
            if pending != current_page {
                visible_pages.push(pending);
            }
        }

        match self.view.layout_mode {
            crate::types::LayoutMode::VerticalScroll => {
                if let Some(layout) = self.layout.as_ref() {
                    let zoom = self.camera.zoom.max(0.0001);
                    let half_w = window_size.0 as f32 / (2.0 * zoom);
                    let half_h = window_size.1 as f32 / (2.0 * zoom);
                    let margin_y = half_h;

                    let view_min_x = self.camera.pan.x - half_w;
                    let view_max_x = self.camera.pan.x + half_w;
                    let view_min_y = self.camera.pan.y - half_h - margin_y;
                    let view_max_y = self.camera.pan.y + half_h + margin_y;

                    for placement in &layout.placements {
                        let page_min_x = placement.position[0];
                        let page_max_x = placement.position[0] + placement.size[0];
                        let page_min_y = placement.position[1];
                        let page_max_y = placement.position[1] + placement.size[1];
                        let visible = !(page_max_x < view_min_x
                            || page_min_x > view_max_x
                            || page_min_y > view_max_y
                            || page_max_y < view_min_y);
                        if visible && placement.page_index != current_page {
                            visible_pages.push(placement.page_index);
                        }
                    }
                }
            }
            crate::types::LayoutMode::Dual { .. } => {
                if let Some(doc) = self.document.as_ref()
                    && let Some(spread) = doc
                        .spreads
                        .iter()
                        .find(|s| s.left == Some(current_page) || s.right == Some(current_page))
                {
                    let partner = if spread.left == Some(current_page) {
                        spread.right
                    } else {
                        spread.left
                    };
                    if let Some(partner_page) = partner {
                        visible_pages.push(partner_page);
                    }
                }
            }
            _ => {}
        }

        visible_pages
    }
}
