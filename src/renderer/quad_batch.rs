// src/renderer/quad_batch.rs
use crate::view::RotationQuarter;
use bytemuck::{Pod, Zeroable};

/// Single vertex used by the 2D textured-quad pipeline.
///
/// ### Coordinate system
/// - `position` is in **WORLD space**:
///   - Same as `PagePlacement.position`
///   - +X: right, +Y: up, unit: logical pixels
/// - `tex_coords` are in texture UV space:
///   - (0,0) = top-left of the image
///   - (1,1) = bottom-right of the image
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Creates vertices and indices for a single quad (page).
///
/// - `pos`: bottom-left corner of the quad in WORLD space.
/// - `size`: (width, height) of the quad in WORLD units.
fn rotate_point(position: [f32; 2], rotation: RotationQuarter, pivot: [f32; 2]) -> [f32; 2] {
    let x = position[0] - pivot[0];
    let y = position[1] - pivot[1];
    let (rx, ry) = match rotation {
        RotationQuarter::Deg0 => (x, y),
        RotationQuarter::Deg90 => (y, -x),   // Clockwise 90
        RotationQuarter::Deg180 => (-x, -y), // 180 is the same either way
        RotationQuarter::Deg270 => (-y, x),  // Clockwise 270 (or CCW 90)
    };
    [rx + pivot[0], ry + pivot[1]]
}

pub fn create_quad_data(
    pos: [f32; 2],
    size: [f32; 2],
    rotation: RotationQuarter,
    pivot: [f32; 2],
) -> (Vec<Vertex>, Vec<u16>) {
    let (x, y) = (pos[0], pos[1]);
    let (w, h) = (size[0], size[1]);

    let bl = rotate_point([x, y], rotation, pivot);
    let tl = rotate_point([x, y + h], rotation, pivot);
    let tr = rotate_point([x + w, y + h], rotation, pivot);
    let br = rotate_point([x + w, y], rotation, pivot);

    let vertices = vec![
        Vertex {
            position: [bl[0], bl[1], 0.0],
            tex_coords: [0.0, 1.0],
        }, // Bottom-Left
        Vertex {
            position: [tl[0], tl[1], 0.0],
            tex_coords: [0.0, 0.0],
        }, // Top-Left
        Vertex {
            position: [tr[0], tr[1], 0.0],
            tex_coords: [1.0, 0.0],
        }, // Top-Right
        Vertex {
            position: [br[0], br[1], 0.0],
            tex_coords: [1.0, 1.0],
        }, // Bottom-Right
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    (vertices, indices)
}
