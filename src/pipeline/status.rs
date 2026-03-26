use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;
use crate::types::MipLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DecodeStatus {
    Pending,
    Reading,
    Decoding,
    Resampling,
    Uploading,
}

impl DecodeStatus {
    pub fn label(&self) -> &'static str {
        match self {
            DecodeStatus::Pending => "Pending...",
            DecodeStatus::Reading => "Reading file...",
            DecodeStatus::Decoding => "Decoding image...",
            DecodeStatus::Resampling => "Resampling...",
            DecodeStatus::Uploading => "Uploading to GPU...",
        }
    }

    pub fn progress(&self) -> f32 {
        match self {
            DecodeStatus::Pending => 0.05,
            DecodeStatus::Reading => 0.1,
            DecodeStatus::Decoding => 0.4,
            DecodeStatus::Resampling => 0.8,
            DecodeStatus::Uploading => 0.95,
        }
    }
}

pub struct PipelineStatus {
    // Maps (doc_id, page_name, mip) to status
    current_statuses: Arc<Mutex<HashMap<(u64, String, MipLevel), DecodeStatus>>>,
}

impl PipelineStatus {
    pub fn new() -> Self {
        Self {
            current_statuses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn update(&self, doc_id: u64, page_name: String, mip: MipLevel, status: DecodeStatus) {
        self.current_statuses.lock().insert((doc_id, page_name, mip), status);
    }

    pub fn remove(&self, doc_id: u64, page_name: &str, mip: MipLevel) {
        self.current_statuses.lock().remove(&(doc_id, page_name.to_string(), mip));
    }

    /// Gets the most "advanced" status for any mip of the given page.
    pub fn get_latest_for_page(&self, page_name: &str) -> Option<DecodeStatus> {
        let lock = self.current_statuses.lock();
        lock.iter()
            .filter(|((_, p, _), _)| p == page_name)
            .map(|(_, status)| *status)
            .max() 
    }

    pub fn clear(&self) {
        self.current_statuses.lock().clear();
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_STATUS: PipelineStatus = PipelineStatus::new();
}
