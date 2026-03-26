use crate::cache::TextureManager;
use crate::types::PageId;
use crate::view::animation_controller::AnimationController;

pub fn process_animations_controller(
    animation_controller: &mut AnimationController,
    visible_pages: &[PageId],
    texture_manager: &TextureManager,
    renderer: Option<&crate::renderer::Renderer>,
) -> bool {
    let frames_to_update = animation_controller.collect_due_frames(visible_pages);
    let has_active_animation = animation_controller.has_active_for(visible_pages);

    let mut gpu_updated = false;
    for frame in frames_to_update {
        if let Some(gpu_image) = texture_manager.get(frame.page_id)
            && let Some(renderer) = renderer
        {
            for tile in &gpu_image.tiles {
                renderer.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &tile.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &frame.pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: ((tile.rect.y * gpu_image.width + tile.rect.x) * 4) as u64,
                        bytes_per_row: Some(4 * gpu_image.width),
                        rows_per_image: Some(tile.rect.height),
                    },
                    wgpu::Extent3d {
                        width: tile.rect.width,
                        height: tile.rect.height,
                        depth_or_array_layers: 1,
                    },
                );
            }
            gpu_updated = true;
        }
    }

    if has_active_animation {
        gpu_updated = true;
    }

    gpu_updated
}
