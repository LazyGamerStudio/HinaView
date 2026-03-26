use crate::bookmark::{BookmarkEntry, BookmarkError, BookmarkService, BookmarkSource};
use crate::database::DatabaseService;
use crate::util::now_unix_ms;
use crate::view::NavigationController;
use tracing::error;

pub fn save_manual_bookmark(
    nav: &NavigationController,
    service: &mut BookmarkService,
    database: &DatabaseService,
) -> Result<(), BookmarkError> {
    let Some(entry) = current_bookmark_entry(nav, BookmarkSource::Manual) else {
        return Ok(());
    };

    service.add_manual(entry.clone())?;

    if let Err(e) = database.save_bookmark(&entry) {
        error!("[Bookmark] Failed to save manual bookmark to DB: {}", e);
    }

    Ok(())
}

pub fn save_auto_recent_bookmark(
    nav: &NavigationController,
    service: &mut BookmarkService,
    database: &DatabaseService,
) {
    let Some(entry) = current_bookmark_entry(nav, BookmarkSource::AutoRecent) else {
        return;
    };

    service.add_auto_recent(entry.clone());

    if let Err(e) = database.save_bookmark(&entry) {
        error!("[Bookmark] Failed to save auto bookmark to DB: {}", e);
    }
}

pub fn delete_bookmark(service: &mut BookmarkService, database: &DatabaseService, id: u64) {
    service.remove(id);

    if let Err(e) = database.delete_bookmark(id) {
        error!("[Bookmark] Failed to delete bookmark from DB: {}", e);
    }
}

pub fn get_bookmark_for_opening(service: &BookmarkService, id: u64) -> Option<BookmarkEntry> {
    service.find(id).cloned()
}

fn current_bookmark_entry(
    nav: &NavigationController,
    source: BookmarkSource,
) -> Option<BookmarkEntry> {
    let doc = nav.document.as_ref()?;
    let page_index = nav.current_page?;
    let page = doc.pages.get(page_index)?;

    let archive_name = doc
        .path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("-")
        .to_string();
    let page_name = page.name.clone();

    Some(BookmarkEntry {
        id: 0,
        source,
        archive_name,
        file_name: page_name.clone(),
        path: doc.path.clone(),
        page_index,
        page_name,
        saved_at_ms: now_unix_ms(),
    })
}
