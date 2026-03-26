use crate::pipeline::decoders::FrameStream;
use crate::types::PageId;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct DueAnimationFrame {
    pub page_id: PageId,
    pub pixels: Vec<u8>,
}

pub struct AnimationState {
    pub stream: Arc<Mutex<Box<dyn FrameStream>>>,
    pub last_update: Instant,
    pub next_delay: Duration,
    pub completed_loops: u32,
    pub has_started: bool,
    pub current_frame: usize,
}

pub struct AnimationController {
    active_animations: HashMap<PageId, AnimationState>,
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            active_animations: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        page_id: PageId,
        stream: Arc<Mutex<Box<dyn FrameStream>>>,
        initial_delay: Duration,
    ) {
        // Replace existing stream for the same page to keep the latest decode source.
        self.active_animations.insert(
            page_id,
            AnimationState {
                stream,
                last_update: Instant::now(),
                next_delay: initial_delay,
                completed_loops: 0,
                has_started: false,
                current_frame: 0,
            },
        );
    }

    pub fn clear(&mut self) {
        self.active_animations.clear();
    }

    /// Returns (has_animation, completed_loops) in a single hashmap lookup
    pub fn get_animation_status(&self, page_id: PageId) -> (bool, u32) {
        self.active_animations
            .get(&page_id)
            .map(|s| (true, s.completed_loops))
            .unwrap_or((false, 0))
    }

    pub fn is_active(&self, page_id: PageId) -> bool {
        self.active_animations.contains_key(&page_id)
    }

    pub fn has_active_for(&self, visible_pages: &[PageId]) -> bool {
        visible_pages
            .iter()
            .any(|&page_id| self.active_animations.contains_key(&page_id))
    }

    pub fn next_redraw_deadline(&self, visible_pages: &[PageId], now: Instant) -> Option<Instant> {
        self.active_animations
            .iter()
            .filter(|(page_id, _)| visible_pages.contains(page_id))
            .map(|(_, state)| state.last_update + state.next_delay)
            .min()
            .map(|deadline| deadline.max(now))
    }

    pub fn retain_visible(&mut self, visible_pages: &[PageId]) {
        self.active_animations
            .retain(|&page_id, _| visible_pages.contains(&page_id));
    }

    pub fn collect_due_frames(&mut self, visible_pages: &[PageId]) -> Vec<DueAnimationFrame> {
        let now = Instant::now();
        let mut due_frames = Vec::new();

        for (page_id, state) in &mut self.active_animations {
            // OPTIMIZATION: ONLY update animations for visible pages.
            // This prevents background animations from consuming VRAM/CPU.
            if !visible_pages.contains(page_id) {
                continue;
            }

            if now.duration_since(state.last_update) >= state.next_delay {
                let lock_t0 = Instant::now();
                let mut stream = state.stream.lock();
                let lock_wait_ms = lock_t0.elapsed().as_secs_f32() * 1000.0;
                if lock_wait_ms >= 0.1 {
                    tracing::debug!(
                        "[Animation][LockWait] stream_mutex={:.2}ms page={}",
                        lock_wait_ms,
                        page_id
                    );
                }
                if let Some(frame) = stream.next_frame() {
                    if frame.is_first_frame {
                        state.current_frame = 0;
                    } else {
                        state.current_frame += 1;
                    }

                    // Track loop completion: increment when we SEE the first frame again
                    // This means the previous loop has completed
                    if frame.is_first_frame && state.has_started {
                        state.completed_loops += 1;
                    }
                    state.has_started = true;

                    state.last_update = now;
                    state.next_delay = frame.delay;
                    due_frames.push(DueAnimationFrame {
                        page_id: *page_id,
                        pixels: frame.pixels,
                    });
                }
            }
        }

        due_frames
    }
}
