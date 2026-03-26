use egui::{Align2, Color32, Frame, Id, Margin, RichText, ProgressBar};
use crate::pipeline::status::DecodeStatus;

pub fn render_loading_indicator(egui_ctx: &egui::Context, status: DecodeStatus) {
    egui::Area::new(Id::new("loading_indicator"))
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .interactable(false)
        .show(egui_ctx, |ui| {
            Frame::new()
                .fill(Color32::from_black_alpha(160))
                .corner_radius(10.0)
                .inner_margin(Margin::symmetric(16, 14))
                .stroke(egui::Stroke::new(1.0, Color32::from_gray(100)))
                .show(ui, |ui| {
                    ui.set_max_width(224.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(status.label())
                                .color(Color32::WHITE)
                                .size(14.0)
                                .strong(),
                        );
                        ui.add_space(10.0);
                        
                        // Fake progress based on the state for a smooth feel
                        let progress = status.progress();
                        ui.add(
                            ProgressBar::new(progress)
                                .animate(true)
                                .show_percentage()
                                .desired_height(12.0)
                                .fill(Color32::from_rgb(0, 150, 255)),
                        );
                        ui.add_space(2.0);
                    });
                });
        });
}
