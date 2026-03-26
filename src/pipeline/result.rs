// src/pipeline/result.rs
use super::decoders::FrameStream;
use crate::types::*;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct DecodeResult {
    pub doc_id: u64,
    pub page_id: PageId,
    pub page_name: String,
    pub mip: MipLevel,
    pub image: DecodedImage,
    pub is_animated: bool,
    pub stream: Option<Arc<Mutex<Box<dyn FrameStream>>>>,
    #[allow(dead_code)]
    pub decoder_name: &'static str,
    #[allow(dead_code)]
    pub decode_ms: f32,
    #[allow(dead_code)]
    pub resample_ms: f32,
    #[allow(dead_code)]
    pub queue_wait_ms: f32,
    #[allow(dead_code)]
    pub read_ms: f32,
    #[allow(dead_code)]
    pub worker_total_ms: f32,
    /// Delay for the first frame of an animation (in milliseconds)
    /// This is used to initialize the animation timing
    pub first_frame_delay_ms: u64,
}
