use super::{left_value, right_label};
use crate::ui::{UiAction, UiSnapshot};
use egui::RichText;

pub fn render_info_section(ui: &mut egui::Ui, snapshot: &UiSnapshot, actions: &mut Vec<UiAction>) {
    let li = &snapshot.lang_info;
    let resp = egui::CollapsingHeader::new(RichText::new(format!("ℹ {}", li.title)).strong())
        .open(Some(snapshot.info_section_open))
        .show(ui, |ui| {
            ui.strong(&snapshot.archive_name_value);
            egui::Grid::new("settings_info_grid")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    right_label(ui, &li.filename);
                    left_value(ui, &snapshot.file_name_value);
                    ui.end_row();
                    right_label(ui, &li.filesize);
                    left_value(ui, &snapshot.file_size_value);
                    ui.end_row();
                    right_label(ui, &li.info);
                    left_value(ui, &snapshot.info_value);
                    ui.end_row();
                    right_label(ui, &li.icc_profile);
                    left_value(ui, &snapshot.icc_profile_value);
                    ui.end_row();
                });

            let progress = if snapshot.cpu_cache_max_mb == 0 {
                0.0
            } else {
                snapshot.cpu_cache_current_mb as f32 / snapshot.cpu_cache_max_mb as f32
            };

            let fill_color = snapshot
                .accent_color
                .map(crate::util::os_colors::find_closest_basic_color)
                .unwrap_or(ui.visuals().selection.bg_fill);

            let pb = egui::ProgressBar::new(progress)
                .show_percentage()
                .text(&snapshot.ram_cache_display)
                .fill(fill_color);

            ui.add(pb);
        });

    if resp.header_response.clicked() {
        actions.push(UiAction::SetInfoSectionOpen(!snapshot.info_section_open));
    }
    ui.separator();
}

pub fn render_exif_section(ui: &mut egui::Ui, snapshot: &UiSnapshot, actions: &mut Vec<UiAction>) {
    let le = &snapshot.lang_exif;
    let resp = egui::CollapsingHeader::new(RichText::new(format!("🔍 {}", le.title)).strong())
        .open(Some(snapshot.exif_section_open))
        .show(ui, |ui| {
            egui::Grid::new("settings_exif_grid")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    right_label(ui, &le.camera);
                    left_value(ui, &snapshot.exif_camera_value);
                    ui.end_row();
                    right_label(ui, &le.lens);
                    left_value(ui, &snapshot.exif_lens_value);
                    ui.end_row();
                    right_label(ui, &le.f_stop);
                    left_value(ui, &snapshot.exif_f_stop_value);
                    ui.end_row();
                    right_label(ui, &le.shutter_speed);
                    left_value(ui, &snapshot.exif_shutter_value);
                    ui.end_row();
                    right_label(ui, &le.iso);
                    left_value(ui, &snapshot.exif_iso_value);
                    ui.end_row();
                    right_label(ui, &le.datetime);
                    left_value(ui, &snapshot.exif_datetime_value);
                    ui.end_row();
                });
        });

    if resp.header_response.clicked() {
        actions.push(UiAction::SetExifSectionOpen(!snapshot.exif_section_open));
    }
    ui.separator();
}
