use crate::types::LayoutMode;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct LangMeta {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangInfo {
    pub title: String,
    pub filename: String,
    pub filesize: String,
    pub info: String,
    pub icc_profile: String,
    pub ram_cache_format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangExif {
    pub title: String,
    pub camera: String,
    pub lens: String,
    pub f_stop: String,
    pub shutter_speed: String,
    pub iso: String,
    pub datetime: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangViewMode {
    pub title: String,
    pub fit_window: String,
    pub fit_width: String,
    pub fit_height: String,
    pub zoom: String,
    pub single: String,
    pub ltr: String,
    pub rtl: String,
    pub one_page_offset: String,
    pub view_reset: String,
    pub slideshow: String,
    pub sec: String,
    pub slideshow_start: String,
    pub slideshow_stop: String,
    pub slideshow_repeat_format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangFilter {
    pub title: String,
    pub color: String,
    pub bright: String,
    pub contrast: String,
    pub gamma: String,
    pub exposure: String,
    pub fsr_enabled: String,
    pub fsr_sharpness: String,
    pub median: String,
    pub median_strength: String,
    pub median_stride: String,
    pub detail: String,
    pub blur: String,
    pub unsharp: String,
    pub unsharp_threshold: String,
    pub levels: String,
    pub levels_in: String,
    pub levels_out: String,
    pub levels_black: String,
    pub levels_mid: String,
    pub levels_white: String,
    pub bypass: String,
    pub reset: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangPreference {
    pub title: String,
    pub ui_auto_hide: String,
    pub no_auto_hide: String,
    pub prefetch_count: String,
    #[serde(default = "default_prefetch_format")]
    pub prefetch_format: String,
    pub webtoon_scroll_speed: String,
    pub ram_cache_setting: String,
    pub vram_cache_setting: String,
    pub archive_sorting: String,
    pub sort_mixed: String,
    pub sort_folders_first: String,
    pub remember_document_position: String,
    pub language: String,
    pub theme: String,
    pub theme_auto: String,
    pub theme_dark: String,
    pub theme_light: String,
    pub file_association: String,
    pub file_association_button: String,
    pub file_association_delete_button: String,
    pub context_menu_add_button: String,
    pub context_menu_delete_button: String,
    pub context_menu_item_text: String,
    #[serde(default)]
    pub context_menu_section_label: String,
    #[serde(default)]
    pub start_menu_add_button: String,
    #[serde(default)]
    pub start_menu_delete_button: String,
    #[serde(default)]
    pub start_menu_section_label: String,
    pub file_association_description: String,
    pub instance_mode: String,
    pub instance_restart_notice: String,
    pub instance_single: String,
    pub instance_multi: String,
    pub config_storage_location: String,
    pub config_storage_restart_notice: String,
    pub storage_app_dir: String,
    pub storage_system_config: String,
}

fn default_prefetch_format() -> String {
    "{0} pages".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangBookmark {
    pub archive: String,
    pub file: String,
    pub auto: String,
    pub manual: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangFileAssociation {
    pub window_title: String,
    pub window_subtitle: String,
    pub select_all: String,
    pub deselect_all: String,
    pub apply: String,
    pub cancel: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangShortcuts {
    pub fullscreen: String,
    pub fit_mode: String,
    pub zoom: String,
    pub layout: String,
    pub offset: String,
    pub archive_move: String,
    pub save_bookmark: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangCommand {
    pub navigate_previous: String,
    pub navigate_next: String,
    pub navigate_first: String,
    pub navigate_last: String,
    pub navigate_previous_archive: String,
    pub navigate_next_archive: String,
    pub set_fit_screen: String,
    pub set_fit_width: String,
    pub set_fit_height: String,
    pub cycle_layout_mode: String,
    pub toggle_first_page_offset: String,
    pub zoom_in_step: String,
    pub zoom_out_step: String,
    pub rotate_ccw: String,
    pub rotate_cw: String,
    pub save_manual_bookmark: String,
    pub reset_view: String,
    pub toggle_fullscreen: String,
    pub toggle_ui_windows: String,
    pub adjust_offset_up: String,
    pub adjust_offset_down: String,
    pub adjust_offset_left: String,
    pub adjust_offset_right: String,
    pub open_file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangUi {
    pub settings: String,
    pub favorites: String,
    pub label_shortcuts: String,
    pub fit_screen: String,
    pub fit_width: String,
    pub fit_height: String,
    pub first_page_offset_on: String,
    pub first_page_offset_off: String,
    pub bookmark_limit_title: String,
    pub bookmark_limit_message: String,
    pub about_title: String,
    pub about_description: String,
    pub layout_single: String,
    pub layout_dual_ltr: String,
    pub layout_dual_rtl: String,
    pub layout_vertical_scroll: String,

    pub info: LangInfo,
    pub exif: LangExif,
    pub view_mode: LangViewMode,
    pub filter: LangFilter,
    pub preference: LangPreference,
    pub bookmark: LangBookmark,
    pub file_association: LangFileAssociation,
    #[serde(rename = "shortcuts")]
    pub shortcuts_list: LangShortcuts,
}

#[derive(Debug, Clone, Deserialize)]
struct LangMetaOnly {
    pub meta: LangMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangToast {
    pub fullscreen_entered: String,
    pub fullscreen_exited: String,
    pub zoom_format: String,
    pub moved_to_format: String,
    pub bookmark_saved: String,
    pub ui_windows_shown: String,
    pub ui_windows_hidden: String,
    pub load_failed: String,
    pub nav_looped_first: String,
    pub nav_looped_last: String,
    pub nav_skipped_empty: String,
    pub nav_no_valid_targets: String,
    pub rotation_not_supported_in_dual: String,
    pub webtoon_not_supported_for_animated: String,
    pub slideshow_started: String,
    pub slideshow_stopped: String,
    pub file_association_deleted: String,
    pub context_menu_registered: String,
    pub context_menu_unregistered: String,
    pub file_association_failed: String,
    pub context_menu_register_failed: String,
    pub context_menu_unregister_failed: String,
    pub directory_context_menu_register_failed: String,
    pub directory_context_menu_unregister_failed: String,
    #[serde(default)]
    pub start_menu_registered: String,
    #[serde(default)]
    pub start_menu_unregistered: String,
    #[serde(default)]
    pub start_menu_register_failed: String,
    #[serde(default)]
    pub start_menu_unregister_failed: String,
    pub file_association_applied: String,
    pub file_association_apply_failed: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangData {
    pub meta: LangMeta,
    pub ui: LangUi,
    pub toast: LangToast,
    pub command: LangCommand,
}

#[derive(Debug, Clone)]
pub struct Localizer {
    data: LangData,
    available_languages: Vec<LangMeta>,
}

impl Localizer {
    pub fn load_available_languages() -> Vec<LangMeta> {
        let mut languages = Vec::new();
        let lang_dir = Path::new("assets/lang");
        if let Ok(entries) = std::fs::read_dir(lang_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("json")
                    && let Ok(content) = std::fs::read_to_string(entry.path())
                {
                    if let Ok(data) = serde_json::from_str::<LangMetaOnly>(&content) {
                        languages.push(data.meta);
                    }
                }
            }
        }

        if languages.is_empty() {
            languages.push(LangMeta {
                name: "English".to_string(),
                code: "en".to_string(),
            });
        }

        languages.sort_by(|a, b| a.name.cmp(&b.name));
        languages
    }

    pub fn new(lang_code: &str) -> Self {
        let available = Self::load_available_languages();
        let lang_path = format!("assets/lang/{}.json", lang_code);

        let data = if let Ok(content) = std::fs::read_to_string(&lang_path) {
            serde_json::from_str::<LangData>(&content).unwrap_or_else(|_| Self::default_data())
        } else {
            let en_path = "assets/lang/en.json";
            if let Ok(content) = std::fs::read_to_string(en_path) {
                serde_json::from_str::<LangData>(&content).unwrap_or_else(|_| Self::default_data())
            } else {
                Self::default_data()
            }
        };

        Self {
            data,
            available_languages: available,
        }
    }

    fn default_data() -> LangData {
        LangData {
            meta: LangMeta {
                name: "English".to_string(),
                code: "en".to_string(),
            },
            ui: LangUi {
                settings: "Settings".to_string(),
                favorites: "Favorites".to_string(),
                label_shortcuts: "Shortcuts".to_string(),
                fit_screen: "Fit Screen".to_string(),
                fit_width: "Fit Width".to_string(),
                fit_height: "Fit Height".to_string(),
                first_page_offset_on: "First Page Offset: On".to_string(),
                first_page_offset_off: "First Page Offset: Off".to_string(),
                bookmark_limit_title: "Bookmark Limit Reached".to_string(),
                bookmark_limit_message: "Manual bookmarks are limited to 15 entries.".to_string(),
                about_title: "About".to_string(),
                about_description: "GPU-accelerated image viewer".to_string(),
                layout_single: "Single View".to_string(),
                layout_dual_ltr: "Dual View (LTR)".to_string(),
                layout_dual_rtl: "Dual View (RTL)".to_string(),
                layout_vertical_scroll: "Webtoon Mode".to_string(),
                info: LangInfo {
                    title: "Info".to_string(),
                    filename: "Filename".to_string(),
                    filesize: "File Size".to_string(),
                    info: "Dimensions".to_string(),
                    icc_profile: "ICC Profile".to_string(),
                    ram_cache_format: "RAM Cache {0}MB / {1}MB".to_string(),
                },
                exif: LangExif {
                    title: "EXIF".to_string(),
                    camera: "Camera".to_string(),
                    lens: "Lens".to_string(),
                    f_stop: "F-Stop".to_string(),
                    shutter_speed: "Shutter Speed".to_string(),
                    iso: "ISO".to_string(),
                    datetime: "Date & Time".to_string(),
                },
                view_mode: LangViewMode {
                    title: "View Mode".to_string(),
                    fit_window: "Fit Window".to_string(),
                    fit_width: "Fit Width".to_string(),
                    fit_height: "Fit Height".to_string(),
                    zoom: "Zoom".to_string(),
                    single: "Single".to_string(),
                    ltr: "LTR".to_string(),
                    rtl: "RTL".to_string(),
                    one_page_offset: "1-Page Offset".to_string(),
                    view_reset: "Reset View".to_string(),
                    slideshow: "Slideshow".to_string(),
                    sec: "sec".to_string(),
                    slideshow_start: "Start".to_string(),
                    slideshow_stop: "Stop".to_string(),
                    slideshow_repeat_format: "Repeat animation {0} times".to_string(),
                },
                filter: LangFilter {
                    title: "Filter".to_string(),
                    color: "Color".to_string(),
                    bright: "Brightness".to_string(),
                    contrast: "Contrast".to_string(),
                    gamma: "Gamma".to_string(),
                    exposure: "Exposure".to_string(),
                    fsr_enabled: "AMD FidelityFX™ FSR 1.0".to_string(),
                    fsr_sharpness: "FSR Sharpness".to_string(),
                    median: "Median Filter".to_string(),
                    median_strength: "Strength".to_string(),
                    median_stride: "Stride".to_string(),
                    detail: "Detail".to_string(),
                    blur: "Gaussian Blur".to_string(),
                    unsharp: "Unsharp Mask".to_string(),
                    unsharp_threshold: "Threshold".to_string(),
                    levels: "Levels".to_string(),
                    levels_in: "Input Levels".to_string(),
                    levels_out: "Output Levels".to_string(),
                    levels_black: "Blacks".to_string(),
                    levels_mid: "Midtones".to_string(),
                    levels_white: "Whites".to_string(),
                    bypass: "Bypass".to_string(),
                    reset: "Reset".to_string(),
                },
                preference: LangPreference {
                    title: "Preference".to_string(),
                    ui_auto_hide: "UI Auto-Hide".to_string(),
                    no_auto_hide: "No Auto-Hide".to_string(),
                    prefetch_count: "Prefetch Count".to_string(),
                    prefetch_format: "{0} pages".to_string(),
                    webtoon_scroll_speed: "Webtoon Scroll Speed".to_string(),
                    ram_cache_setting: "RAM Cache Setting ({0}MB / {1}MB)".to_string(),
                    vram_cache_setting: "VRAM Cache Setting ({0}MB / {1}MB)".to_string(),
                    archive_sorting: "Archive Sorting".to_string(),
                    sort_mixed: "Mixed Natural Sort".to_string(),
                    sort_folders_first: "Folders First".to_string(),
                    remember_document_position: "Remember position per archive".to_string(),
                    language: "Language".to_string(),
                    theme: "Theme".to_string(),
                    theme_auto: "Auto".to_string(),
                    theme_dark: "Dark".to_string(),
                    theme_light: "Light".to_string(),
                    file_association: "File Association".to_string(),
                    file_association_button: "File Association Settings...".to_string(),
                    file_association_delete_button: "Delete All".to_string(),
                    context_menu_add_button: "Add".to_string(),
                    context_menu_delete_button: "Delete".to_string(),
                    context_menu_item_text: "Open with HinaView".to_string(),
                    context_menu_section_label: "Register in Explorer right-click menu".to_string(),
                    start_menu_add_button: "Add".to_string(),
                    start_menu_delete_button: "Delete".to_string(),
                    start_menu_section_label: "Register in Windows Start Menu".to_string(),
                    file_association_description: "Set HinaView as default image viewer"
                        .to_string(),
                    instance_mode: "Window Mode".to_string(),
                    instance_restart_notice: "(Applied after restart)".to_string(),
                    instance_single: "Single Window Mode (Recommended)".to_string(),
                    instance_multi: "Multi Window Mode (Open in New Window)".to_string(),
                    config_storage_location: "Config Storage Location".to_string(),
                    config_storage_restart_notice: "(Applied after restart)".to_string(),
                    storage_app_dir: "Application Directory (Recommended)".to_string(),
                    storage_system_config: "System Config Directory".to_string(),
                },
                bookmark: LangBookmark {
                    archive: "Archive".to_string(),
                    file: "File".to_string(),
                    auto: "Auto".to_string(),
                    manual: "Manual".to_string(),
                },
                file_association: LangFileAssociation {
                    window_title: "File Association Settings".to_string(),
                    window_subtitle: "Select file types to set HinaView as default viewer."
                        .to_string(),
                    select_all: "Select All".to_string(),
                    deselect_all: "Deselect All".to_string(),
                    apply: "Apply".to_string(),
                    cancel: "Cancel".to_string(),
                },
                shortcuts_list: LangShortcuts {
                    fullscreen: "f : Fullscreen".to_string(),
                    fit_mode: "' / w / h : Fit Mode".to_string(),
                    zoom: "+ / - : Zoom Step".to_string(),
                    layout: "d : Single -> LTR -> RTL".to_string(),
                    offset: "o : First Page Offset".to_string(),
                    archive_move: "[ / ] : Archive Move".to_string(),
                    save_bookmark: "b : Save Bookmark".to_string(),
                },
            },
            toast: LangToast {
                fullscreen_entered: "Entered Fullscreen Mode".to_string(),
                fullscreen_exited: "Exited Fullscreen Mode".to_string(),
                zoom_format: "Zoom {0}%".to_string(),
                moved_to_format: "Moved: {0}".to_string(),
                bookmark_saved: "Bookmark saved".to_string(),
                ui_windows_shown: "UI Shown".to_string(),
                ui_windows_hidden: "UI Hidden".to_string(),
                load_failed: "Failed to load image(s)".to_string(),
                nav_looped_first: "Returned to the first item".to_string(),
                nav_looped_last: "Moved to the last item".to_string(),
                nav_skipped_empty: "Skipped empty folder \"{0}\"".to_string(),
                nav_no_valid_targets: "No valid directories found to move to".to_string(),
                rotation_not_supported_in_dual: "Rotation is not supported in dual-page view"
                    .to_string(),
                webtoon_not_supported_for_animated:
                    "Webtoon mode is not supported for animated images".to_string(),
                slideshow_started: "Slideshow Started".to_string(),
                slideshow_stopped: "Slideshow Stopped".to_string(),
                file_association_deleted: "All file associations deleted.".to_string(),
                context_menu_registered: "Context menu registered.".to_string(),
                context_menu_unregistered: "Context menu unregistered.".to_string(),
                file_association_failed: "Failed to delete file associations: {0}".to_string(),
                context_menu_register_failed: "Failed to register context menu: {0}".to_string(),
                context_menu_unregister_failed: "Failed to unregister context menu: {0}"
                    .to_string(),
                directory_context_menu_register_failed:
                    "Failed to register directory context menu: {0}".to_string(),
                directory_context_menu_unregister_failed:
                    "Failed to unregister directory context menu: {0}".to_string(),
                start_menu_registered: "Start menu shortcut registered.".to_string(),
                start_menu_unregistered: "Start menu shortcut deleted.".to_string(),
                start_menu_register_failed: "Failed to register start menu shortcut: {0}"
                    .to_string(),
                start_menu_unregister_failed: "Failed to delete start menu shortcut: {0}"
                    .to_string(),
                file_association_applied: "File associations applied.".to_string(),
                file_association_apply_failed: "Failed to apply file associations: {0}".to_string(),
            },
            command: LangCommand {
                navigate_previous: "Previous".to_string(),
                navigate_next: "Next".to_string(),
                navigate_first: "First Page".to_string(),
                navigate_last: "Last Page".to_string(),
                navigate_previous_archive: "Previous Archive".to_string(),
                navigate_next_archive: "Next Archive".to_string(),
                set_fit_screen: "Fit Screen".to_string(),
                set_fit_width: "Fit Width".to_string(),
                set_fit_height: "Fit Height".to_string(),
                cycle_layout_mode: "Cycle Layout".to_string(),
                toggle_first_page_offset: "Toggle 1-Page Offset".to_string(),
                zoom_in_step: "Zoom In".to_string(),
                zoom_out_step: "Zoom Out".to_string(),
                rotate_ccw: "Rotate CCW".to_string(),
                rotate_cw: "Rotate CW".to_string(),
                save_manual_bookmark: "Save Bookmark".to_string(),
                reset_view: "Reset View".to_string(),
                open_file: "Open File".to_string(),
                toggle_fullscreen: "Toggle Fullscreen".to_string(),
                toggle_ui_windows: "Toggle UI".to_string(),
                adjust_offset_up: "Pan Up".to_string(),
                adjust_offset_down: "Pan Down".to_string(),
                adjust_offset_left: "Pan Left".to_string(),
                adjust_offset_right: "Pan Right".to_string(),
            },
        }
    }

    pub fn available_languages(&self) -> &[LangMeta] {
        &self.available_languages
    }

    pub fn current_code(&self) -> &str {
        &self.data.meta.code
    }

    pub fn fullscreen_entered(&self) -> &str {
        &self.data.toast.fullscreen_entered
    }
    pub fn fullscreen_exited(&self) -> &str {
        &self.data.toast.fullscreen_exited
    }
    pub fn fit_screen(&self) -> &str {
        &self.data.ui.fit_screen
    }
    pub fn fit_width(&self) -> &str {
        &self.data.ui.fit_width
    }
    pub fn fit_height(&self) -> &str {
        &self.data.ui.fit_height
    }

    pub fn zoom_percent(&self, zoom: f32) -> String {
        self.data
            .toast
            .zoom_format
            .replace("{0}", &format!("{:.1}", zoom * 100.0))
    }

    pub fn moved_to(&self, name: &str) -> String {
        self.data.toast.moved_to_format.replace("{0}", name)
    }

    pub fn layout_mode_label(&self, mode: LayoutMode) -> &str {
        match mode {
            LayoutMode::Single => &self.data.ui.layout_single,
            LayoutMode::Dual { rtl: false, .. } => &self.data.ui.layout_dual_ltr,
            LayoutMode::Dual { rtl: true, .. } => &self.data.ui.layout_dual_rtl,
            LayoutMode::VerticalScroll => &self.data.ui.layout_vertical_scroll,
        }
    }

    pub fn label_settings(&self) -> &str {
        &self.data.ui.settings
    }
    pub fn label_favorites(&self) -> &str {
        &self.data.ui.favorites
    }
    pub fn label_shortcuts(&self) -> &str {
        &self.data.ui.label_shortcuts
    }
    pub fn first_page_offset_on(&self) -> &str {
        &self.data.ui.first_page_offset_on
    }
    pub fn first_page_offset_off(&self) -> &str {
        &self.data.ui.first_page_offset_off
    }

    pub fn bookmark_limit_title(&self) -> &str {
        &self.data.ui.bookmark_limit_title
    }
    pub fn bookmark_limit_message(&self) -> &str {
        &self.data.ui.bookmark_limit_message
    }
    pub fn bookmark_saved(&self) -> &str {
        &self.data.toast.bookmark_saved
    }
    pub fn load_failed(&self) -> &str {
        &self.data.toast.load_failed
    }

    pub fn nav_looped_first(&self) -> &str {
        &self.data.toast.nav_looped_first
    }

    pub fn nav_looped_last(&self) -> &str {
        &self.data.toast.nav_looped_last
    }

    pub fn nav_skipped_empty(&self, name: &str) -> String {
        self.data.toast.nav_skipped_empty.replace("{0}", name)
    }

    pub fn nav_no_valid_targets(&self) -> &str {
        &self.data.toast.nav_no_valid_targets
    }

    pub fn rotation_not_supported_in_dual(&self) -> &str {
        &self.data.toast.rotation_not_supported_in_dual
    }

    pub fn webtoon_not_supported_for_animated(&self) -> &str {
        &self.data.toast.webtoon_not_supported_for_animated
    }

    pub fn ui_windows_shown(&self) -> &str {
        &self.data.toast.ui_windows_shown
    }

    pub fn ui_windows_hidden(&self) -> &str {
        &self.data.toast.ui_windows_hidden
    }

    pub fn slideshow_started(&self) -> &str {
        &self.data.toast.slideshow_started
    }

    pub fn slideshow_stopped(&self) -> &str {
        &self.data.toast.slideshow_stopped
    }

    pub fn file_association_deleted(&self) -> &str {
        &self.data.toast.file_association_deleted
    }

    pub fn context_menu_registered(&self) -> &str {
        &self.data.toast.context_menu_registered
    }

    pub fn context_menu_unregistered(&self) -> &str {
        &self.data.toast.context_menu_unregistered
    }

    pub fn start_menu_registered(&self) -> &str {
        if self.data.toast.start_menu_registered.is_empty() {
            "Start menu shortcut registered."
        } else {
            &self.data.toast.start_menu_registered
        }
    }

    pub fn start_menu_unregistered(&self) -> &str {
        if self.data.toast.start_menu_unregistered.is_empty() {
            "Start menu shortcut deleted."
        } else {
            &self.data.toast.start_menu_unregistered
        }
    }

    pub fn file_association_failed(&self, error: &str) -> String {
        self.data
            .toast
            .file_association_failed
            .replace("{0}", error)
    }

    pub fn context_menu_register_failed(&self, error: &str) -> String {
        self.data
            .toast
            .context_menu_register_failed
            .replace("{0}", error)
    }

    pub fn context_menu_unregister_failed(&self, error: &str) -> String {
        self.data
            .toast
            .context_menu_unregister_failed
            .replace("{0}", error)
    }

    pub fn directory_context_menu_register_failed(&self, error: &str) -> String {
        self.data
            .toast
            .directory_context_menu_register_failed
            .replace("{0}", error)
    }

    pub fn directory_context_menu_unregister_failed(&self, error: &str) -> String {
        self.data
            .toast
            .directory_context_menu_unregister_failed
            .replace("{0}", error)
    }

    pub fn start_menu_register_failed(&self, error: &str) -> String {
        let template = if self.data.toast.start_menu_register_failed.is_empty() {
            "Failed to register start menu shortcut: {0}"
        } else {
            &self.data.toast.start_menu_register_failed
        };
        template.replace("{0}", error)
    }

    pub fn start_menu_unregister_failed(&self, error: &str) -> String {
        let template = if self.data.toast.start_menu_unregister_failed.is_empty() {
            "Failed to delete start menu shortcut: {0}"
        } else {
            &self.data.toast.start_menu_unregister_failed
        };
        template.replace("{0}", error)
    }

    pub fn file_association_applied(&self) -> &str {
        &self.data.toast.file_association_applied
    }

    pub fn file_association_apply_failed(&self, error: &str) -> String {
        self.data
            .toast
            .file_association_apply_failed
            .replace("{0}", error)
    }

    pub fn about_title(&self) -> &str {
        &self.data.ui.about_title
    }
    pub fn about_description(&self) -> &str {
        &self.data.ui.about_description
    }

    pub fn info(&self) -> &LangInfo {
        &self.data.ui.info
    }
    pub fn exif(&self) -> &LangExif {
        &self.data.ui.exif
    }
    pub fn view_mode(&self) -> &LangViewMode {
        &self.data.ui.view_mode
    }
    pub fn filter(&self) -> &LangFilter {
        &self.data.ui.filter
    }
    pub fn preference(&self) -> &LangPreference {
        &self.data.ui.preference
    }
    pub fn file_association(&self) -> &LangFileAssociation {
        &self.data.ui.file_association
    }
    pub fn bookmark(&self) -> &LangBookmark {
        &self.data.ui.bookmark
    }
    pub fn shortcuts_list(&self) -> &LangShortcuts {
        &self.data.ui.shortcuts_list
    }

    pub fn slideshow_repeat_label(&self, count: u32) -> String {
        self.data
            .ui
            .view_mode
            .slideshow_repeat_format
            .replace("{0}", &count.to_string())
    }

    pub fn ram_cache_display(&self, current: usize, max: usize) -> String {
        self.data
            .ui
            .info
            .ram_cache_format
            .replace("{0}", &current.to_string())
            .replace("{1}", &max.to_string())
    }

    pub fn ram_cache_setting_label(&self, current: usize, max: usize) -> String {
        self.data
            .ui
            .preference
            .ram_cache_setting
            .replace("{0}", &current.to_string())
            .replace("{1}", &max.to_string())
    }

    pub fn vram_cache_setting_label(&self, current: usize, max: usize) -> String {
        self.data
            .ui
            .preference
            .vram_cache_setting
            .replace("{0}", &current.to_string())
            .replace("{1}", &max.to_string())
    }

    pub fn command_name(&self, cmd_key: &str) -> &str {
        match cmd_key {
            "navigate_previous" => &self.data.command.navigate_previous,
            "navigate_next" => &self.data.command.navigate_next,
            "navigate_first" => &self.data.command.navigate_first,
            "navigate_last" => &self.data.command.navigate_last,
            "navigate_previous_archive" => &self.data.command.navigate_previous_archive,
            "navigate_next_archive" => &self.data.command.navigate_next_archive,
            "set_fit_screen" => &self.data.command.set_fit_screen,
            "set_fit_width" => &self.data.command.set_fit_width,
            "set_fit_height" => &self.data.command.set_fit_height,
            "cycle_layout_mode" => &self.data.command.cycle_layout_mode,
            "toggle_first_page_offset" => &self.data.command.toggle_first_page_offset,
            "zoom_in_step" => &self.data.command.zoom_in_step,
            "zoom_out_step" => &self.data.command.zoom_out_step,
            "rotate_ccw" => &self.data.command.rotate_ccw,
            "rotate_cw" => &self.data.command.rotate_cw,
            "save_manual_bookmark" => &self.data.command.save_manual_bookmark,
            "reset_view" => &self.data.command.reset_view,
            "toggle_fullscreen" => &self.data.command.toggle_fullscreen,
            "toggle_ui_windows" => &self.data.command.toggle_ui_windows,
            "adjust_offset_up" => &self.data.command.adjust_offset_up,
            "adjust_offset_down" => &self.data.command.adjust_offset_down,
            "adjust_offset_left" => &self.data.command.adjust_offset_left,
            "adjust_offset_right" => &self.data.command.adjust_offset_right,
            "open_file" => &self.data.command.open_file,
            _ => "",
        }
    }
}
