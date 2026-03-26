use glam::{Mat4, Vec2, Vec3};

/// Camera for 2D world space → clip space.
///
/// ### Coordinate system (WORLD SPACE)
/// - Unit: **logical pixels** (image pixels at zoom = 1.0)
/// - Origin: free (decided by layout), but shared across:
///   - `layout::PagePlacement.position` (bottom-left corner of a page)
///   - `quad_batch::create_quad_data` vertex positions
///   - `Camera.pan`
/// - Axes:
///   - +X: right
///   - +Y: **up**
///
/// ### Screen / clip space
/// - `window_size` is in physical pixels (from winit).
/// - We build an **orthographic projection** where:
///   - visible area in world space is roughly `[-W/2..W/2] x [-H/2..H/2]`
///   - then transformed into WGPU NDC (X-right, Y-up, Z-forward).
///
/// In short:
///   `world (layout, pages) → Camera (zoom, pan, window_size) → clip (shader)`
pub struct Camera {
    pub zoom: f32,
    pub pan: Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }

    /// Build a view-projection matrix that transforms WORLD coordinates → CLIP coordinates.
    ///
    /// WORLD:
    ///   - X-right, Y-up
    ///   - units are logical pixels (image pixel size at zoom = 1.0)
    ///
    /// SCREEN:
    ///   - `window_size` is in physical pixels (from winit).
    ///
    /// The orthographic projection maps:
    ///   x ∈ [-W/2, W/2], y ∈ [-H/2, H/2]  →  clip space in [-1, 1]²,
    /// then applies view (pan, zoom).
    pub fn build_view_projection(&self, window_size: (u32, u32)) -> Mat4 {
        let (width, height) = (window_size.0 as f32, window_size.1 as f32);
        if width == 0.0 || height == 0.0 {
            return Mat4::IDENTITY;
        }

        // Orthographic projection for 2D screen coordinates.
        // Adjusts for WGPU NDC (Y-up) by inverting top and bottom values.
        let projection = Mat4::orthographic_lh(
            -width / 2.0,
            width / 2.0,
            -height / 2.0, // bottom
            height / 2.0,  // top
            -1000.0,
            1000.0,
        );

        // View matrix: translate to camera center, then apply zoom.
        // Order is important: we want (world - pan) * zoom.
        let view = Mat4::from_scale(Vec3::new(self.zoom, self.zoom, 1.0))
            * Mat4::from_translation(Vec3::new(-self.pan.x, -self.pan.y, 0.0));

        projection * view
    }

    /// Converts screen pixel coordinates to world coordinates.
    /// Useful for mapping mouse input to page locations.
    #[allow(dead_code)]
    pub fn screen_to_world(&self, screen_pos: Vec2, window_size: (u32, u32)) -> Vec2 {
        let (width, height) = (window_size.0 as f32, window_size.1 as f32);

        // Convert screen (0..width, 0..height) to centered space (-w/2..w/2, -h/2..h/2)
        let centered_pos = Vec2::new(screen_pos.x - width / 2.0, screen_pos.y - height / 2.0);

        // Apply inverse zoom and add pan
        (centered_pos / self.zoom) + self.pan
    }
}
