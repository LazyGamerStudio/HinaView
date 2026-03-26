use crate::bookmark::BookmarkService;
use crate::cache::TextureManager;
use crate::color_management::ColorManagementController;
use crate::i18n::Localizer;
use crate::input::{AppCommand, InputCommand, RuntimeCommand, get_shortcut_string};
use crate::pipeline::DecodeScheduler;
use crate::settings::SettingsState;
use crate::slideshow::SlideshowController;
use crate::ui::{UiBookmarkRow, UiFitMode, UiLayoutMode, UiSnapshot};
use crate::util::formats::format_file_size;
use crate::view::NavigationController;

pub struct UiSnapshotContext<'a> {
    pub nav: &'a NavigationController,
    pub settings: &'a SettingsState,
    pub localizer: &'a Localizer,
    pub bookmark_service: &'a BookmarkService,
    pub scheduler: &'a DecodeScheduler,
    pub texture_manager: &'a TextureManager,
    pub color_management: &'a ColorManagementController,
    pub slideshow: &'a SlideshowController,
    pub show_bookmark_limit_dialog: bool,
    pub show_bookmarks: bool,
    pub show_about_dialog: bool,
    pub ui_windows_visible: bool,
    pub max_gpu_setting_mb: usize,
    pub dual_first_page_offset: bool,
    pub accent_color: Option<egui::Color32>,
    pub show_loading_spinner: bool,
    pub file_association_states: Vec<(String, String, bool)>,
    pub show_file_association_window: bool,
}

pub fn build_ui_snapshot(ctx: UiSnapshotContext) -> UiSnapshot {
    let (
        archive_name_value,
        file_name_value,
        file_size_value,
        info_value,
        icc_profile_value,
        exif_camera_value,
        exif_lens_value,
        exif_f_stop_value,
        exif_shutter_value,
        exif_iso_value,
        exif_datetime_value,
    ) = if let (Some(doc), Some(page_id)) = (ctx.nav.document.as_ref(), ctx.nav.current_page) {
        if let Some(page) = doc.pages.get(page_id) {
            let archive = doc
                .path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("-")
                .to_string();
            let file = page.name.clone();
            let file_size = page
                .file_size_bytes
                .map(format_file_size)
                .unwrap_or_else(|| "-".to_string());
            let info = format!("{} | {}x{}", page.format_label, page.width, page.height);
            let src_icc = page
                .icc_profile
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let icc = format!(
                "{} | {}",
                src_icc,
                ctx.color_management.display_profile_name()
            );
            let exif_camera = page.exif_camera.clone().unwrap_or_default();
            let exif_lens = page.exif_lens.clone().unwrap_or_default();
            let exif_f_stop = page.exif_f_stop.clone().unwrap_or_default();
            let exif_shutter = page.exif_shutter.clone().unwrap_or_default();
            let exif_iso = page.exif_iso.clone().unwrap_or_default();
            let exif_datetime = page.exif_datetime.clone().unwrap_or_default();
            (
                archive,
                file,
                file_size,
                info,
                icc,
                exif_camera,
                exif_lens,
                exif_f_stop,
                exif_shutter,
                exif_iso,
                exif_datetime,
            )
        } else {
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            )
        }
    } else {
        (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        )
    };

    let commands_to_show = vec![
        InputCommand::App(AppCommand::OpenFile),
        InputCommand::App(AppCommand::NavigatePrevious),
        InputCommand::App(AppCommand::NavigateNext),
        InputCommand::App(AppCommand::NavigateFirst),
        InputCommand::App(AppCommand::NavigateLast),
        InputCommand::Runtime(RuntimeCommand::ToggleFullscreen),
        InputCommand::Runtime(RuntimeCommand::ToggleUiWindows),
        InputCommand::App(AppCommand::SetFitScreen),
        InputCommand::App(AppCommand::SetFitWidth),
        InputCommand::App(AppCommand::SetFitHeight),
        InputCommand::App(AppCommand::CycleLayoutMode),
        InputCommand::App(AppCommand::ToggleFirstPageOffset),
        InputCommand::App(AppCommand::ZoomInStep),
        InputCommand::App(AppCommand::ZoomOutStep),
        InputCommand::App(AppCommand::RotateCCW),
        InputCommand::App(AppCommand::RotateCW),
        InputCommand::App(AppCommand::NavigatePreviousArchive),
        InputCommand::App(AppCommand::NavigateNextArchive),
        InputCommand::App(AppCommand::SaveManualBookmark),
        InputCommand::App(AppCommand::AdjustOffset(0.0, 1.0)),
        InputCommand::App(AppCommand::AdjustOffset(0.0, -1.0)),
        InputCommand::App(AppCommand::AdjustOffset(-1.0, 0.0)),
        InputCommand::App(AppCommand::AdjustOffset(1.0, 0.0)),
        InputCommand::App(AppCommand::ResetView),
    ];

    let mut shortcuts_lines = Vec::new();
    for cmd in commands_to_show {
        let keys = get_shortcut_string(cmd);
        let name = ctx.localizer.command_name(cmd.to_localized_key());
        if !keys.is_empty() {
            shortcuts_lines.push(format!("{} : {}", keys, name));
        }
    }

    let bookmark_rows = ctx
        .bookmark_service
        .entries()
        .iter()
        .map(|e| UiBookmarkRow {
            id: e.id,
            source_label: match e.source {
                crate::bookmark::BookmarkSource::AutoRecent => {
                    ctx.localizer.bookmark().auto.clone()
                }
                crate::bookmark::BookmarkSource::Manual => ctx.localizer.bookmark().manual.clone(),
            },
            archive_name: e.archive_name.clone(),
            page_name: e.page_name.clone(),
        })
        .collect();

    let ui_auto_hide_label = if ctx.settings.ui_auto_hide_sec >= 11 {
        ctx.localizer.preference().no_auto_hide.clone()
    } else {
        format!(
            "{}{}",
            ctx.settings.ui_auto_hide_sec,
            ctx.localizer.view_mode().sec
        )
    };

    UiSnapshot {
        settings_title: ctx.localizer.label_settings().to_string(),
        favorites_title: ctx.localizer.label_favorites().to_string(),
        shortcuts_title: ctx.localizer.label_shortcuts().to_string(),
        lang_info: ctx.localizer.info().clone(),
        lang_exif: ctx.localizer.exif().clone(),
        lang_view_mode: ctx.localizer.view_mode().clone(),
        lang_filter: ctx.localizer.filter().clone(),
        lang_preference: ctx.localizer.preference().clone(),
        lang_file_association: ctx.localizer.file_association().clone(),
        lang_bookmark: ctx.localizer.bookmark().clone(),

        archive_name_value,
        file_name_value,
        file_size_value,
        info_value,
        icc_profile_value,
        exif_camera_value,
        exif_lens_value,
        exif_f_stop_value,
        exif_shutter_value,
        exif_iso_value,
        exif_datetime_value,
        ram_cache_display: ctx.localizer.ram_cache_display(
            ctx.scheduler.cpu_cache_memory_mb(),
            ctx.scheduler.cpu_cache_max_mb(),
        ),

        info_section_open: ctx.settings.sections_open.info,
        exif_section_open: ctx.settings.sections_open.exif,
        view_mode_section_open: ctx.settings.sections_open.view_mode,
        filter_section_open: ctx.settings.sections_open.filter,
        preference_section_open: ctx.settings.sections_open.preference,
        ui_windows_visible: ctx.ui_windows_visible,
        fit_mode: match ctx.settings.fit_mode {
            crate::settings::model::FitModeSetting::FitScreen => UiFitMode::FitScreen,
            crate::settings::model::FitModeSetting::FitWidth => UiFitMode::FitWidth,
            crate::settings::model::FitModeSetting::FitHeight => UiFitMode::FitHeight,
            crate::settings::model::FitModeSetting::Zoom => UiFitMode::Zoom,
        },
        layout_mode: match ctx.nav.view.layout_mode {
            crate::types::LayoutMode::Single => UiLayoutMode::Single,
            crate::types::LayoutMode::Dual { rtl: false, .. } => UiLayoutMode::DualLtr,
            crate::types::LayoutMode::Dual { rtl: true, .. } => UiLayoutMode::DualRtl,
            crate::types::LayoutMode::VerticalScroll => UiLayoutMode::VerticalScroll,
        },
        layout_vertical_scroll_label: ctx
            .localizer
            .layout_mode_label(crate::types::LayoutMode::VerticalScroll)
            .to_string(),
        first_page_offset: ctx.dual_first_page_offset,
        slideshow_enabled: ctx.slideshow.enabled(),
        slideshow_interval_sec: ctx.slideshow.interval_sec(),
        slideshow_repeat_label: {
            let repeat_count = ctx.slideshow.interval_sec().min(5);
            format!(
                "{} | {}",
                ctx.localizer.view_mode().slideshow,
                ctx.localizer.slideshow_repeat_label(repeat_count)
            )
        },
        ui_auto_hide_sec: ctx.settings.ui_auto_hide_sec,
        ui_auto_hide_label,
        prefetch_count: ctx.settings.prefetch_count,
        cpu_cache_current_mb: ctx.scheduler.cpu_cache_memory_mb(),
        cpu_cache_max_mb: ctx.scheduler.cpu_cache_max_mb(),
        cpu_cache_setting_mb: ctx.settings.cpu_cache_mb,
        ram_cache_setting_label: ctx.localizer.ram_cache_setting_label(
            ctx.scheduler.cpu_cache_memory_mb(),
            ctx.scheduler.cpu_cache_max_mb(),
        ),
        gpu_cache_allowed_max_mb: ctx.max_gpu_setting_mb,
        gpu_cache_setting_mb: ctx.settings.gpu_cache_mb,
        vram_cache_setting_label: ctx.localizer.vram_cache_setting_label(
            ctx.texture_manager.gpu_cache_memory_mb(),
            ctx.texture_manager.gpu_cache_max_mb(),
        ),
        shortcuts_lines,
        bookmark_rows,
        settings_window_collapsed: ctx.settings.settings_window_collapsed,
        shortcuts_window_collapsed: ctx.settings.shortcuts_window_collapsed,
        bookmark_drawer_open: ctx.show_bookmarks,
        archive_sorting_mode: match ctx.settings.archive_sorting_mode {
            crate::settings::model::ArchiveSortingMode::Mixed => {
                crate::ui::UiArchiveSortingMode::Mixed
            }
            crate::settings::model::ArchiveSortingMode::FoldersFirst => {
                crate::ui::UiArchiveSortingMode::FoldersFirst
            }
        },
        remember_document_position: ctx.settings.remember_document_position,
        webtoon_scroll_speed_px_per_sec: ctx.settings.webtoon_scroll_speed_px_per_sec,
        theme_mode: match ctx.settings.theme_mode {
            crate::settings::model::ThemeModeSetting::Auto => crate::ui::UiThemeMode::Auto,
            crate::settings::model::ThemeModeSetting::Dark => crate::ui::UiThemeMode::Dark,
            crate::settings::model::ThemeModeSetting::Light => crate::ui::UiThemeMode::Light,
        },
        single_instance: ctx.settings.single_instance,
        config_storage_location: ctx.settings.config_storage_location,
        ui_opacity: if ctx.ui_windows_visible { 1.0 } else { 0.0 },
        accent_color: ctx.accent_color,
        filter_bypass_color: ctx.settings.filters.bypass_color,
        filter_bypass_median: ctx.settings.filters.bypass_median,
        filter_bypass_fsr: ctx.settings.filters.bypass_fsr,
        filter_bypass_detail: ctx.settings.filters.bypass_detail,
        filter_bypass_levels: ctx.settings.filters.bypass_levels,
        filter_bright: ctx.settings.filters.bright,
        filter_contrast: ctx.settings.filters.contrast,
        filter_gamma: ctx.settings.filters.gamma,
        filter_exposure: ctx.settings.filters.exposure,
        filter_fsr_sharpness: ctx.settings.filters.fsr_sharpness,
        filter_median_strength: ctx.settings.filters.median_strength,
        filter_median_stride: ctx.settings.filters.median_stride,
        filter_blur_radius: ctx.settings.filters.blur_radius,
        filter_unsharp_amount: ctx.settings.filters.unsharp_amount,
        filter_unsharp_threshold: ctx.settings.filters.unsharp_threshold,
        filter_levels_in_black: ctx.settings.filters.levels_in_black,
        filter_levels_in_white: ctx.settings.filters.levels_in_white,
        filter_levels_gamma: ctx.settings.filters.levels_gamma,
        filter_levels_out_black: ctx.settings.filters.levels_out_black,
        filter_levels_out_white: ctx.settings.filters.levels_out_white,
        show_bookmark_limit_dialog: ctx.show_bookmark_limit_dialog,
        bookmark_limit_title: ctx.localizer.bookmark_limit_title().to_string(),
        bookmark_limit_message: ctx.localizer.bookmark_limit_message().to_string(),
        app_version_label: format!("HinaView v{}", env!("CARGO_PKG_VERSION")),
        show_about_dialog: ctx.show_about_dialog,
        about_title: ctx.localizer.about_title().to_string(),
        about_lines: vec![
            format!("HinaView v{}", env!("CARGO_PKG_VERSION")),
            ctx.localizer.about_description().to_string(),
            "Copyright (c) HinaView".to_string(),
        ],
        available_languages: ctx.localizer.available_languages().to_vec(),
        current_locale: ctx.localizer.current_code().to_string(),
        show_loading_spinner: ctx.show_loading_spinner,
        show_file_association_window: ctx.show_file_association_window,
        file_association_states: ctx.file_association_states,
    }
}
