// src/cache/texture_manager.rs
use crate::cache::GpuTextureCache;
use crate::cache::gpu_uploader::GpuUploadContext;
use crate::cache::texture_index::{TextureIndex, map_to_page};
use crate::pipeline::DecodeResult;
use crate::renderer::GpuImage;
use crate::types::{MipLevel, PageId};
use std::collections::HashMap;
use std::sync::Arc;

/// Reverse index: (doc_id, page_hash, mip) -> page_id
/// Used to sync TextureIndex with GpuCache evictions
type GpuCacheReverseIndex = HashMap<(u64, u64, MipLevel), PageId>;

/// Reference count for each page_id in GPU cache
/// Used to track when a page has no more GPU cache entries (O(1) lookup)
type GpuCacheRefCount = HashMap<PageId, usize>;

pub struct TextureManager {
    pub textures: TextureIndex,
    pub gpu_cache: GpuTextureCache,
    /// Reverse index to find page_id from GPU cache key
    pub gpu_cache_reverse_index: GpuCacheReverseIndex,
    /// Reference count per page_id for O(1) eviction check
    pub page_ref_count: GpuCacheRefCount,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: TextureIndex::new(),
            gpu_cache: GpuTextureCache::new(),
            gpu_cache_reverse_index: HashMap::new(),
            page_ref_count: HashMap::new(),
        }
    }

    pub fn set_gpu_cache(&mut self, cache: GpuTextureCache) {
        self.gpu_cache = cache;
    }

    pub fn set_gpu_cache_limit_mb(&mut self, mb: usize) {
        self.gpu_cache.set_max_memory_mb(mb);
    }

    pub fn gpu_cache_memory_mb(&self) -> usize {
        self.gpu_cache.memory_usage_mb()
    }

    pub fn gpu_cache_max_mb(&self) -> usize {
        self.gpu_cache.max_memory_mb()
    }

    pub fn clear_page_table(&mut self) {
        self.textures.clear();
        self.gpu_cache_reverse_index.clear();
        self.page_ref_count.clear();
    }

    #[allow(dead_code)]
    pub fn clear_gpu_cache(&mut self) {
        self.gpu_cache.clear();
        self.gpu_cache_reverse_index.clear();
        self.page_ref_count.clear();
    }

    /// Explicitly remove a page from both the index and the GPU cache.
    /// This ensures that VRAM resources are actually freed, not just hidden from the UI.
    #[allow(dead_code)]
    pub fn remove_page(&mut self, page_id: PageId) {
        // 1. Remove from TextureIndex (UI mapping)
        self.textures.remove(&page_id);

        // 2. Remove all related entries from GpuTextureCache (VRAM resources)
        // Find all keys in reverse index that belong to this page_id
        let keys_to_remove: Vec<(u64, u64, MipLevel)> = self
            .gpu_cache_reverse_index
            .iter()
            .filter(|&(_, &id)| id == page_id)
            .map(|(key, _)| *key)
            .collect();

        for key in keys_to_remove {
            self.gpu_cache.remove(key.0, key.1, key.2);
            self.gpu_cache_reverse_index.remove(&key);
        }

        self.page_ref_count.remove(&page_id);
    }

    /// Handle evicted entries from GPU cache - removes them from TextureIndex
    pub fn handle_evicted(
        &mut self,
        evicted_entries: Vec<crate::cache::gpu_cache::EvictedGpuEntry>,
    ) {
        for entry in evicted_entries {
            let cache_key = (entry.doc_id, entry.page_hash, entry.mip);
            if let Some(page_id) = self.gpu_cache_reverse_index.remove(&cache_key) {
                // Decrement reference count (O(1))
                let count = self.page_ref_count.entry(page_id).or_insert(0);
                *count = count.saturating_sub(1);

                // Only remove from TextureIndex if no more GPU cache entries for this page (O(1))
                if *count == 0 {
                    self.textures.remove(&page_id);
                    self.page_ref_count.remove(&page_id);
                }
            }
        }
    }

    pub fn get(&self, page_id: PageId) -> Option<&Arc<GpuImage>> {
        self.textures.get(&page_id)
    }

    pub fn has_optimal_mip(
        &self,
        page_id: PageId,
        requested_mip: crate::types::MipLevel,
        is_animated: bool,
    ) -> bool {
        if let Some(gpu_image) = self.get(page_id) {
            if is_animated {
                // Animated pages are always stored as Full mip.
                // If any mip exists for an animated page, it's the Full one.
                true
            } else {
                gpu_image.mip == requested_mip
            }
        } else {
            false
        }
    }

    pub fn has_page(&self, page_id: PageId) -> bool {
        self.textures.contains_key(&page_id)
    }

    pub fn upload_to_gpu(
        &mut self,
        result: DecodeResult,
        current_doc_id: u64,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) {
        let evicted = upload_image_to_gpu_internal(
            &mut self.textures,
            &mut self.gpu_cache,
            &mut self.gpu_cache_reverse_index,
            &mut self.page_ref_count,
            GpuUploadContext {
                doc_id: result.doc_id,
                _page_id: result.page_id,
                page_name: &result.page_name,
                mip: result.mip,
                image: &result.image,
                device,
                queue,
                texture_bind_group_layout,
                sampler,
            },
            current_doc_id,
        );
        self.handle_evicted(evicted);
    }

    /// Update the protection window in the GPU cache.
    pub fn update_protection(
        &mut self,
        doc_id: u64,
        protections: Vec<crate::cache::prefetch::SlidingWindowPriority>,
        pages: &[crate::document::PageMeta],
    ) {
        let mut gpu_protections = HashMap::new();

        for p in protections {
            if let Some(meta) = pages.get(p.page_id) {
                // Protect all major mip levels for the page
                let mips = [
                    MipLevel::Full,
                    MipLevel::Half,
                    MipLevel::Quarter,
                    MipLevel::Eighth,
                ];

                for mip in mips {
                    let hash = crate::cache::gpu_uploader::cache_key(doc_id, &meta.name, mip);
                    gpu_protections.insert(hash, p.priority);
                }
            }
        }

        self.gpu_cache.set_protection(gpu_protections);
    }
}

/// Independent function to avoid borrowing self while borrowing self.gpu_cache
pub fn upload_image_to_gpu_internal(
    textures: &mut TextureIndex,
    gpu_cache: &mut GpuTextureCache,
    reverse_index: &mut crate::cache::texture_manager::GpuCacheReverseIndex,
    page_ref_count: &mut crate::cache::texture_manager::GpuCacheRefCount,
    ctx: GpuUploadContext,
    current_doc_id: u64,
) -> Vec<crate::cache::gpu_cache::EvictedGpuEntry> {
    let doc_id = ctx.doc_id;
    let page_id = ctx._page_id;
    let page_name = ctx.page_name.to_string();
    let mip = ctx.mip;

    // Compute cache key
    let page_hash = crate::cache::gpu_uploader::cache_key(doc_id, &page_name, mip);
    let cache_key = (doc_id, page_hash, mip);
    let replacing_existing = reverse_index.get(&cache_key).copied() == Some(page_id);

    // Upload to GPU and insert into cache, collecting evicted entries
    let (gpu_image, mut evicted) = upload_to_gpu_cache(gpu_cache, ctx, page_hash);

    if replacing_existing {
        evicted.retain(|entry| (entry.doc_id, entry.page_hash, entry.mip) != cache_key);
    }

    // Update reverse index before mapping
    reverse_index.insert(cache_key, page_id);

    // Only count a new reference when this key did not already belong to the same page.
    if !replacing_existing {
        *page_ref_count.entry(page_id).or_insert(0) += 1;
    }

    map_to_page(textures, current_doc_id, doc_id, page_id, gpu_image);
    tracing::debug!(
        "[GPU Map] page={} ({}) mip={:?} | doc={} current_doc={}",
        page_id,
        page_name,
        mip,
        doc_id,
        current_doc_id
    );

    evicted
}

/// Upload image to GPU and insert into cache, returning evicted entries
fn upload_to_gpu_cache(
    gpu_cache: &mut GpuTextureCache,
    ctx: GpuUploadContext,
    page_hash: u64,
) -> (Arc<GpuImage>, Vec<crate::cache::gpu_cache::EvictedGpuEntry>) {
    // Try to get from cache first
    if let Some(gpu_image) = gpu_cache.get(ctx.doc_id, page_hash, ctx.mip) {
        tracing::debug!(
            "[GPU Reuse] {} | {:?}: reused cached GPU texture",
            ctx.page_name,
            ctx.mip
        );
        return (gpu_image, Vec::new());
    }

    let start = std::time::Instant::now();

    let max_texture_size = ctx.device.limits().max_texture_dimension_2d;
    let tile_size = max_texture_size.min(4096);

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

    // Insert into cache and get evicted entries
    let evicted = gpu_cache.insert(ctx.doc_id, page_hash, ctx.mip, gpu_image.clone());

    (gpu_image, evicted)
}
