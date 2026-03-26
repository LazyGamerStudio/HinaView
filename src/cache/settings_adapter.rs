use crate::cache::TextureManager;
use crate::pipeline::DecodeScheduler;

pub fn apply_cpu_cache_limit(scheduler: &mut DecodeScheduler, mb: usize) {
    scheduler.set_cpu_cache_limit_mb(mb);
}

pub fn apply_gpu_cache_limit(texture_manager: &mut TextureManager, mb: usize) {
    texture_manager.set_gpu_cache_limit_mb(mb);
}
