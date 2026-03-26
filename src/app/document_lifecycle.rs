use crate::cache::TextureManager;
use crate::document::archive_navigator::ArchiveNavigator;
use crate::pipeline::DecodeScheduler;
use crate::types::PageId;
use crate::view::NavigationController;
use crate::view::animation_controller::AnimationController;
use anyhow::Result;
use std::path::PathBuf;

pub fn load_document_into_app(
    path: PathBuf,
    nav: &mut NavigationController,
    texture_manager: &mut TextureManager,
    scheduler: &mut DecodeScheduler,
    animation_controller: &mut AnimationController,
    archive_navigator: &mut ArchiveNavigator,
    window_size: (u32, u32),
) -> Result<()> {
    // 1. Open document and queue metadata (This sets up the document ID and pages)
    let (new_doc, initial_page): (crate::document::Document, PageId) =
        crate::document::Document::open_with_initial(path)?;

    tracing::info!(
        "[Lifecycle] Document successfully initialized: {} pages",
        new_doc.pages.len()
    );

    // 2. Set the document immediately to ensure MetadataSync doesn't discard results
    nav.document = Some(new_doc);
    nav.current_page = None;
    nav.target_page = Some(initial_page);
    nav.pending_page = None;
    nav.state = crate::view::NavState::Idle;
    nav.view.pan = [0.0, 0.0];
    nav.view.image_offset = [0.0, 0.0];
    nav.camera.pan = glam::Vec2::ZERO;
    if matches!(nav.view.layout_mode, crate::types::LayoutMode::Dual { .. }) {
        nav.view.layout_mode = crate::types::LayoutMode::Single;
    }

    // 3. Clear transient states
    texture_manager.clear_page_table();
    scheduler.clear_inflight(); // Clear stale decode/metadata jobs from previous archive.
    animation_controller.clear();
    archive_navigator.invalidate_cache();

    // 4. Update layout and start decoding the initial page
    nav.refresh_layout(window_size);
    nav.refresh_camera();
    nav.prefetch_after_first_present = true;
    nav.navigate(initial_page);

    Ok(())
}
