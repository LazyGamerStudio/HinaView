use crate::app::App;
use crate::runtime::window_state::WindowState;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::error;
use winit::window::Window;

fn try_rebuild_renderer(app: &mut App, window: &Arc<Window>) -> bool {
    match pollster::block_on(crate::bootstrap::graphics::rebuild_renderer(window.clone())) {
        Ok(resources) => {
            app.apply_renderer_recovery(window, resources.renderer, resources.estimated_vram_mb);
            true
        }
        Err(e) => {
            error!("[Render] Failed to rebuild renderer after device loss: {e}");
            false
        }
    }
}

pub fn handle_redraw(
    state: &Option<Arc<RwLock<App>>>,
    window: &Option<Arc<Window>>,
    window_state: &mut WindowState,
) {
    if let (Some(state), Some(window)) = (state, window) {
        if !window_state.is_visible {
            return;
        }

        let mut app = state.write();
        let redraw_start = std::time::Instant::now();

        if window.is_minimized().unwrap_or(false) {
            return;
        }

        let new_title = app.compute_window_title();
        if app.last_window_title != new_title {
            window.set_title(&new_title);
            app.last_window_title = new_title;
        }

        let physical_size = window.inner_size();
        if physical_size.width == 0 || physical_size.height == 0 {
            return;
        }

        if app.window_size != (physical_size.width, physical_size.height) {
            app.window_size = (physical_size.width, physical_size.height);
            let ws = app.window_size;
            app.nav.refresh_layout(ws);
            app.needs_visible_check = true;
        }

        let callback_device_lost = app
            .renderer
            .as_ref()
            .map(|r| r.take_device_lost())
            .unwrap_or(false);
        let recovery_requested = app.take_renderer_recovery_request() || callback_device_lost;
        if recovery_requested {
            if try_rebuild_renderer(&mut app, window) {
                window.request_redraw();
            }
            return;
        }

        let _gpu_updated = app.update();
        let toast_text = app.toast_overlay.current_text().map(|s| s.to_string());
        let warning_text = app.warning_overlay.current_text();
        let ui_snapshot = app.ui_snapshot();
        let mut toast_renderer_opt = app.toast_renderer.take();
        let mut ui_actions = Vec::new();
        let mut recover_renderer = false;

        let mut egui_repaint_requested = false;
        let needs_texture_poll = if let Some(mut renderer) = app.renderer.take() {
            let (color_matrix, icc_gamma) = app.current_color_management_params();
            renderer.set_filter_params(app.current_filter_params(), icc_gamma, color_matrix);

            // Temporarily take ownership of file_association_icons to avoid borrow conflicts
            let mut file_association_icons = std::mem::take(&mut app.file_association_icons);

            let result = match renderer.render_frame_with_overlay(
                &app.texture_manager,
                app.window_size,
                &app.nav,
                |device, queue, encoder, view| {
                    if let Some(toast_renderer) = toast_renderer_opt.as_mut() {
                        let (actions, repaint) =
                            toast_renderer.paint(crate::ui_overlay::EguiRenderContext {
                                window,
                                device,
                                queue,
                                encoder,
                                target_view: view,
                                ui_snapshot: &ui_snapshot,
                                text: toast_text.as_deref(),
                                warning_text: warning_text.clone(),
                                file_association_icons: &mut file_association_icons,
                            });
                        ui_actions = actions;
                        egui_repaint_requested = repaint;
                    }
                },
            ) {
                Ok(needs_poll) => needs_poll,
                Err(wgpu::SurfaceError::Outdated) => {
                    renderer.resize(app.window_size);
                    false
                }
                Err(wgpu::SurfaceError::Timeout) => false,
                Err(wgpu::SurfaceError::Lost) => {
                    error!("[Render] Surface lost, scheduling full renderer rebuild");
                    recover_renderer = true;
                    false
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    error!("[Render] GPU out of memory, scheduling full renderer rebuild");
                    recover_renderer = true;
                    false
                }
                Err(wgpu::SurfaceError::Other) => {
                    error!("[Render] Surface acquisition returned generic error");
                    recover_renderer = true;
                    false
                }
            };
            // Put the icons back
            app.file_association_icons = file_association_icons;
            if recover_renderer {
                app.request_renderer_recovery();
            } else {
                app.renderer = Some(renderer);
            }
            app.toast_renderer = toast_renderer_opt;
            for action in ui_actions.drain(..) {
                app.handle_ui_action(action);
            }
            result
        } else {
            app.toast_renderer = toast_renderer_opt;
            false
        };

        // Mark this frame as rendered (attempted)
        window_state.last_redraw_instant = redraw_start;

        if app.wants_idle_ui_redraw()
            || recover_renderer
            || needs_texture_poll
            || egui_repaint_requested
        {
            window.request_redraw();
        }
    }
}
