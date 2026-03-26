// src/color_management/display_profile.rs

#[cfg(windows)]
pub fn detect_display_profile() -> Option<(String, Vec<u8>)> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;
    use windows_sys::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
    use windows_sys::Win32::UI::ColorSystem::GetICMProfileW;

    unsafe {
        // SAFETY: We request the desktop DC with a null HWND and release it on all paths below.
        let hdc = GetDC(std::ptr::null_mut());
        if hdc.is_null() {
            return None;
        }

        let mut len: u32 = 260;
        let mut buf: Vec<u16> = vec![0u16; len as usize];
        // SAFETY: `buf` is allocated for `len` UTF-16 code units and remains valid for the call.
        let ok = GetICMProfileW(hdc, &mut len, buf.as_mut_ptr());
        // SAFETY: `hdc` was returned by GetDC above and must be paired with ReleaseDC once.
        let _ = ReleaseDC(std::ptr::null_mut(), hdc);
        if ok == 0 || len == 0 {
            return None;
        }

        let slice_len = len.saturating_sub(1) as usize;
        let profile_path = OsString::from_wide(&buf[..slice_len]);
        let profile_path_str = profile_path.to_string_lossy().to_string();

        let file_name = Path::new(&profile_path)
            .file_stem()
            .and_then(|v| v.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| profile_path_str.clone());

        let icc_data = std::fs::read(&profile_path).ok()?;

        Some((file_name, icc_data))
    }
}

#[cfg(not(windows))]
pub fn detect_display_profile() -> Option<(String, Vec<u8>)> {
    None
}
