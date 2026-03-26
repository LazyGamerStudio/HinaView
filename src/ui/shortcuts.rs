use crate::ui::{UiAction, UiSnapshot};
use egui::RichText;

/// Default width of the shortcuts panel in pixels
const SHORTCUTS_PANEL_WIDTH: f32 = 280.0;

pub fn render_shortcuts(ctx: &egui::Context, snapshot: &UiSnapshot, actions: &mut Vec<UiAction>) {
    let window_id = egui::Id::new("shortcuts_window");
    let collapsing_id = window_id.with("collapsing");

    // [1] Initial seed: if memory is empty, set from snapshot
    if egui::collapsing_header::CollapsingState::load(ctx, collapsing_id).is_none() {
        egui::collapsing_header::CollapsingState::load_with_default_open(
            ctx,
            collapsing_id,
            !snapshot.shortcuts_window_collapsed,
        )
        .store(ctx);
    }

    let res = egui::Window::new(RichText::new(&snapshot.shortcuts_title).size(15.0).strong())
        .id(window_id)
        .title_bar(true)
        .resizable(false)
        .movable(false)
        .default_width(SHORTCUTS_PANEL_WIDTH)
        .max_width(SHORTCUTS_PANEL_WIDTH)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
        .frame(
            egui::Frame::window(&ctx.style()).fill(
                ctx.style()
                    .visuals
                    .window_fill()
                    .linear_multiply(snapshot.ui_opacity),
            ),
        )
        .show(ctx, |ui: &mut egui::Ui| {
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

            ui.add_space(4.0);
            egui::Grid::new("shortcuts_grid")
                .num_columns(2)
                .spacing([16.0, 5.0])
                .striped(false)
                .show(ui, |ui| {
                    for line in &snapshot.shortcuts_lines {
                        if let Some((keys_str, name)) = line.split_once(" : ") {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.spacing_mut().item_spacing.x = 2.0;
                                    let keys: Vec<&str> = keys_str.split(" | ").collect();
                                    for key in keys.iter().rev() {
                                        render_key_cap(ui, key);
                                    }
                                },
                            );

                            ui.horizontal(|ui| {
                                ui.add_space(2.0);
                                ui.label(
                                    egui::RichText::new(name).color(ui.visuals().text_color()),
                                );
                            });

                            ui.end_row();
                        }
                    }
                });
            ui.add_space(4.0);
        });

    // [3] Detect changes after rendering
    if res.is_some() {
        if let Some(state) = egui::collapsing_header::CollapsingState::load(ctx, collapsing_id) {
            let is_collapsed = !state.is_open();
            if is_collapsed != snapshot.shortcuts_window_collapsed {
                actions.push(UiAction::SetShortcutsWindowCollapsed(is_collapsed));
            }
        }
    }
}

fn render_key_cap(ui: &mut egui::Ui, text: &str) {
    let is_dark = ui.visuals().dark_mode;
    let base_color = if is_dark {
        egui::Color32::from_gray(60)
    } else {
        egui::Color32::from_gray(230)
    };

    let text_color = ui.visuals().text_color();

    egui::Frame::canvas(ui.style())
        .fill(base_color)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).small().strong().color(text_color));
        });
}
