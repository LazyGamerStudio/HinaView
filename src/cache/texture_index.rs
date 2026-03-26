use crate::renderer::GpuImage;
use crate::types::PageId;
use std::collections::HashMap;
use std::sync::Arc;

pub type TextureIndex = HashMap<PageId, Arc<GpuImage>>;

pub fn map_to_page(
    index: &mut TextureIndex,
    current_doc_id: u64,
    result_doc_id: u64,
    page_id: PageId,
    image: Arc<GpuImage>,
) {
    if result_doc_id == current_doc_id {
        index.insert(page_id, image);
    }
}
