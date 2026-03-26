// src/layout/placement.rs
/// Physical placement of a **single logical page** in world space.
///
/// ### Coordinate system
/// - Same WORLD space as `camera::Camera` expects:
///   - +X: right
///   - +Y: up
///   - unit: logical pixels (image pixels at zoom = 1.0)
///
/// ### Semantics
/// - `position` is the **bottom-left corner** of the rendered quad in world space.
/// - `size` is the width/height in world units (typically original image width/height).
///
/// The renderer (`quad_batch`) uses this directly to build vertex positions.
pub struct PagePlacement {
    pub page_index: usize,
    pub position: [f32; 2],
    pub size: [f32; 2],
}
