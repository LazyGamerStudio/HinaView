use crate::filter::FilterParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FitModeSetting {
    FitScreen,
    FitWidth,
    FitHeight,
    Zoom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayoutModeSetting {
    Single,
    DualLtr,
    DualRtl,
    VerticalScroll,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ThemeModeSetting {
    #[default]
    Auto,
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArchiveSortingMode {
    Mixed,
    FoldersFirst,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ConfigStorageLocation {
    #[default]
    AppDir,
    SystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSectionOpenState {
    pub info: bool,
    pub exif: bool,
    pub view_mode: bool,
    pub filter: bool,
    pub preference: bool,
}

impl Default for UiSectionOpenState {
    fn default() -> Self {
        Self {
            info: true,
            exif: false,
            view_mode: false,
            filter: false,
            preference: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsState {
    pub fit_mode: FitModeSetting,
    pub layout_mode: LayoutModeSetting,
    pub first_page_offset: bool,
    pub slideshow_enabled: bool,
    pub slideshow_interval_sec: u32,
    pub ui_auto_hide_sec: u32,
    pub prefetch_count: u32,
    pub cpu_cache_mb: usize,
    pub gpu_cache_mb: usize,
    pub settings_window_collapsed: bool,
    pub shortcuts_window_collapsed: bool,
    pub archive_sorting_mode: ArchiveSortingMode,
    pub remember_document_position: bool,
    pub webtoon_scroll_speed_px_per_sec: f32,
    pub theme_mode: ThemeModeSetting,
    pub single_instance: bool,
    pub config_storage_location: ConfigStorageLocation,
    pub sections_open: UiSectionOpenState,
    pub filters: FilterParams,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            fit_mode: FitModeSetting::FitScreen,
            layout_mode: LayoutModeSetting::Single,
            first_page_offset: false,
            slideshow_enabled: false,
            slideshow_interval_sec: 3,
            ui_auto_hide_sec: 3,
            prefetch_count: 3,
            cpu_cache_mb: 256,
            gpu_cache_mb: 512,
            settings_window_collapsed: false,
            shortcuts_window_collapsed: false,
            archive_sorting_mode: ArchiveSortingMode::Mixed,
            remember_document_position: true,
            webtoon_scroll_speed_px_per_sec: 1200.0,
            theme_mode: ThemeModeSetting::Auto,
            single_instance: true,
            config_storage_location: ConfigStorageLocation::AppDir,
            sections_open: UiSectionOpenState::default(),
            filters: FilterParams::default(),
        }
    }
}
