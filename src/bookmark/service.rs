use super::model::{BookmarkEntry, BookmarkSource};

pub const MAX_TOTAL: usize = 20;
pub const MAX_AUTO_RECENT: usize = 5;
pub const MAX_MANUAL: usize = 15;

#[derive(Debug)]
pub enum BookmarkError {
    ManualLimitExceeded,
}

#[derive(Default)]
pub struct BookmarkService {
    entries: Vec<BookmarkEntry>,
    next_id: u64,
}

impl BookmarkService {
    pub fn from_entries(entries: Vec<BookmarkEntry>) -> Self {
        let next_id = entries.iter().map(|e| e.id).max().unwrap_or(0) + 1;
        let mut service = Self { entries, next_id };
        service.sort_recent_first();
        service
    }

    pub fn entries(&self) -> &[BookmarkEntry] {
        &self.entries
    }

    pub fn add_manual(&mut self, mut entry: BookmarkEntry) -> Result<(), BookmarkError> {
        let manual_count = self
            .entries
            .iter()
            .filter(|e| e.source == BookmarkSource::Manual)
            .count();
        if manual_count >= MAX_MANUAL {
            return Err(BookmarkError::ManualLimitExceeded);
        }

        entry.id = self.issue_id();
        entry.source = BookmarkSource::Manual;
        self.upsert_recent(entry);
        self.enforce_limits();
        Ok(())
    }

    pub fn add_auto_recent(&mut self, mut entry: BookmarkEntry) {
        entry.id = self.issue_id();
        entry.source = BookmarkSource::AutoRecent;
        self.upsert_recent(entry);
        self.enforce_limits();
    }

    pub fn remove(&mut self, id: u64) {
        self.entries.retain(|e| e.id != id);
    }

    pub fn find(&self, id: u64) -> Option<&BookmarkEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    fn upsert_recent(&mut self, entry: BookmarkEntry) {
        self.entries.retain(|e| {
            !(e.source == entry.source && e.path == entry.path && e.page_index == entry.page_index)
        });
        self.entries.push(entry);
        self.sort_recent_first();
    }

    fn sort_recent_first(&mut self) {
        self.entries
            .sort_by_key(|e| std::cmp::Reverse(e.saved_at_ms));
    }

    fn enforce_limits(&mut self) {
        self.sort_recent_first();

        let mut auto = Vec::new();
        let mut manual = Vec::new();
        for e in self.entries.drain(..) {
            match e.source {
                BookmarkSource::AutoRecent => auto.push(e),
                BookmarkSource::Manual => manual.push(e),
            }
        }

        auto.truncate(MAX_AUTO_RECENT);
        manual.truncate(MAX_MANUAL);

        let mut merged = Vec::new();
        merged.extend(auto);
        merged.extend(manual);
        merged.sort_by_key(|e| std::cmp::Reverse(e.saved_at_ms));
        merged.truncate(MAX_TOTAL);

        self.entries = merged;
    }

    fn issue_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
