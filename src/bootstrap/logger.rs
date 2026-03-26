use tracing::{error, info, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::prelude::*;

fn prune_old_logs(log_dir: &std::path::Path, keep_latest: usize) {
    let mut entries: Vec<(std::path::PathBuf, std::time::SystemTime)> =
        match std::fs::read_dir(log_dir) {
            Ok(read_dir) => read_dir
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with("HinaView-") && n.ends_with(".log"))
                        .unwrap_or(false)
                })
                .filter_map(|path| {
                    let modified = std::fs::metadata(&path).ok()?.modified().ok()?;
                    Some((path, modified))
                })
                .collect(),
            Err(_) => return,
        };

    if entries.len() <= keep_latest {
        return;
    }

    entries.sort_by_key(|(_, modified)| *modified);
    let remove_count = entries.len().saturating_sub(keep_latest);
    for (path, _) in entries.into_iter().take(remove_count) {
        if let Err(e) = std::fs::remove_file(&path) {
            warn!("[Logger] Failed to remove old log {:?}: {}", path, e);
        }
    }
}

pub fn init_logger(debug_mode: bool) -> Option<WorkerGuard> {
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing_subscriber::EnvFilter;

    // If not debug mode, don't create file logger
    if !debug_mode {
        let console_filter =
            EnvFilter::new("HinaView=warn,wgpu=error,naga=error,egui=warn,winit=warn,glow=warn");

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(true)
                    .with_target(true)
                    .with_filter(console_filter),
            )
            .init();

        return None;
    }

    let log_dir = match std::env::current_exe() {
        Ok(exe_path) => exe_path
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .join("logs"),
        Err(_) => std::path::PathBuf::from("logs"),
    };

    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        error!("[Logger] Failed to create log directory: {}", e);
    }

    prune_old_logs(&log_dir, 10);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let log_filename = format!("HinaView-{}.log", timestamp);

    let file_appender = tracing_appender::rolling::never(&log_dir, &log_filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Debug mode: detailed logs for everything
    let console_filter_str = "HinaView=debug,wgpu=warn,naga=warn,egui=warn,winit=warn,glow=warn";
    let file_filter_str = "HinaView=debug,wgpu=warn,naga=warn,egui=warn,winit=warn,glow=warn";

    let console_filter = EnvFilter::new(console_filter_str);
    let file_filter = EnvFilter::new(file_filter_str);

    tracing_subscriber::registry()
        // Console Layer
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_target(true)
                .with_filter(console_filter),
        )
        // File Layer
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_filter(file_filter),
        )
        .init();

    info!(
        "Logger initialized (Debug: {}). Log file: {}",
        debug_mode,
        log_dir.join(log_filename).display()
    );
    Some(guard)
}
