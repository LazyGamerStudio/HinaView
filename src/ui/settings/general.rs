use crate::ui::{UiAction, UiArchiveSortingMode, UiSnapshot, UiThemeMode};
use egui::RichText;

pub fn render_preference_section(
    ui: &mut egui::Ui,
    snapshot: &UiSnapshot,
    actions: &mut Vec<UiAction>,
) {
    let lp = &snapshot.lang_preference;
    let resp = egui::CollapsingHeader::new(RichText::new(format!("⚙ {}", lp.title)).strong())
        .open(Some(snapshot.preference_section_open))
        .show(ui, |ui| {
            let mut hide_sec = snapshot.ui_auto_hide_sec as i32;
            ui.label(&lp.ui_auto_hide);
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Slider::new(&mut hide_sec, 1..=11).show_value(false))
                    .changed()
                {
                    actions.push(UiAction::SetUiAutoHideSec(hide_sec as u32));
                }
                ui.label(&snapshot.ui_auto_hide_label);
            });

            ui.label(&snapshot.lang_preference.prefetch_count);
            let mut prefetch_count = snapshot.prefetch_count;
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Slider::new(&mut prefetch_count, 3..=10).show_value(false))
                    .changed()
                {
                    actions.push(UiAction::SetPrefetchCount(prefetch_count));
                }
                ui.label(format!(
                    "{} {}",
                    prefetch_count,
                    snapshot.lang_view_mode.sec.replace("sec", "pages")
                )); // Fallback label logic
            });

            ui.label(&snapshot.ram_cache_setting_label);
            let mut cpu_mb = snapshot.cpu_cache_setting_mb as i32;
            if ui
                .add(
                    egui::Slider::new(&mut cpu_mb, 128..=2048)
                        .step_by(128.0)
                        .text("MB"),
                )
                .changed()
            {
                actions.push(UiAction::SetCpuCacheMb(cpu_mb as usize));
            }

            ui.label(&snapshot.vram_cache_setting_label);
            let mut vram_mb = snapshot.gpu_cache_setting_mb as i32;
            let max_vram = snapshot.gpu_cache_allowed_max_mb.max(64) as i32;
            if ui
                .add(
                    egui::Slider::new(&mut vram_mb, 64..=max_vram)
                        .step_by(64.0)
                        .text("MB"),
                )
                .changed()
            {
                actions.push(UiAction::SetGpuCacheMb(vram_mb as usize));
            }

            ui.separator();
            ui.label(&lp.archive_sorting);
            ui.horizontal(|ui| {
                let mut current_mode = snapshot.archive_sorting_mode;
                if ui
                    .radio_value(
                        &mut current_mode,
                        UiArchiveSortingMode::Mixed,
                        &lp.sort_mixed,
                    )
                    .clicked()
                {
                    actions.push(UiAction::SetArchiveSortingMode(UiArchiveSortingMode::Mixed));
                }
                if ui
                    .radio_value(
                        &mut current_mode,
                        UiArchiveSortingMode::FoldersFirst,
                        &lp.sort_folders_first,
                    )
                    .clicked()
                {
                    actions.push(UiAction::SetArchiveSortingMode(
                        UiArchiveSortingMode::FoldersFirst,
                    ));
                }
            });

            let mut remember_position = snapshot.remember_document_position;
            if ui
                .checkbox(&mut remember_position, &lp.remember_document_position)
                .changed()
            {
                actions.push(UiAction::SetRememberDocumentPosition(remember_position));
            }

            ui.label(&lp.webtoon_scroll_speed);
            let mut webtoon_speed = snapshot.webtoon_scroll_speed_px_per_sec;
            if ui
                .add(egui::Slider::new(&mut webtoon_speed, 100.0..=1600.0).text("px/s"))
                .changed()
            {
                actions.push(UiAction::SetWebtoonScrollSpeed(webtoon_speed));
            }

            ui.separator();
            ui.label(&lp.language);
            let mut current_locale = snapshot.current_locale.clone();
            egui::ComboBox::from_id_salt("language_select")
                .selected_text(
                    snapshot
                        .available_languages
                        .iter()
                        .find(|l| l.code == current_locale)
                        .map(|l| l.name.as_str())
                        .unwrap_or("Unknown"),
                )
                .show_ui(ui, |ui| {
                    for lang in &snapshot.available_languages {
                        if ui
                            .selectable_value(&mut current_locale, lang.code.clone(), &lang.name)
                            .clicked()
                        {
                            actions.push(UiAction::SetLocale(lang.code.clone()));
                        }
                    }
                });

            ui.label(&lp.theme);
            ui.horizontal(|ui| {
                let mut current_theme = snapshot.theme_mode;
                if ui
                    .radio_value(&mut current_theme, UiThemeMode::Auto, &lp.theme_auto)
                    .clicked()
                {
                    actions.push(UiAction::SetThemeMode(UiThemeMode::Auto));
                }
                if ui
                    .radio_value(&mut current_theme, UiThemeMode::Dark, &lp.theme_dark)
                    .clicked()
                {
                    actions.push(UiAction::SetThemeMode(UiThemeMode::Dark));
                }
                if ui
                    .radio_value(&mut current_theme, UiThemeMode::Light, &lp.theme_light)
                    .clicked()
                {
                    actions.push(UiAction::SetThemeMode(UiThemeMode::Light));
                }
            });

            ui.separator();
            ui.label(&lp.file_association_description);
            ui.horizontal(|ui| {
                if ui.button(&lp.file_association_button).clicked() {
                    actions.push(UiAction::ShowFileAssociationWindow(true));
                }
                if ui.button(&lp.file_association_delete_button).clicked() {
                    actions.push(UiAction::DeleteAllFileAssociations);
                }
            });
            ui.label(&lp.context_menu_section_label);
            ui.horizontal(|ui| {
                if ui.button(&lp.context_menu_add_button).clicked() {
                    actions.push(UiAction::AddContextMenu);
                }
                if ui.button(&lp.context_menu_delete_button).clicked() {
                    actions.push(UiAction::DeleteContextMenu);
                }
            });
            ui.label(&lp.start_menu_section_label);
            ui.horizontal(|ui| {
                if ui.button(&lp.start_menu_add_button).clicked() {
                    actions.push(UiAction::RegisterStartMenuShortcut);
                }
                if ui.button(&lp.start_menu_delete_button).clicked() {
                    actions.push(UiAction::UnregisterStartMenuShortcut);
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(&lp.instance_mode);
                ui.label(RichText::new(&lp.instance_restart_notice).size(11.0).weak());
            });

            let mut single_instance = snapshot.single_instance;
            if ui
                .radio_value(&mut single_instance, true, &lp.instance_single)
                .clicked()
            {
                actions.push(UiAction::SetSingleInstanceMode(true));
            }
            if ui
                .radio_value(&mut single_instance, false, &lp.instance_multi)
                .clicked()
            {
                actions.push(UiAction::SetSingleInstanceMode(false));
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(&lp.config_storage_location);
                ui.label(
                    RichText::new(&lp.config_storage_restart_notice)
                        .size(11.0)
                        .weak(),
                );
            });
            let mut storage_location = snapshot.config_storage_location;
            if ui
                .radio_value(
                    &mut storage_location,
                    crate::settings::model::ConfigStorageLocation::AppDir,
                    &lp.storage_app_dir,
                )
                .clicked()
            {
                actions.push(UiAction::SetConfigStorageLocation(
                    crate::settings::model::ConfigStorageLocation::AppDir,
                ));
            }
            if ui
                .radio_value(
                    &mut storage_location,
                    crate::settings::model::ConfigStorageLocation::SystemConfig,
                    &lp.storage_system_config,
                )
                .clicked()
            {
                actions.push(UiAction::SetConfigStorageLocation(
                    crate::settings::model::ConfigStorageLocation::SystemConfig,
                ));
            }
        });

    if resp.header_response.clicked() {
        actions.push(UiAction::SetPreferenceSectionOpen(
            !snapshot.preference_section_open,
        ));
    }

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button(snapshot.about_title.to_lowercase()).clicked() {
            actions.push(UiAction::ToggleAboutDialog(true));
        }
        ui.label(&snapshot.app_version_label);
    });
}
