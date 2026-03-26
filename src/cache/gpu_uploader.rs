use crate::cache::GpuTextureCache;
use crate::renderer::GpuImage;
use crate::types::{MipLevel, PageId};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub fn cache_key(doc_id: u64, page_name: &str, mip: MipLevel) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    doc_id.hash(&mut hasher);
    page_name.hash(&mut hasher);
    (mip as u8).hash(&mut hasher);
    hasher.finish()
}

pub struct GpuUploadContext<'a> {
    pub doc_id: u64,
    pub _page_id: PageId,
    pub page_name: &'a str,
    pub mip: MipLevel,
    pub image: &'a crate::types::DecodedImage,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub sampler: &'a wgpu::Sampler,
}

#[allow(dead_code)]
pub fn upload_image_or_get_cached(
    gpu_cache: &mut GpuTextureCache,
    ctx: GpuUploadContext,
) -> Arc<GpuImage> {
    let hash = cache_key(ctx.doc_id, ctx.page_name, ctx.mip);

    if let Some(gpu_image) = gpu_cache.get(ctx.doc_id, hash, ctx.mip) {
        tracing::debug!(
            "[GPU Reuse] {} | {:?}: reused cached GPU texture",
            ctx.page_name,
            ctx.mip
        );
        return gpu_image;
    }

    let start = std::time::Instant::now();

    let max_texture_size = ctx.device.limits().max_texture_dimension_2d;
    // Always use 4096 max tile size for conservative VRAM fragmentation and universal support (or limits max if it's smaller)
    let tile_size = max_texture_size.min(4096);

    // If downscaled mip level fits in a smaller size, it will automatically yield a single tile
    let tile_rects =
        crate::util::tiling::compute_tiles(ctx.image.width, ctx.image.height, tile_size);
    let mut tiles = Vec::with_capacity(tile_rects.len());

    for rect in tile_rects {
        let texture_size = wgpu::Extent3d {
            width: rect.width,
            height: rect.height,
            depth_or_array_layers: 1,
        };

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!(
                "PageTexture_{}_{}_{}",
                ctx.page_name, rect.x, rect.y
            )),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        ctx.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &ctx.image.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: ((rect.y * ctx.image.width + rect.x) * 4) as u64,
                bytes_per_row: Some(4 * ctx.image.width),
                rows_per_image: Some(rect.height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: ctx.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(ctx.sampler),
                },
            ],
            label: Some(&format!(
                "BindGroup_{}_{}_{}",
                ctx.page_name, rect.x, rect.y
            )),
        });

        tiles.push(crate::renderer::GpuTile {
            texture,
            bind_group,
            rect,
        });
    }

    let duration = start.elapsed();
    super::log_gpu_upload_info(
        ctx.page_name,
        ctx.mip,
        ctx.image.width,
        ctx.image.height,
        duration.as_secs_f32() * 1000.0,
    );

    let gpu_image = Arc::new(GpuImage {
        tiles,
        width: ctx.image.width,
        height: ctx.image.height,
        mip: ctx.mip,
    });

    gpu_cache.insert(ctx.doc_id, hash, ctx.mip, gpu_image.clone());
    gpu_image
}
