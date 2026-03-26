// src/ui/favorites.rs
// CRITICAL: This is a CUSTOM DRAWER implementation with manual animation and area-based rendering.
// DO NOT refactor this into a standard egui::Window or any other built-in windowing system.
// The animation and handle logic are precisely tuned for the HinaView side-drawer experience.

use crate::ui::{UiAction, UiSnapshot};
use egui::RichText;

/// Default width of side drawers (favorites, shortcuts) in pixels
const DRAWER_DEFAULT_WIDTH: f32 = 300.0;

pub fn render_favorites(ctx: &egui::Context, snapshot: &UiSnapshot, actions: &mut Vec<UiAction>) {
    let lb = &snapshot.lang_bookmark;
    let drawer_width = DRAWER_DEFAULT_WIDTH;
    let animation_duration = 0.2;

    // Animation factor: 0.0 (closed) to 1.0 (open)
    let animation_factor = ctx.animate_bool_with_time(
        egui::Id::new("bookmark_drawer_animation"),
        snapshot.bookmark_drawer_open,
        animation_duration,
    );

    if animation_factor <= 0.0 && !snapshot.bookmark_drawer_open {
        // Render handle only when closed
        egui::Area::new(egui::Id::new("bookmark_drawer_handle"))
            .anchor(egui::Align2::RIGHT_TOP, [0.0, 120.0])
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_at_least(egui::vec2(16.0, 64.0), egui::Sense::click());
                let color = if response.hovered() {
                    ui.visuals().widgets.active.bg_fill
                } else {
                    let mut c = ui.visuals().widgets.noninteractive.bg_fill;
                    c = c.linear_multiply(1.5);
                    if c.r() < 80 && c.g() < 80 && c.b() < 80 {
                        egui::Color32::from_gray(80)
                    } else {
                        c
                    }
                };

                ui.painter()
                    .rect_filled(rect, egui::CornerRadius::same(4), color);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "◀",
                    egui::FontId::proportional(14.0),
                    ui.visuals().widgets.active.fg_stroke.color,
                );

                if response.clicked() {
                    actions.push(UiAction::SetBookmarkDrawerOpen(true));
                }
            });
        return;
    }

    // Render drawer
    let offset = (1.0 - animation_factor) * drawer_width;
    egui::Area::new(egui::Id::new("bookmark_drawer"))
        .anchor(egui::Align2::RIGHT_TOP, [offset, 20.0])
        .show(ctx, |ui| {
            let screen_rect = ui.ctx().viewport_rect();
            let frame = egui::Frame::window(&ui.style())
                .fill(
                    ui.visuals()
                        .window_fill()
                        .linear_multiply(snapshot.ui_opacity),
                )
                .inner_margin(8.0)
                .corner_radius(egui::CornerRadius {
                    nw: 8,
                    sw: 8,
                    ..Default::default()
                });

            frame.show(ui, |ui| {
                ui.set_min_width(drawer_width);
                // egui::Grid columns default to a minimum width based on `interact_size.x` (typically 40px).
                // Overriding this to 20px allows the first column (delete button) to be more compact,
                // effectively eliminating the large visual gap to the directory label.
                ui.spacing_mut().interact_size.x = 20.0;
                ui.spacing_mut().button_padding = egui::vec2(2.0, 1.0);

                // Apply UI opacity to internal widgets while keeping text solid
                let opacity = snapshot.ui_opacity;
                let visuals = ui.visuals_mut();
                visuals.widgets.noninteractive.bg_fill = visuals
                    .widgets
                    .noninteractive
                    .bg_fill
                    .linear_multiply(opacity);
                visuals.widgets.inactive.bg_fill =
                    visuals.widgets.inactive.bg_fill.linear_multiply(opacity);
                visuals.widgets.hovered.bg_fill =
                    visuals.widgets.hovered.bg_fill.linear_multiply(opacity);
                visuals.widgets.active.bg_fill =
                    visuals.widgets.active.bg_fill.linear_multiply(opacity);
                visuals.selection.bg_fill = visuals.selection.bg_fill.linear_multiply(opacity);
                visuals.window_fill = visuals.window_fill.linear_multiply(opacity);
                visuals.extreme_bg_color = visuals.extreme_bg_color.linear_multiply(opacity);
                visuals.faint_bg_color = visuals.faint_bg_color.linear_multiply(opacity);
                visuals.override_text_color = None; // Keep text solid

                // Title Section
                ui.label(RichText::new(&snapshot.favorites_title).size(15.0).strong());
                ui.separator();

                if snapshot.bookmark_rows.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label("-");
                    });
                } else {
                    let mut auto_rows = Vec::new();
                    let mut manual_rows = Vec::new();
                    for entry in &snapshot.bookmark_rows {
                        if entry.source_label == lb.auto {
                            auto_rows.push(entry);
                        } else {
                            manual_rows.push(entry);
                        }
                    }

                    egui::ScrollArea::vertical()
                        .id_salt("bookmark_scroll")
                        .max_height(screen_rect.height() * 0.8)
                        .show(ui, |ui| {
                            if !auto_rows.is_empty() {
                                egui::Grid::new("auto_bookmark_grid")
                                    .num_columns(2)
                                    .spacing([2.0, 6.0])
                                    .show(ui, |ui| {
                                        for entry in &auto_rows {
                                            render_bookmark_row(ui, entry, actions);
                                        }
                                    });
                            }

                            if !manual_rows.is_empty() && !auto_rows.is_empty() {
                                ui.separator();
                            }

                            if !manual_rows.is_empty() {
                                egui::Grid::new("manual_bookmark_grid")
                                    .num_columns(2)
                                    .spacing([2.0, 6.0])
                                    .show(ui, |ui| {
                                        for entry in &manual_rows {
                                            render_bookmark_row(ui, entry, actions);
                                        }
                                    });
                            }
                        });
                }
            });
        });

    // Close when clicking outside
    if snapshot.bookmark_drawer_open && ctx.input(|i| i.pointer.any_pressed()) {
        if let Some(pos) = ctx.input(|i| i.pointer.press_origin()) {
            if pos.x < ctx.viewport_rect().width() - drawer_width {
                actions.push(UiAction::SetBookmarkDrawerOpen(false));
            }
        }
    }
}

fn render_bookmark_row(
    ui: &mut egui::Ui,
    entry: &crate::ui::UiBookmarkRow,
    actions: &mut Vec<UiAction>,
) {
    // Column 1: Delete button (compact)
    if ui
        .add(egui::Button::new("🗑").min_size(egui::vec2(18.0, 18.0)))
        .clicked()
    {
        actions.push(UiAction::DeleteBookmark(entry.id));
    }

    // Column 2: Combined directory/file label
    // SelectableLabel is natively left-aligned and efficient
    let combined = format!("{} / {}", entry.archive_name, entry.page_name);
    let resp = ui.selectable_label(false, RichText::new(combined).size(11.0));

    if resp.clicked() {
        actions.push(UiAction::OpenBookmark(entry.id));
        actions.push(UiAction::SetBookmarkDrawerOpen(false));
    }

    ui.end_row();
}
