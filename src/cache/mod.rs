// src/cache/mod.rs
pub mod cpu_cache;
pub mod gpu_cache;
pub mod gpu_uploader;
pub mod prefetch;
pub mod settings_adapter;
pub mod texture_index;
pub mod texture_manager;

pub use cpu_cache::CpuDecodeCache;
pub use gpu_cache::GpuTextureCache;
pub use prefetch::{PrefetchDirection, compute_prefetch_pages, compute_sliding_window_priorities};
pub use texture_manager::TextureManager;

pub fn log_gpu_upload_info(name: &str, mip: crate::types::MipLevel, w: u32, h: u32, ms: f32) {
    tracing::debug!(
        "[GPU Upload] {} | {:?}: {}x{} ({:.2}ms)",
        name,
        mip,
        w,
        h,
        ms
    );
}
