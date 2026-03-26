#![windows_subsystem = "windows"]

pub mod app;
mod bookmark;
mod bootstrap;
mod cache;
mod camera;
mod color_management;
mod config;
mod database;
mod document;
mod filter;
mod i18n;
mod input;
mod layout;
mod pipeline;
mod renderer;
mod runtime;
mod sampling;
mod settings;
mod slideshow;
mod system;
mod types;
mod ui;
mod ui_overlay;
mod updater;
mod util;
mod view;

use app::App;
use bootstrap::graphics::init_app_state;
use bootstrap::logger::init_logger;
use config::app_config::AppConfig;
use config::store::load_config;
use parking_lot::RwLock;
use runtime::event_router::route_window_event;
use runtime::window_state::WindowState;
use std::sync::Arc;
use tracing_appender::non_blocking::WorkerGuard;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

/// Custom events for the application
#[derive(Debug, Clone)]
enum UserEvent {
    OpenFile(String),
}

impl From<String> for UserEvent {
    fn from(path: String) -> Self {
        UserEvent::OpenFile(path)
    }
}

struct HinaViewApp {
    state: Option<Arc<RwLock<App>>>,
    window: Option<Arc<Window>>,
    window_state: WindowState,
    config: AppConfig,
    first_frame_rendered: bool,
    pending_file_to_open: Option<String>,
}

impl HinaViewApp {
    fn new() -> Self {
        Self {
            state: None,
            window: None,
            window_state: WindowState::new(),
            config: load_config(),
            first_frame_rendered: false,
            pending_file_to_open: None,
        }
    }

    async fn init_graphics(&mut self, window: Arc<Window>) {
        let mut app = init_app_state(window.clone(), self.config.settings.config_storage_location).await;
        app.set_locale(&self.config.locale);
        app.apply_settings_state(self.config.settings.clone());

        self.state = Some(Arc::new(RwLock::new(app)));
        self.window = Some(window);

        // Spawn background update worker
        crate::updater::worker::spawn_update_worker();
    }
}

fn load_icon() -> Option<winit::window::Icon> {
    // Include embedded icon directly from binary resources
    let icon_data = include_bytes!("../icon_win/hinaview.ico");

    // Parse ICO file using image crate (already supports ICO format)
    image::load_from_memory_with_format(icon_data, image::ImageFormat::Ico)
        .ok()
        .and_then(|img| {
            let rgba = img.into_rgba8();
            let (width, height) = rgba.dimensions();
            winit::window::Icon::from_rgba(rgba.into_raw(), width, height).ok()
        })
}

impl ApplicationHandler<UserEvent> for HinaViewApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let icon = load_icon();

            let window_attributes = Window::default_attributes()
                .with_title("Hina View")
                .with_inner_size(winit::dpi::PhysicalSize::new(
                    self.config.window.width,
                    self.config.window.height,
                ))
                .with_position(winit::dpi::PhysicalPosition::new(
                    self.config.window.x,
                    self.config.window.y,
                ))
                .with_window_icon(icon)
                .with_visible(false);

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create window"),
            );

            pollster::block_on(self.init_graphics(window.clone()));

            window.set_visible(true);
            self.first_frame_rendered = true;

            // Open pending file if any
            if let Some(file_path) = self.pending_file_to_open.take() {
                if let Some(state) = self.state.as_ref() {
                    let mut app = state.write();
                    app.open_file_from_path(&file_path);
                }
            }

            window.request_redraw();
        } else {
            if let Some(state) = self.state.as_ref() {
                let mut app = state.write();
                app.request_renderer_recovery();
            }
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        route_window_event(
            &self.state,
            &self.window,
            &mut self.window_state,
            &mut self.config,
            event_loop,
            event,
        );
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::OpenFile(path) => {
                tracing::info!("Opening file from user event: {}", path);
                if let Some(state) = self.state.as_ref() {
                    let mut app = state.write();
                    app.open_file_from_path(&path);
                    if let Some(window) = self.window.as_ref() {
                        window.set_minimized(false);
                        window.focus_window();
                        window.request_redraw();
                    }
                } else {
                    // Graphics not initialized yet, store as pending
                    self.pending_file_to_open = Some(path);
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(state) = self.state.as_ref() else {
            return;
        };

        if !self.window_state.is_visible {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(500),
            ));
            return;
        }

        self.window_state.hide_cursor_if_idle(window);

        let should_redraw = {
            let app = state.read();
            (
                app.wants_idle_ui_redraw(),
                app.next_animation_redraw_deadline(),
                app.next_ui_auto_hide_deadline(),
                self.window_state.next_cursor_hide_deadline(),
            )
        };

        if should_redraw.0 {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(1),
            ));
            window.request_redraw();
        } else if let Some(deadline) = [should_redraw.1, should_redraw.2, should_redraw.3]
            .into_iter()
            .flatten()
            .min()
        {
            if deadline <= std::time::Instant::now() {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    std::time::Instant::now() + std::time::Duration::from_millis(1),
                ));
                if should_redraw.1.is_some() || should_redraw.2.is_some() {
                    window.request_redraw();
                }
            } else {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(deadline));
            }
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(100),
            ));
        }
    }
}

fn main() {
    if crate::updater::try_start_pending_update() {
        return;
    }

    // Clean up leftover files from previous update (e.g., updater.exe.old)
    crate::updater::cleanup_old_updater();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("lib");

        if path.exists() {
            // 1. Add to PATH environment variable (most reliable for dependency chains)
            if let Ok(existing_path) = std::env::var("PATH") {
                let new_path = format!("{};{}", path.display(), existing_path);
                unsafe {
                    // SAFETY: Updating the current process environment is intended here before
                    // worker threads are spawned, so no concurrent environment access occurs.
                    std::env::set_var("PATH", new_path);
                }
            }

            // 2. Also set DLL directory for LoadLibrary calls
            let mut wide_path: Vec<u16> = path.as_os_str().encode_wide().collect();
            wide_path.push(0);
            unsafe {
                // SAFETY: `wide_path` is null-terminated and lives for the duration of the call.
                windows_sys::Win32::System::LibraryLoader::SetDllDirectoryW(wide_path.as_ptr());
            }
        }
    }

    let args: Vec<String> = std::env::args().collect();
    let debug_mode = args.iter().any(|arg| arg == "--debug" || arg == "-d");

    // Find file path from command line (non-flag arguments)
    let file_to_open = args
        .iter()
        .skip(1) // Skip executable name
        .find(|arg| !arg.starts_with('-'))
        .cloned();

    // Load config first to check single instance setting
    let config = load_config();

    // Check for single instance IF enabled in settings
    if config.settings.single_instance {
        if crate::system::single_instance::handle_single_instance(file_to_open.clone()) {
            // Another instance is running and handled the file, so we exit
            return;
        }
    }

    if debug_mode {
        unsafe {
            use windows_sys::Win32::System::Console::{
                ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole,
            };
            // SAFETY: Console attach/allocation only affects the current process and uses no
            // borrowed pointers. We fall back to AllocConsole only if attach fails.
            if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
                AllocConsole();
            }
        }
    }

    let _guard: Option<WorkerGuard> = init_logger(debug_mode);

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to build event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    // Start Pipe Server only if single instance is enabled
    if config.settings.single_instance {
        let proxy = Arc::new(event_loop.create_proxy());
        crate::system::single_instance::start_pipe_server(proxy);
    }

    let mut application = HinaViewApp::new();
    // Use the already loaded config
    application.config = config;

    // Store file path to open after initialization
    if let Some(file_path) = file_to_open {
        application.pending_file_to_open = Some(file_path);
    }

    event_loop
        .run_app(&mut application)
        .expect("Failed to run app");
}
