use parking_lot::{Condvar, Mutex};
use std::sync::Arc;

/// Dynamic Decoding Concurrency Control (DDCC)
/// Limits the number of concurrent heavy decoding tasks based on CPU core count.
pub struct DecodingSemaphore {
    state: Mutex<SemaphoreState>,
    cvar: Condvar,
}

struct SemaphoreState {
    permits: usize,
    available: usize,
}

impl DecodingSemaphore {
    /// Create a new semaphore with a limit calculated from the system core count.
    /// Formula: max(1, total_cores - 1)
    pub fn new_auto() -> Arc<Self> {
        let total_cores = num_cpus::get();
        // 4 cores -> 3, 6 cores -> 5, 8 cores -> 7
        let limit = total_cores.saturating_sub(1).max(1);

        tracing::info!(
            "[DDCC] Initialized with {} permits (Total CPU Cores: {})",
            limit,
            total_cores
        );

        Arc::new(Self {
            state: Mutex::new(SemaphoreState {
                permits: limit,
                available: limit,
            }),
            cvar: Condvar::new(),
        })
    }

    /// Acquire a permit. Blocks if no permits are available.
    pub fn acquire(&self) {
        let mut state = self.state.lock();
        while state.available == 0 {
            self.cvar.wait(&mut state);
        }
        state.available -= 1;
    }

    /// Try to acquire a permit without blocking.
    /// Returns true if permit was acquired, false otherwise.
    pub fn try_acquire(&self) -> bool {
        let mut state = self.state.lock();
        if state.available > 0 {
            state.available -= 1;
            true
        } else {
            false
        }
    }

    /// Release a permit, allowing other waiting tasks to proceed.
    pub fn release(&self) {
        let mut state = self.state.lock();
        state.available += 1;
        self.cvar.notify_one();
    }

    pub fn limit(&self) -> usize {
        self.state.lock().permits
    }

    /// Dynamically update decode concurrency limit.
    /// Running workers keep their permits; only future acquisitions are affected.
    pub fn set_limit(&self, new_limit: usize) {
        let mut state = self.state.lock();
        let target = new_limit.max(1);
        let in_use = state.permits.saturating_sub(state.available);
        state.permits = target;
        state.available = target.saturating_sub(in_use);
        self.cvar.notify_all();
    }
}

/// A RAII guard that releases the permit when dropped.
pub struct DecodingPermit {
    semaphore: Arc<DecodingSemaphore>,
}

impl DecodingPermit {
    pub fn acquire(semaphore: Arc<DecodingSemaphore>) -> Self {
        semaphore.acquire();
        Self { semaphore }
    }

    pub fn try_acquire(semaphore: Arc<DecodingSemaphore>) -> Option<Self> {
        if semaphore.try_acquire() {
            Some(Self { semaphore })
        } else {
            None
        }
    }
}

impl Drop for DecodingPermit {
    fn drop(&mut self) {
        self.semaphore.release();
    }
}
