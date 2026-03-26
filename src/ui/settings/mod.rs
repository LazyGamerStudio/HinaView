use crate::ui::favorites::render_favorites;
use crate::ui::file_association::render_file_association_window;
use crate::ui::{UiAction, UiSnapshot};
use egui::{Align, Layout, RichText};

pub mod debug;
pub mod general;
pub mod view;

const SETTINGS_PANEL_DEFAULT_WIDTH: f32 = 320.0;
const SETTINGS_PANEL_MAX_WIDTH: f32 = 320.0;

pub fn render_fixed_panels(
    ctx: &egui::Context,
    snapshot: &UiSnapshot,
    actions: &mut Vec<UiAction>,
    file_association_icons: &mut std::collections::HashMap<String, egui::TextureHandle>,
) {
    if snapshot.ui_windows_visible {
        render_settings_window(ctx, snapshot, actions);
        render_favorites(ctx, snapshot, actions);
    }

    // File Association Window (always render, regardless of ui_windows_visible)
    render_file_association_window(ctx, snapshot, actions, file_association_icons);

    if snapshot.show_bookmark_limit_dialog {
        egui::Window::new(&snapshot.bookmark_limit_title)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(&snapshot.bookmark_limit_message);
                if ui.button("OK").clicked() {
                    actions.push(UiAction::DismissBookmarkLimitDialog);
                }
            });
    }

    if snapshot.show_about_dialog {
        egui::Window::new(&snapshot.about_title)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                for line in &snapshot.about_lines {
                    ui.label(line);
                }
                if ui.button("OK").clicked() {
                    actions.push(UiAction::ToggleAboutDialog(false));
                }
            });
    }
}

fn render_settings_window(ctx: &egui::Context, snapshot: &UiSnapshot, actions: &mut Vec<UiAction>) {
    let window_id = egui::Id::new("settings_window");
    let collapsing_id = window_id.with("collapsing");

    if egui::collapsing_header::CollapsingState::load(ctx, collapsing_id).is_none() {
        egui::collapsing_header::CollapsingState::load_with_default_open(
            ctx,
            collapsing_id,
            !snapshot.settings_window_collapsed,
        )
        .store(ctx);
    }

    let screen_rect = ctx.viewport_rect();
    let max_height = screen_rect.height() - 16.0;

    let res = egui::Window::new(RichText::new(&snapshot.settings_title).size(15.0).strong())
        .id(window_id)
        .title_bar(true)
        .resizable(false)
        .movable(false)
        .auto_sized()
        .default_width(SETTINGS_PANEL_DEFAULT_WIDTH)
        .max_width(SETTINGS_PANEL_MAX_WIDTH)
        .default_pos([8.0, 8.0])
        .frame(
            egui::Frame::window(&ctx.style()).fill(
                ctx.style()
                    .visuals
                    .window_fill()
                    .linear_multiply(snapshot.ui_opacity),
            ),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("settings_scroll_internal")
                .max_height(max_height - 40.0)
                .auto_shrink([true, true])
                .show(ui, |ui| {
                    ui.set_width(SETTINGS_PANEL_DEFAULT_WIDTH - 20.0);

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
                    visuals.override_text_color = None;

                    self::debug::render_info_section(ui, snapshot, actions);
                    self::debug::render_exif_section(ui, snapshot, actions);
                    self::view::render_view_section(ui, snapshot, actions);
                    self::view::render_filter_section(ui, snapshot, actions);
                    self::general::render_preference_section(ui, snapshot, actions);
                });
        });

    if res.is_some() {
        if let Some(state) = egui::collapsing_header::CollapsingState::load(ctx, collapsing_id) {
            let is_collapsed = !state.is_open();
            if is_collapsed != snapshot.settings_window_collapsed {
                actions.push(UiAction::SetSettingsWindowCollapsed(is_collapsed));
            }
        }
    }
}

pub(super) fn right_label(ui: &mut egui::Ui, label: &str) {
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.label(label);
    });
}

pub(super) fn left_value(ui: &mut egui::Ui, value: &str) {
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.label(value);
    });
}
