// src/system/single_instance.rs
// Single instance enforcement and inter-process communication (IPC) via Named Pipes

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::sync::Arc;
use std::thread;

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, ERROR_PIPE_CONNECTED, GENERIC_WRITE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, ReadFile, WriteFile,
};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
    PIPE_WAIT,
};
use windows_sys::Win32::System::Threading::CreateMutexW;

const MUTEX_NAME: &str = "Global\\HinaView_SingleInstance_Mutex";
const PIPE_NAME: &str = "\\\\.\\pipe\\HinaView_Command_Pipe";

// Pipe access flags (from Win32 API)
const PIPE_ACCESS_INBOUND: u32 = 1;

/// Helper to convert string to null-terminated wide string
fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Checks if another instance is already running.
/// If so, sends the file path to it and returns true (indicating this instance should exit).
/// If not, returns false (indicating this is the first instance).
pub fn handle_single_instance(file_to_open: Option<String>) -> bool {
    let mutex_name_wide = to_wide_null(MUTEX_NAME);

    unsafe {
        // SAFETY: The mutex name is null-terminated and lives for the duration of the call.
        // 1. Try to create/open the named mutex
        let h_mutex = CreateMutexW(null_mut(), 0, mutex_name_wide.as_ptr());

        if h_mutex.is_null() {
            tracing::error!("Failed to create mutex");
            return false;
        }

        // 2. Check if mutex already exists (another instance is running)
        if std::io::Error::last_os_error().raw_os_error() == Some(ERROR_ALREADY_EXISTS as i32) {
            tracing::info!("Another instance is already running. Sending path...");

            // 3. If there's a file to open, send it via Named Pipe
            if let Some(path) = file_to_open {
                send_to_existing_instance(&path);
            }
            return true; // We should exit
        }
    }

    // Keep the mutex handle alive for the duration of the process
    false
}

/// Sends a file path to the existing instance via Named Pipe
fn send_to_existing_instance(path: &str) {
    let pipe_name_wide = to_wide_null(PIPE_NAME);
    let path_bytes = path.as_bytes();

    unsafe {
        // SAFETY: The pipe name is a stable null-terminated UTF-16 buffer for the call.
        let h_pipe = CreateFileW(
            pipe_name_wide.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        );

        if h_pipe != INVALID_HANDLE_VALUE {
            let mut bytes_written = 0;
            // SAFETY: `path_bytes` points to the UTF-8 payload we want to send and remains valid
            // until WriteFile returns.
            WriteFile(
                h_pipe,
                path_bytes.as_ptr(),
                path_bytes.len() as u32,
                &mut bytes_written,
                null_mut(),
            );
            // SAFETY: `h_pipe` is a valid handle from CreateFileW above.
            CloseHandle(h_pipe);
        }
    }
}

/// Starts a background thread to listen for commands from other instances
pub fn start_pipe_server<T>(proxy: Arc<winit::event_loop::EventLoopProxy<T>>)
where
    T: From<String> + Send + Sync + 'static,
{
    thread::spawn(move || {
        let pipe_name_wide = to_wide_null(PIPE_NAME);

        loop {
            unsafe {
                // SAFETY: The pipe name is a valid null-terminated UTF-16 buffer for the call.
                let h_pipe = CreateNamedPipeW(
                    pipe_name_wide.as_ptr(),
                    PIPE_ACCESS_INBOUND,
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                    1,    // Max instances
                    1024, // Out buffer
                    1024, // In buffer
                    0,    // Default timeout
                    null_mut(),
                );

                if h_pipe == INVALID_HANDLE_VALUE {
                    tracing::error!("Failed to create named pipe");
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }

                // Wait for a client to connect
                let connected = ConnectNamedPipe(h_pipe, null_mut());
                let error = if connected == 0 {
                    std::io::Error::last_os_error().raw_os_error()
                } else {
                    None
                };

                if connected != 0 || error == Some(ERROR_PIPE_CONNECTED as i32) {
                    let mut buffer = [0u8; 1024];
                    let mut bytes_read = 0;

                    // SAFETY: `buffer` is a writable stack buffer that remains valid until ReadFile returns.
                    if ReadFile(
                        h_pipe,
                        buffer.as_mut_ptr() as *mut _,
                        buffer.len() as u32,
                        &mut bytes_read,
                        null_mut(),
                    ) != 0
                    {
                        if bytes_read > 0 {
                            if let Ok(path) =
                                String::from_utf8(buffer[..bytes_read as usize].to_vec())
                            {
                                tracing::info!("Received path from pipe: {}", path);
                                let _ = proxy.send_event(path.into());
                            }
                        }
                    }
                }

                // SAFETY: `h_pipe` is a live pipe handle created by CreateNamedPipeW above.
                DisconnectNamedPipe(h_pipe);
                // SAFETY: `h_pipe` is a live handle that must be closed once after use.
                CloseHandle(h_pipe);
            }
        }
    });
}
