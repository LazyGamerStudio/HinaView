// src/ui/file_association.rs
// File Association Settings Window

use crate::ui::{UiAction, UiSnapshot};
use crate::util::formats::SUPPORTED_IMAGE_EXTENSIONS;
use egui::{Align, Layout, RichText, ScrollArea};

/// Render the file association settings window
pub fn render_file_association_window(
    ctx: &egui::Context,
    snapshot: &UiSnapshot,
    actions: &mut Vec<UiAction>,
    file_association_icons: &mut std::collections::HashMap<String, egui::TextureHandle>,
) {
    if !snapshot.show_file_association_window {
        return;
    }

    // Initialize icons if not already done
    if file_association_icons.is_empty() {
        tracing::info!("Initializing file association icons...");
        for ext in SUPPORTED_IMAGE_EXTENSIONS {
            if let Some(ico_data) = crate::ui::file_association_icons::get_icon_for_extension(ext) {
                tracing::info!("Loading icon for {}", ext);
                if let Some(rgba) = crate::ui::file_association_icons::load_icon_from_ico(ico_data)
                {
                    let image = egui::ColorImage::from_rgba_unmultiplied([16, 16], &rgba);
                    let texture = ctx.load_texture(
                        &format!("icon_{}", ext),
                        egui::ImageData::Color(image.into()),
                        Default::default(),
                    );
                    file_association_icons.insert(ext.to_string(), texture);
                    tracing::info!("Icon loaded for {}", ext);
                } else {
                    tracing::warn!("Failed to load icon for {}", ext);
                }
            } else {
                tracing::warn!("No ICO data found for {}", ext);
            }
        }
        tracing::info!("Loaded {} icons", file_association_icons.len());
    }

    let lfa = &snapshot.lang_file_association;
    let screen_rect = ctx.viewport_rect();
    let window_width = 400.0;
    let default_pos = [
        (screen_rect.width() - window_width) / 2.0,
        (screen_rect.height() - 400.0) / 4.0,
    ];

    egui::Window::new(RichText::new(&lfa.window_title).size(16.0).strong())
        .id(egui::Id::new("file_association_window"))
        .default_width(window_width)
        .default_pos(default_pos)
        .resizable(false)
        .movable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(&lfa.window_subtitle);
            ui.separator();

            // Scrollable list of extensions with aligned columns
            ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("file_association_grid")
                    .num_columns(3)
                    .spacing([10.0, 4.0])
                    .min_col_width(0.0)
                    .show(ui, |ui| {
                        for (ext, description, is_associated) in &snapshot.file_association_states {
                            // Icon + Extension (combined in one column, left aligned)
                            ui.horizontal(|ui| {
                                ui.set_width(60.0); // Fixed width for icon+ext column
                                if let Some(icon) = file_association_icons.get(ext) {
                                    ui.add(
                                        egui::Image::new(icon)
                                            .fit_to_exact_size([16.0, 16.0].into()),
                                    );
                                } else {
                                    ui.label("📁");
                                }
                                ui.label(
                                    RichText::new(ext)
                                        .font(egui::FontId::monospace(14.0))
                                        .strong(),
                                );
                            });

                            // Description
                            ui.label(description);

                            // Checkbox (right aligned)
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let mut checked = *is_associated;
                                if ui.checkbox(&mut checked, "").changed() {
                                    actions.push(UiAction::UpdateFileAssociation(
                                        ext.clone(),
                                        checked,
                                    ));
                                }
                            });

                            ui.end_row();
                        }
                    });
            });

            ui.separator();

            // Bottom buttons
            ui.horizontal(|ui| {
                // Select All / Deselect All
                let all_selected = snapshot
                    .file_association_states
                    .iter()
                    .all(|(_, _, checked)| *checked);

                if ui
                    .add(egui::Button::new(if all_selected {
                        &lfa.deselect_all
                    } else {
                        &lfa.select_all
                    }))
                    .clicked()
                {
                    actions.push(UiAction::SelectAllFileAssociations(!all_selected));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Apply button (rightmost)
                    if ui.button(&lfa.apply).clicked() {
                        actions.push(UiAction::ApplyFileAssociations);
                        actions.push(UiAction::ShowFileAssociationWindow(false));
                    }

                    // Cancel button (left of apply)
                    if ui.button(&lfa.cancel).clicked() {
                        actions.push(UiAction::ShowFileAssociationWindow(false));
                    }
                });
            });
        });
}

/// Initialize file association states from registry
pub fn init_file_association_states() -> Vec<(String, String, bool)> {
    let mut states = Vec::new();

    for ext in SUPPORTED_IMAGE_EXTENSIONS {
        let ext_str = ext.to_string();
        let description = get_extension_description(ext);
        let is_associated = crate::system::win_registry::is_associated(ext);

        states.push((ext_str, description, is_associated));
    }

    states
}

/// Get a human-readable description for an extension
fn get_extension_description(ext: &str) -> String {
    match ext.trim_start_matches('.').to_lowercase().as_str() {
        "webp" => "WebP Image".to_string(),
        "avif" => "AVIF Image".to_string(),
        "heif" | "heic" => "HEIF/HEIC Image".to_string(),
        "jxl" => "JPEG XL Image".to_string(),
        "jpg" | "jpeg" => "JPEG Image".to_string(),
        "png" => "PNG Image".to_string(),
        "gif" => "GIF Image".to_string(),
        "bmp" => "Windows Bitmap".to_string(),
        "tiff" | "tif" => "TIFF Image".to_string(),
        "tga" => "Targa Image".to_string(),
        "dds" => "DirectDraw Surface".to_string(),
        "exr" => "OpenEXR Image".to_string(),
        "hdr" => "Radiance HDR Image".to_string(),
        "pnm" => "Portable Anymap".to_string(),
        "ico" => "Windows Icon".to_string(),
        "cbz" => "Comic Book Archive".to_string(),
        _ => format!("{} File", ext.trim_start_matches('.').to_uppercase()),
    }
}
