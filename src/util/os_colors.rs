use egui::Color32;

#[cfg(target_os = "windows")]
pub fn get_windows_accent_color() -> Option<Color32> {
    use windows_sys::Win32::Graphics::Dwm::DwmGetColorizationColor;

    let mut color: u32 = 0;
    let mut opaque: i32 = 0;

    // SAFETY: DwmGetColorizationColor writes to the provided out-pointers for the duration
    // of the call, and both pointers refer to valid stack locals.
    let hr = unsafe { DwmGetColorizationColor(&mut color, &mut opaque) };

    if hr == 0 {
        // color is ARGB
        let a = ((color >> 24) & 0xFF) as u8;
        let r = ((color >> 16) & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = (color & 0xFF) as u8;

        // We usually want full opacity for the progress bar if the system color is too transparent
        Some(Color32::from_rgba_unmultiplied(r, g, b, a.max(128)))
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_windows_accent_color() -> Option<Color32> {
    None
}

pub fn find_closest_basic_color(target: Color32) -> Color32 {
    let basics = [
        Color32::RED,
        Color32::GREEN,
        Color32::BLUE,
        Color32::YELLOW,
        Color32::GOLD,
        Color32::KHAKI,
        Color32::BROWN,
    ];

    let mut closest = basics[0];
    let mut min_dist_sq = f32::MAX;

    let tr = target.r() as f32;
    let tg = target.g() as f32;
    let tb = target.b() as f32;

    for &b in &basics {
        let dr = tr - b.r() as f32;
        let dg = tg - b.g() as f32;
        let db = tb - b.b() as f32;
        let dist_sq = dr * dr + dg * dg + db * db;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            closest = b;
        }
    }

    closest
}
