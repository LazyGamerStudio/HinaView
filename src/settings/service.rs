use super::model::SettingsState;

pub fn clamp_cpu_cache_mb(value: usize) -> usize {
    let v = value.clamp(128, 2048);
    (v / 128) * 128
}

pub fn clamp_gpu_cache_mb(value: usize, max_mb: usize) -> usize {
    let min = 64usize;
    let max = max_mb.max(min);
    let v = value.clamp(min, max);
    (v / 64) * 64
}

pub fn clamp_slideshow_sec(value: u32) -> u32 {
    value.clamp(0, 30)
}

pub fn clamp_auto_hide_sec(value: u32) -> u32 {
    value.clamp(1, 11)
}

pub fn clamp_webtoon_scroll_speed_px_per_sec(value: f32) -> f32 {
    value.clamp(100.0, 1600.0)
}

pub fn normalize(mut s: SettingsState, gpu_cap_mb: usize) -> SettingsState {
    s.cpu_cache_mb = clamp_cpu_cache_mb(s.cpu_cache_mb);
    s.gpu_cache_mb = clamp_gpu_cache_mb(s.gpu_cache_mb, gpu_cap_mb);
    s.slideshow_interval_sec = clamp_slideshow_sec(s.slideshow_interval_sec);
    s.ui_auto_hide_sec = clamp_auto_hide_sec(s.ui_auto_hide_sec);
    s.webtoon_scroll_speed_px_per_sec =
        clamp_webtoon_scroll_speed_px_per_sec(s.webtoon_scroll_speed_px_per_sec);
    s.filters.bright = s.filters.bright.clamp(-1.0, 1.0);
    s.filters.contrast = s.filters.contrast.clamp(0.0, 2.0);
    s.filters.gamma = s.filters.gamma.clamp(0.2, 3.0);
    s.filters.exposure = s.filters.exposure.clamp(-4.0, 4.0);
    s
}
