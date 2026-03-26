pub mod favorites;
pub mod file_association;
pub mod file_association_icons;
pub mod settings;
pub mod shortcuts;
pub mod snapshot;

use crate::i18n::localizer::LangFileAssociation;
use crate::i18n::{
    LangBookmark, LangExif, LangFilter, LangInfo, LangMeta, LangPreference, LangViewMode,
};

#[derive(Clone, Copy)]
pub enum UiFitMode {
    FitScreen,
    FitWidth,
    FitHeight,
    Zoom,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiLayoutMode {
    Single,
    DualLtr,
    DualRtl,
    VerticalScroll,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiThemeMode {
    Auto,
    Dark,
    Light,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiArchiveSortingMode {
    Mixed,
    FoldersFirst,
}

#[derive(Clone)]
pub struct UiBookmarkRow {
    pub id: u64,
    pub source_label: String,
    pub archive_name: String,
    pub page_name: String,
}

pub enum UiAction {
    OpenBookmark(u64),
    DeleteBookmark(u64),
    DismissBookmarkLimitDialog,
    SetFitMode(UiFitMode),
    SetLayoutMode(UiLayoutMode),
    SetFirstPageOffset(bool),
    SetSlideshowEnabled(bool),
    SetSlideshowIntervalSec(u32),
    SetCpuCacheMb(usize),
    SetGpuCacheMb(usize),
    SetUiAutoHideSec(u32),
    SetPrefetchCount(u32),
    SetRememberDocumentPosition(bool),
    SetWebtoonScrollSpeed(f32),
    SetThemeMode(UiThemeMode),
    SetSingleInstanceMode(bool),
    SetConfigStorageLocation(crate::settings::model::ConfigStorageLocation),
    ToggleAboutDialog(bool),
    SetInfoSectionOpen(bool),
    SetExifSectionOpen(bool),
    SetViewModeSectionOpen(bool),
    SetFilterSectionOpen(bool),
    SetPreferenceSectionOpen(bool),
    SetSettingsWindowCollapsed(bool),
    SetShortcutsWindowCollapsed(bool),
    /// CRITICAL: Specifically for the custom Drawer implementation. DO NOT change to standard Window toggle.
    SetBookmarkDrawerOpen(bool),
    SetFilterBypassColor(bool),
    SetFilterBypassMedian(bool),
    SetFilterBypassFsr(bool),
    SetFilterBypassDetail(bool),
    SetFilterBypassLevels(bool),
    SetFilterBright(f32),
    SetFilterContrast(f32),
    SetFilterGamma(f32),
    SetFilterExposure(f32),
    SetFilterFsrSharpness(f32),
    SetFilterMedianStrength(f32),
    SetFilterMedianStride(f32),
    SetFilterBlurRadius(f32),
    SetFilterUnsharpAmount(f32),
    SetFilterUnsharpThreshold(f32),
    SetFilterLevelsInBlack(f32),
    SetFilterLevelsInWhite(f32),
    SetFilterLevelsGamma(f32),
    SetFilterLevelsOutBlack(f32),
    SetFilterLevelsOutWhite(f32),
    SetArchiveSortingMode(UiArchiveSortingMode),
    ResetFilterColor,
    ResetFilterMedian,
    ResetFilterFsr,
    ResetFilterDetail,
    ResetFilterLevels,
    ResetFilters,
    ResetView,
    SetLocale(String),
    // File Association
    ShowFileAssociationWindow(bool),
    UpdateFileAssociation(String, bool),
    ApplyFileAssociations,
    SelectAllFileAssociations(bool),
    DeleteAllFileAssociations,
    AddContextMenu,
    DeleteContextMenu,
    RegisterStartMenuShortcut,
    UnregisterStartMenuShortcut,
}

pub struct UiSnapshot {
    // Labels (i18n)
    pub settings_title: String,
    pub favorites_title: String,
    pub shortcuts_title: String,
    pub lang_info: LangInfo,
    pub lang_exif: LangExif,
    pub lang_view_mode: LangViewMode,
    pub lang_filter: LangFilter,
    pub lang_preference: LangPreference,
    pub lang_file_association: LangFileAssociation,
    pub lang_bookmark: LangBookmark,
    // lang_shortcuts is only used during snapshot building, no need to store here

    // Values
    pub archive_name_value: String,
    pub file_name_value: String,
    pub file_size_value: String,
    pub info_value: String,
    pub icc_profile_value: String,
    pub exif_camera_value: String,
    pub exif_lens_value: String,
    pub exif_f_stop_value: String,
    pub exif_shutter_value: String,
    pub exif_iso_value: String,
    pub exif_datetime_value: String,
    pub ram_cache_display: String,

    // State
    pub info_section_open: bool,
    pub exif_section_open: bool,
    pub view_mode_section_open: bool,
    filter_section_open: bool,
    pub preference_section_open: bool,
    pub ui_windows_visible: bool,
    pub fit_mode: UiFitMode,
    pub layout_mode: UiLayoutMode,
    pub layout_vertical_scroll_label: String,
    pub first_page_offset: bool,
    pub slideshow_enabled: bool,
    pub slideshow_interval_sec: u32,
    pub slideshow_repeat_label: String,
    pub ui_auto_hide_sec: u32,
    pub ui_auto_hide_label: String,
    pub prefetch_count: u32,
    pub cpu_cache_current_mb: usize,
    pub cpu_cache_max_mb: usize,
    pub cpu_cache_setting_mb: usize,
    pub ram_cache_setting_label: String,
    pub gpu_cache_allowed_max_mb: usize,
    pub gpu_cache_setting_mb: usize,
    pub vram_cache_setting_label: String,
    pub shortcuts_lines: Vec<String>,
    pub bookmark_rows: Vec<UiBookmarkRow>,
    pub settings_window_collapsed: bool,
    pub shortcuts_window_collapsed: bool,
    pub bookmark_drawer_open: bool,
    pub archive_sorting_mode: UiArchiveSortingMode,
    pub remember_document_position: bool,
    pub webtoon_scroll_speed_px_per_sec: f32,
    pub theme_mode: UiThemeMode,
    pub single_instance: bool,
    pub config_storage_location: crate::settings::model::ConfigStorageLocation,
    pub ui_opacity: f32,
    pub accent_color: Option<egui::Color32>,
    pub filter_bypass_color: bool,
    pub filter_bypass_median: bool,
    pub filter_bypass_fsr: bool,
    pub filter_bypass_detail: bool,
    pub filter_bypass_levels: bool,
    pub filter_bright: f32,
    pub filter_contrast: f32,
    pub filter_gamma: f32,
    pub filter_exposure: f32,
    pub filter_fsr_sharpness: f32,
    pub filter_median_strength: f32,
    pub filter_median_stride: f32,
    pub filter_blur_radius: f32,
    pub filter_unsharp_amount: f32,
    pub filter_unsharp_threshold: f32,
    pub filter_levels_in_black: f32,
    pub filter_levels_in_white: f32,
    pub filter_levels_gamma: f32,
    pub filter_levels_out_black: f32,
    pub filter_levels_out_white: f32,
    pub show_bookmark_limit_dialog: bool,
    pub bookmark_limit_title: String,
    pub bookmark_limit_message: String,
    pub app_version_label: String,
    pub show_about_dialog: bool,
    pub about_title: String,
    pub about_lines: Vec<String>,
    pub available_languages: Vec<LangMeta>,
    pub current_locale: String,
    pub show_loading_spinner: bool,
    // File Association
    pub show_file_association_window: bool,
    pub file_association_states: Vec<(String, String, bool)>, // (ext, description, is_associated)
}
