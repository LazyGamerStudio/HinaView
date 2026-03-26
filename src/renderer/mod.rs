// src/renderer/mod.rs
pub mod quad_batch;

use quad_batch::Vertex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::error;

/// Camera uniform structure for GPU buffer transfer.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FilterUniform {
    /// 3x3 Color conversion matrix for gamut mapping.
    /// ALIGNMENT: Represented as [[f32; 4]; 3] to match WGSL mat3x3<f32> alignment,
    /// where each column is 16-byte aligned (padded with an unused f32).
    /// NECESSITY: Required for real-time gamut conversion from image space to display space.
    pub color_matrix: [[f32; 4]; 3],

    pub bright: f32,
    pub contrast: f32,
    pub gamma: f32,
    pub exposure: f32,

    pub fsr_enabled: f32,
    pub icc_gamma: f32,
    pub fsr_sharpness: f32,
    pub median_enabled: f32,

    pub median_strength: f32,
    pub median_stride: f32,
    pub blur_radius: f32,
    pub unsharp_amount: f32,

    pub unsharp_threshold: f32,
    pub levels_in_black: f32,
    pub levels_in_white: f32,
    pub levels_gamma: f32,

    pub levels_out_black: f32,
    pub levels_out_white: f32,
    pub bypass_color: f32,
    pub bypass_median: f32,

    pub bypass_fsr: f32,
    pub bypass_detail: f32,
    pub bypass_levels: f32,
    pub _pad0: f32,
}

/// A single tile of a decoded page.
pub struct GpuTile {
    pub texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
    pub rect: crate::types::TileRect,
}

/// Encapsulates GPU texture resources for a decoded page.
pub struct GpuImage {
    pub tiles: Vec<GpuTile>,
    pub width: u32,
    pub height: u32,
    pub mip: crate::types::MipLevel,
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    vertex_buffer_size: u64,
    index_buffer_size: u64,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub filter_buffer: wgpu::Buffer,
    pub filter_bind_group: wgpu::BindGroup,
    last_camera_uniform: Option<CameraUniform>,
    device_lost: Arc<AtomicBool>,
}

impl Renderer {
    fn should_render_page(
        nav: &crate::view::NavigationController,
        window_size: (u32, u32),
        current_page: usize,
        placement: &crate::layout::PagePlacement,
    ) -> bool {
        match nav.view.layout_mode {
            crate::types::LayoutMode::Dual { .. } => {
                let Some(doc) = nav.document.as_ref() else {
                    return placement.page_index == current_page;
                };

                // Render both current page and target page to prevent flickering
                let is_current_page = placement.page_index == current_page;
                let is_target_page = nav.target_page == Some(placement.page_index);

                if is_current_page || is_target_page {
                    return true;
                }

                // Also render pages in the same spread
                if let Some(spread) = doc
                    .spreads
                    .iter()
                    .find(|s| s.left == Some(current_page) || s.right == Some(current_page))
                {
                    return spread.left == Some(placement.page_index)
                        || spread.right == Some(placement.page_index);
                }

                placement.page_index == current_page
            }
            crate::types::LayoutMode::VerticalScroll => {
                let zoom = nav.camera.zoom.max(0.0001);
                let half_w = window_size.0 as f32 / (2.0 * zoom);
                let half_h = window_size.1 as f32 / (2.0 * zoom);
                let margin_y = half_h; // one extra viewport height

                let view_min_x = nav.camera.pan.x - half_w;
                let view_max_x = nav.camera.pan.x + half_w;
                let view_min_y = nav.camera.pan.y - half_h - margin_y;
                let view_max_y = nav.camera.pan.y + half_h + margin_y;

                let page_min_x = placement.position[0];
                let page_max_x = placement.position[0] + placement.size[0];
                let page_min_y = placement.position[1];
                let page_max_y = placement.position[1] + placement.size[1];

                !(page_max_x < view_min_x
                    || page_min_x > view_max_x
                    || page_max_y < view_min_y
                    || page_min_y > view_max_y)
            }
            _ => placement.page_index == current_page,
        }
    }

    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("HinaView_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera_BindGroupLayout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture_BindGroupLayout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let filter_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Filter_BindGroupLayout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("RenderPipeline_Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_bind_group_layout,
                &filter_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render_Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling to render both front and back faces
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        use wgpu::util::DeviceExt;

        let camera_uniform = CameraUniform {
            view_proj: [[0.0; 4]; 4], // Initialized with dummy, updated in render_frame
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera_Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Camera_BindGroup"),
        });

        let filter_uniform = FilterUniform {
            color_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
            bright: 0.0,
            contrast: 1.0,
            gamma: 1.0,
            exposure: 0.0,
            fsr_enabled: 0.0,
            icc_gamma: 1.0,
            fsr_sharpness: 0.2,
            median_enabled: 0.0,
            median_strength: 0.0,
            median_stride: 1.0,
            blur_radius: 0.0,
            unsharp_amount: 0.0,
            unsharp_threshold: 0.05,
            levels_in_black: 0.0,
            levels_in_white: 1.0,
            levels_gamma: 1.0,
            levels_out_black: 0.0,
            levels_out_white: 1.0,
            bypass_color: 0.0,
            bypass_median: 0.0,
            bypass_fsr: 0.0,
            bypass_detail: 0.0,
            bypass_levels: 0.0,
            _pad0: 0.0,
        };
        let filter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Filter_Buffer"),
            contents: bytemuck::cast_slice(&[filter_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let filter_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &filter_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: filter_buffer.as_entire_binding(),
            }],
            label: Some("Filter_BindGroup"),
        });

        let initial_vertex_size = 1024 * 1024; // 1MB initial
        let initial_index_size = 1024 * 512; // 512KB initial

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex_Buffer"),
            size: initial_vertex_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index_Buffer"),
            size: initial_index_size,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let device_lost = Arc::new(AtomicBool::new(false));
        let device_lost_flag = device_lost.clone();
        device.set_device_lost_callback(move |reason, message| {
            error!(
                "[Render] Device lost callback: reason={:?}, message={}",
                reason, message
            );
            device_lost_flag.store(true, Ordering::SeqCst);
        });

        Self {
            device,
            queue,
            surface,
            surface_config,
            pipeline,
            camera_bind_group_layout,
            texture_bind_group_layout,
            sampler,
            vertex_buffer,
            index_buffer,
            vertex_buffer_size: initial_vertex_size,
            index_buffer_size: initial_index_size,
            camera_buffer,
            camera_bind_group,
            filter_buffer,
            filter_bind_group,
            last_camera_uniform: None,
            device_lost,
        }
    }

    pub fn take_device_lost(&self) -> bool {
        self.device_lost.swap(false, Ordering::SeqCst)
    }

    pub fn set_filter_params(
        &self,
        params: crate::filter::FilterParams,
        icc_gamma: f32,
        color_matrix: [[f32; 4]; 3],
    ) {
        let uniform = FilterUniform {
            color_matrix,
            bright: params.bright,
            contrast: params.contrast,
            gamma: params.gamma,
            exposure: params.exposure,

            fsr_enabled: if params.fsr_enabled { 1.0 } else { 0.0 },
            icc_gamma,
            fsr_sharpness: params.fsr_sharpness,
            median_enabled: if params.median_strength > 0.001 {
                1.0
            } else {
                0.0
            },

            median_strength: params.median_strength,
            median_stride: params.median_stride,
            blur_radius: params.blur_radius,
            unsharp_amount: params.unsharp_amount,

            unsharp_threshold: params.unsharp_threshold,
            levels_in_black: params.levels_in_black,
            levels_in_white: params.levels_in_white,
            levels_gamma: params.levels_gamma,

            levels_out_black: params.levels_out_black,
            levels_out_white: params.levels_out_white,
            bypass_color: if params.bypass_color { 1.0 } else { 0.0 },
            bypass_median: if params.bypass_median { 1.0 } else { 0.0 },

            bypass_fsr: if params.bypass_fsr { 1.0 } else { 0.0 },
            bypass_detail: if params.bypass_detail { 1.0 } else { 0.0 },
            bypass_levels: if params.bypass_levels { 1.0 } else { 0.0 },
            _pad0: 0.0,
        };
        self.queue
            .write_buffer(&self.filter_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.surface_config.width = new_size.0;
            self.surface_config.height = new_size.1;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn render_frame_with_overlay<F>(
        &mut self,
        texture_manager: &crate::cache::TextureManager,
        window_size: (u32, u32),
        nav: &crate::view::NavigationController,
        mut overlay_pass: F,
    ) -> Result<bool, wgpu::SurfaceError>
    where
        F: FnMut(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &wgpu::TextureView),
    {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut draw_calls = Vec::new();
        let mut needs_texture_poll = false;

        if let Some(layout) = &nav.layout
            && let Some(cp) = nav.current_page
        {
            let mut all_vertices = Vec::new();
            let mut all_indices: Vec<u16> = Vec::new();

            for placement in &layout.placements {
                if !Self::should_render_page(nav, window_size, cp, placement) {
                    continue;
                }

                if let Some(gpu_image) = texture_manager.get(placement.page_index) {
                    for (tile_idx, tile) in gpu_image.tiles.iter().enumerate() {
                        let sx = tile.rect.x as f32 / gpu_image.width as f32;
                        let sy = tile.rect.y as f32 / gpu_image.height as f32;
                        let sw = tile.rect.width as f32 / gpu_image.width as f32;
                        let sh = tile.rect.height as f32 / gpu_image.height as f32;

                        let mut tile_pos = placement.position;
                        tile_pos[0] += placement.size[0] * sx;
                        // Invert Y because image space is Y-Down but world space is Y-Up.
                        // Subtract sh because create_quad_data expects the bottom-left corner.
                        tile_pos[1] += placement.size[1] * (1.0 - sy - sh);
                        let mut tile_size = placement.size;
                        tile_size[0] *= sw;
                        tile_size[1] *= sh;

                        let rotation = if matches!(
                            nav.view.layout_mode,
                            crate::types::LayoutMode::Dual { .. }
                        ) {
                            crate::view::RotationQuarter::Deg0
                        } else {
                            nav.view.rotation
                        };
                        let page_center = [
                            placement.position[0] + placement.size[0] * 0.5,
                            placement.position[1] + placement.size[1] * 0.5,
                        ];

                        let (verts, idxs) = crate::renderer::quad_batch::create_quad_data(
                            tile_pos,
                            tile_size,
                            rotation,
                            page_center,
                        );

                        let index_start = all_indices.len() as u32;
                        let index_count = idxs.len() as u32;
                        let base_vertex = all_vertices.len() as u16;
                        let adjusted_idxs =
                            idxs.iter().map(|i| i + base_vertex).collect::<Vec<_>>();

                        all_vertices.extend(verts);
                        all_indices.extend(adjusted_idxs);
                        draw_calls.push((placement.page_index, tile_idx, index_start, index_count));
                    }
                } else {
                    needs_texture_poll = true;
                }
            }

            if !all_vertices.is_empty() {
                let vertex_data = bytemuck::cast_slice(&all_vertices);
                let index_data = bytemuck::cast_slice(&all_indices);

                let v_size = vertex_data.len() as u64;
                let i_size = index_data.len() as u64;

                // Dynamically grow vertex buffer if needed
                if v_size > self.vertex_buffer_size {
                    self.vertex_buffer_size = v_size.next_power_of_two();
                    self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Vertex Buffer (Expanded)"),
                        size: self.vertex_buffer_size,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                }

                // Dynamically grow index buffer if needed
                if i_size > self.index_buffer_size {
                    self.index_buffer_size = i_size.next_power_of_two();
                    self.index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Index Buffer (Expanded)"),
                        size: self.index_buffer_size,
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                }

                self.queue.write_buffer(&self.vertex_buffer, 0, vertex_data);
                self.queue.write_buffer(&self.index_buffer, 0, index_data);
            }
        }

        let uniform = CameraUniform {
            view_proj: nav
                .camera
                .build_view_projection(window_size)
                .to_cols_array_2d(),
        };
        if self.last_camera_uniform.as_ref() != Some(&uniform) {
            self.queue
                .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
            self.last_camera_uniform = Some(uniform);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !draw_calls.is_empty() {
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(2, &self.filter_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                for (page_id, tile_idx, index_start, index_count) in draw_calls {
                    if let Some(gpu_image) = texture_manager.get(page_id) {
                        let tile = &gpu_image.tiles[tile_idx];
                        render_pass.set_bind_group(1, &tile.bind_group, &[]);
                        render_pass.draw_indexed(index_start..index_start + index_count, 0, 0..1);
                    }
                }
            }
        }

        overlay_pass(&self.device, &self.queue, &mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(needs_texture_poll)
    }
}
