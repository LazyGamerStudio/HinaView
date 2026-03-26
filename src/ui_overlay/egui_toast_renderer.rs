use egui::{Align2, Color32, Frame, Id, Margin, RichText};

pub struct EguiToastRenderer {
    context: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

pub struct EguiRenderContext<'a> {
    pub window: &'a winit::window::Window,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub target_view: &'a wgpu::TextureView,
    pub ui_snapshot: &'a crate::ui::UiSnapshot,
    pub text: Option<&'a str>,
    pub warning_text: Option<String>,
    pub file_association_icons: &'a mut std::collections::HashMap<String, egui::TextureHandle>,
}

impl EguiToastRenderer {
    pub fn new(
        window: &winit::window::Window,
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        let context = egui::Context::default();
        let viewport_id = egui::ViewportId::ROOT;
        let state = egui_winit::State::new(
            context.clone(),
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let renderer = egui_wgpu::Renderer::new(device, output_format, Default::default());
        crate::ui_overlay::font_setup::apply_preferred_cjk_fonts(&context);

        Self {
            context,
            state,
            renderer,
        }
    }

    pub fn on_window_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        let response = self.state.on_window_event(window, event);
        response.repaint
    }

    pub fn wants_pointer_input(&self) -> bool {
        self.context.wants_pointer_input()
    }

    pub fn wants_keyboard_input(&self) -> bool {
        self.context.wants_keyboard_input()
    }

    pub fn paint(&mut self, ctx: EguiRenderContext) -> (Vec<crate::ui::UiAction>, bool) {
        let mut actions = Vec::new();

        // Apply theme and global style
        let mut visuals = match ctx.ui_snapshot.theme_mode {
            crate::ui::UiThemeMode::Auto => egui::Visuals::default(),
            crate::ui::UiThemeMode::Dark => egui::Visuals::dark(),
            crate::ui::UiThemeMode::Light => egui::Visuals::light(),
        };
        visuals.selection.bg_fill = Color32::from_gray(128);

        self.context.style_mut(|s| {
            s.visuals = visuals;
            // Reduce window title bar height and padding
            s.spacing.window_margin = egui::Margin::symmetric(8, 6); // Slightly reduced
            s.spacing.item_spacing = egui::vec2(8.0, 4.0);
        });

        let raw_input = self.state.take_egui_input(ctx.window);
        let output = self.context.run(raw_input, |egui_ctx| {
            if ctx.ui_snapshot.ui_windows_visible {
                crate::ui::settings::render_fixed_panels(
                    egui_ctx,
                    ctx.ui_snapshot,
                    &mut actions,
                    ctx.file_association_icons,
                );
                crate::ui::shortcuts::render_shortcuts(egui_ctx, ctx.ui_snapshot, &mut actions);
            }

            if let Some(msg) = ctx.text {
                egui::Area::new(Id::new("toast_overlay"))
                    .anchor(Align2::CENTER_BOTTOM, [0.0, -20.0])
                    .interactable(false)
                    .show(egui_ctx, |ui| {
                        Frame::new()
                            .fill(Color32::from_black_alpha(180))
                            .corner_radius(8.0)
                            .inner_margin(Margin::symmetric(14, 8))
                            .show(ui, |ui| {
                                ui.set_max_width(
                                    (ctx.window.inner_size().width as f32 * 0.85).max(600.0),
                                );
                                ui.label(RichText::new(msg).color(Color32::WHITE).size(18.0).strong());
                            });
                    });
            }

            if let Some(msg) = &ctx.warning_text {
                egui::Area::new(Id::new("warning_overlay"))
                    .anchor(Align2::CENTER_TOP, [0.0, 20.0])
                    .interactable(false)
                    .show(egui_ctx, |ui| {
                        Frame::new()
                            .fill(Color32::from_rgb(180, 60, 0)) // Distinct warning color
                            .corner_radius(8.0)
                            .inner_margin(Margin::symmetric(14, 8))
                            .show(ui, |ui| {
                                ui.set_max_width(
                                    (ctx.window.inner_size().width as f32 * 0.85).max(600.0),
                                );
                                ui.label(
                                    RichText::new(msg).color(Color32::WHITE).strong().size(16.0),
                                );
                            });
                    });
            }

            if ctx.ui_snapshot.show_loading_spinner {
                render_loading_spinner(egui_ctx, ctx.ui_snapshot.accent_color);
            }
        });

        self.state
            .handle_platform_output(ctx.window, output.platform_output.clone());

        let repaint_requested = output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|v| v.repaint_delay.is_zero())
            .unwrap_or(false);

        let clipped_primitives = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);

        for (id, image_delta) in &output.textures_delta.set {
            self.renderer
                .update_texture(ctx.device, ctx.queue, *id, image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                ctx.window.inner_size().width,
                ctx.window.inner_size().height,
            ],
            pixels_per_point: ctx.window.scale_factor() as f32,
        };

        self.renderer.update_buffers(
            ctx.device,
            ctx.queue,
            ctx.encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let mut render_pass = ctx
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Egui Toast Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: ctx.target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .forget_lifetime();

            self.renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }

        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        (actions, repaint_requested)
    }
}

fn render_loading_spinner(ctx: &egui::Context, _accent_color: Option<egui::Color32>) {
    egui::Area::new(egui::Id::new("loading_spinner"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .interactable(false)
        .show(ctx, |ui| {
            let speed = 4.0;
            let time = ui.input(|i| i.time) as f32;

            let size = 80.0;
            let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
            let center = rect.center();

            let num_dots = 12;
            let radius = 30.0;

            for i in 0..num_dots {
                let factor = i as f32 / num_dots as f32;
                let angle = factor * std::f32::consts::TAU + (time * speed);

                let pos = center + egui::vec2(angle.cos(), angle.sin()) * radius;

                // Color wheel logic using HSL
                // Hue shifts based on the dot's position to create a rainbow circle
                let hue = factor;
                let dot_color = egui::Color32::from(egui::ecolor::Hsva::new(hue, 0.8, 1.0, 1.0));

                // Pulsing dot size based on position to add dynamic feel
                let dot_pulse = (angle * 0.5).sin() * 0.2 + 0.8;
                let dot_size = 4.0 * dot_pulse;

                ui.painter().circle_filled(pos, dot_size, dot_color);
            }
        });
    ctx.request_repaint();
}
