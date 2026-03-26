// src/pipeline/upload_queue.rs
use super::result::DecodeResult;
use crossbeam::channel::Receiver;

pub struct UploadQueue {
    rx: Receiver<DecodeResult>,
}

impl UploadQueue {
    pub fn new(rx: Receiver<DecodeResult>) -> Self {
        Self { rx }
    }

    pub fn try_recv(&self) -> Option<DecodeResult> {
        self.rx.try_recv().ok()
    }

    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }
}
