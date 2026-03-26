use crate::ui::{UiAction, UiFitMode, UiLayoutMode, UiSnapshot};
use egui::{Align, Layout, RichText};

pub fn render_view_section(ui: &mut egui::Ui, snapshot: &UiSnapshot, actions: &mut Vec<UiAction>) {
    let lv = &snapshot.lang_view_mode;
    let resp = egui::CollapsingHeader::new(RichText::new(format!("🖼 {}", lv.title)).strong())
        .open(Some(snapshot.view_mode_section_open))
        .show(ui, |ui| {
            if ui.button(format!("🔄 {}", lv.view_reset)).clicked() {
                actions.push(UiAction::ResetView);
            }
            ui.separator();

            let mut fit_mode = snapshot.fit_mode;
            ui.horizontal(|ui| {
                if ui
                    .radio(matches!(fit_mode, UiFitMode::FitScreen), &lv.fit_window)
                    .clicked()
                {
                    fit_mode = UiFitMode::FitScreen;
                    actions.push(UiAction::SetFitMode(fit_mode));
                }
                if ui
                    .radio(matches!(fit_mode, UiFitMode::FitWidth), &lv.fit_width)
                    .clicked()
                {
                    fit_mode = UiFitMode::FitWidth;
                    actions.push(UiAction::SetFitMode(fit_mode));
                }
                if ui
                    .radio(matches!(fit_mode, UiFitMode::FitHeight), &lv.fit_height)
                    .clicked()
                {
                    fit_mode = UiFitMode::FitHeight;
                    actions.push(UiAction::SetFitMode(fit_mode));
                }
                if ui
                    .radio(matches!(fit_mode, UiFitMode::Zoom), &lv.zoom)
                    .clicked()
                {
                    fit_mode = UiFitMode::Zoom;
                    actions.push(UiAction::SetFitMode(fit_mode));
                }
            });

            let mut layout_mode = snapshot.layout_mode;
            ui.horizontal(|ui| {
                if ui
                    .radio(matches!(layout_mode, UiLayoutMode::Single), &lv.single)
                    .clicked()
                {
                    layout_mode = UiLayoutMode::Single;
                    actions.push(UiAction::SetLayoutMode(layout_mode));
                }
                if ui
                    .radio(matches!(layout_mode, UiLayoutMode::DualLtr), &lv.ltr)
                    .clicked()
                {
                    layout_mode = UiLayoutMode::DualLtr;
                    actions.push(UiAction::SetLayoutMode(layout_mode));
                }
                if ui
                    .radio(matches!(layout_mode, UiLayoutMode::DualRtl), &lv.rtl)
                    .clicked()
                {
                    layout_mode = UiLayoutMode::DualRtl;
                    actions.push(UiAction::SetLayoutMode(layout_mode));
                }
                if ui
                    .radio(
                        matches!(layout_mode, UiLayoutMode::VerticalScroll),
                        &snapshot.layout_vertical_scroll_label,
                    )
                    .clicked()
                {
                    layout_mode = UiLayoutMode::VerticalScroll;
                    actions.push(UiAction::SetLayoutMode(layout_mode));
                }
            });

            let mut first_page_offset = snapshot.first_page_offset;
            if ui
                .checkbox(&mut first_page_offset, &lv.one_page_offset)
                .changed()
            {
                actions.push(UiAction::SetFirstPageOffset(first_page_offset));
            }

            ui.label(&snapshot.slideshow_repeat_label);
            ui.horizontal(|ui| {
                let mut slideshow_sec = snapshot.slideshow_interval_sec as i32;
                if ui
                    .add(
                        egui::Slider::new(&mut slideshow_sec, 0..=30)
                            .text(&lv.sec)
                            .clamping(egui::SliderClamping::Always),
                    )
                    .changed()
                {
                    actions.push(UiAction::SetSlideshowIntervalSec(slideshow_sec as u32));
                }

                let next = !snapshot.slideshow_enabled;
                let label = if snapshot.slideshow_enabled {
                    &lv.slideshow_stop
                } else {
                    &lv.slideshow_start
                };
                ui.push_id("slideshow_toggle", |ui| {
                    if ui.button(label).clicked() {
                        actions.push(UiAction::SetSlideshowEnabled(next));
                    }
                });
            });
        });

    if resp.header_response.clicked() {
        actions.push(UiAction::SetViewModeSectionOpen(
            !snapshot.view_mode_section_open,
        ));
    }
    ui.separator();
}

pub fn render_filter_section(
    ui: &mut egui::Ui,
    snapshot: &UiSnapshot,
    actions: &mut Vec<UiAction>,
) {
    let lf = &snapshot.lang_filter;
    let resp = egui::CollapsingHeader::new(RichText::new(format!("🎨 {}", lf.title)).strong())
        .open(Some(snapshot.filter_section_open))
        .show(ui, |ui| {
            // 1. Base Color Section
            ui.horizontal(|ui| {
                if ui.button(&lf.reset).clicked() {
                    actions.push(UiAction::ResetFilterColor);
                }
                ui.label(RichText::new(format!(" {}", &lf.color)).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut bypass = snapshot.filter_bypass_color;
                    if ui.checkbox(&mut bypass, &lf.bypass).changed() {
                        actions.push(UiAction::SetFilterBypassColor(bypass));
                    }
                });
            });

            let mut bright = snapshot.filter_bright;
            if ui
                .add(egui::Slider::new(&mut bright, -1.0..=1.0).text(&lf.bright))
                .changed()
            {
                actions.push(UiAction::SetFilterBright(bright));
            }
            let mut contrast = snapshot.filter_contrast;
            if ui
                .add(egui::Slider::new(&mut contrast, 0.0..=2.0).text(&lf.contrast))
                .changed()
            {
                actions.push(UiAction::SetFilterContrast(contrast));
            }
            let mut gamma = snapshot.filter_gamma;
            if ui
                .add(egui::Slider::new(&mut gamma, 0.2..=3.0).text(&lf.gamma))
                .changed()
            {
                actions.push(UiAction::SetFilterGamma(gamma));
            }
            let mut exposure = snapshot.filter_exposure;
            if ui
                .add(egui::Slider::new(&mut exposure, -4.0..=4.0).text(&lf.exposure))
                .changed()
            {
                actions.push(UiAction::SetFilterExposure(exposure));
            }

            ui.separator();

            // 2. Median Filter Section
            ui.horizontal(|ui| {
                if ui.button(&lf.reset).clicked() {
                    actions.push(UiAction::ResetFilterMedian);
                }
                ui.label(RichText::new(format!(" {}", &lf.median)).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut bypass = snapshot.filter_bypass_median;
                    if ui.checkbox(&mut bypass, &lf.bypass).changed() {
                        actions.push(UiAction::SetFilterBypassMedian(bypass));
                    }
                });
            });

            let mut m_strength = snapshot.filter_median_strength;
            if ui
                .add(egui::Slider::new(&mut m_strength, 0.0..=1.0).text(&lf.median_strength))
                .changed()
            {
                actions.push(UiAction::SetFilterMedianStrength(m_strength));
            }
            if snapshot.filter_median_strength > 0.01 {
                let mut m_stride = snapshot.filter_median_stride;
                if ui
                    .add(egui::Slider::new(&mut m_stride, 1.0..=5.0).text(&lf.median_stride))
                    .changed()
                {
                    actions.push(UiAction::SetFilterMedianStride(m_stride));
                }
            }

            ui.separator();

            // 3. Detail Correction Section (Blur + Unsharp)
            ui.horizontal(|ui| {
                if ui.button(&lf.reset).clicked() {
                    actions.push(UiAction::ResetFilterDetail);
                }
                ui.label(RichText::new(format!(" {}", &lf.detail)).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut bypass = snapshot.filter_bypass_detail;
                    if ui.checkbox(&mut bypass, &lf.bypass).changed() {
                        actions.push(UiAction::SetFilterBypassDetail(bypass));
                    }
                });
            });

            let mut blur = snapshot.filter_blur_radius;
            if ui
                .add(egui::Slider::new(&mut blur, 0.0..=5.0).text(&lf.blur))
                .changed()
            {
                actions.push(UiAction::SetFilterBlurRadius(blur));
            }
            let mut unsharp = snapshot.filter_unsharp_amount;
            if ui
                .add(egui::Slider::new(&mut unsharp, 0.0..=2.0).text(&lf.unsharp))
                .changed()
            {
                actions.push(UiAction::SetFilterUnsharpAmount(unsharp));
            }
            if snapshot.filter_unsharp_amount > 0.01 {
                let mut threshold = snapshot.filter_unsharp_threshold;
                if ui
                    .add(egui::Slider::new(&mut threshold, 0.0..=1.0).text(&lf.unsharp_threshold))
                    .changed()
                {
                    actions.push(UiAction::SetFilterUnsharpThreshold(threshold));
                }
            }

            ui.separator();

            // 4. Levels Section
            ui.horizontal(|ui| {
                if ui.button(&lf.reset).clicked() {
                    actions.push(UiAction::ResetFilterLevels);
                }
                ui.label(RichText::new(format!(" {}", &lf.levels)).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut bypass = snapshot.filter_bypass_levels;
                    if ui.checkbox(&mut bypass, &lf.bypass).changed() {
                        actions.push(UiAction::SetFilterBypassLevels(bypass));
                    }
                });
            });

            let mut l_in_b = snapshot.filter_levels_in_black;
            let mut l_in_w = snapshot.filter_levels_in_white;
            let mut l_gamma = snapshot.filter_levels_gamma;
            let mut l_out_b = snapshot.filter_levels_out_black;
            let mut l_out_w = snapshot.filter_levels_out_white;

            ui.label(RichText::new(&lf.levels_in).size(12.0));
            if ui
                .add(egui::Slider::new(&mut l_in_b, 0.0..=1.0).text(&lf.levels_black))
                .changed()
            {
                actions.push(UiAction::SetFilterLevelsInBlack(l_in_b));
            }
            if ui
                .add(egui::Slider::new(&mut l_gamma, 0.1..=5.0).text(&lf.levels_mid))
                .changed()
            {
                actions.push(UiAction::SetFilterLevelsGamma(l_gamma));
            }
            if ui
                .add(egui::Slider::new(&mut l_in_w, 0.0..=1.0).text(&lf.levels_white))
                .changed()
            {
                actions.push(UiAction::SetFilterLevelsInWhite(l_in_w));
            }

            ui.add_space(4.0);
            ui.label(RichText::new(&lf.levels_out).size(12.0));
            if ui
                .add(egui::Slider::new(&mut l_out_b, 0.0..=1.0).text(&lf.levels_black))
                .changed()
            {
                actions.push(UiAction::SetFilterLevelsOutBlack(l_out_b));
            }
            if ui
                .add(egui::Slider::new(&mut l_out_w, 0.0..=1.0).text(&lf.levels_white))
                .changed()
            {
                actions.push(UiAction::SetFilterLevelsOutWhite(l_out_w));
            }

            ui.separator();

            // 5. AMD FSR
            ui.horizontal(|ui| {
                ui.label(RichText::new(&lf.fsr_enabled).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut bypass = snapshot.filter_bypass_fsr;
                    if ui.checkbox(&mut bypass, &lf.bypass).changed() {
                        actions.push(UiAction::SetFilterBypassFsr(bypass));
                    }
                });
            });
            let mut sharpness = snapshot.filter_fsr_sharpness;
            if ui
                .add(egui::Slider::new(&mut sharpness, 0.0..=1.0).text(&lf.fsr_sharpness))
                .changed()
            {
                actions.push(UiAction::SetFilterFsrSharpness(sharpness));
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button(format!("🔄 {}", &lf.reset)).clicked() {
                    actions.push(UiAction::ResetFilters);
                }
            });
        });

    if resp.header_response.clicked() {
        actions.push(UiAction::SetFilterSectionOpen(
            !snapshot.filter_section_open,
        ));
    }
    ui.separator();
}
