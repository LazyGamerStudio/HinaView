use super::job::DecodeJob;
use super::result::DecodeResult;
use ::tracing::error;

pub fn execute_decode_job(worker_id: usize, job: DecodeJob) -> Option<DecodeResult> {
    let worker_start = std::time::Instant::now();
    let queue_wait_ms = worker_start
        .saturating_duration_since(job.enqueued_at)
        .as_secs_f32()
        * 1000.0;

    let read_start = std::time::Instant::now();
    let raw_data: Vec<u8> = match job.reader.read_file(&job.page_name) {
        Ok(data) => data,
        Err(e) => {
            error!("[Worker {}] └─ Read ERROR: {}", worker_id, e);
            return None;
        }
    };
    let read_ms = read_start.elapsed().as_secs_f32() * 1000.0;

    let mut is_animated = false;
    let mut stream = None;
    let decoder_name;
    let mut first_frame_delay_ms = 100u64;

    let decode_start = std::time::Instant::now();

    // Determine if this job should be treated as "Heavy" (Singly executed to maximize resource usage).
    // Always treat the CURRENT page as heavy to guarantee maximum responsiveness.
    let is_heavy = super::decode::is_heavy_format(&raw_data) || job.priority == 0;

    let decoded_result = {
        let _guard = if is_heavy {
            Some(super::decode::HEAVY_FORMAT_LOCK.lock())
        } else {
            None
        };

        let decoded = if super::decoders::webp_ffi::is_animated(&raw_data) {
            match super::decoders::webp_ffi::create_stream(raw_data) {
                Ok(mut s) => {
                    is_animated = true;
                    decoder_name = "WebP-FFI (Animated)";
                    if let Some(first_frame) = s.next_frame() {
                        let (w, h) = s.dimensions();
                        first_frame_delay_ms = first_frame.delay.as_millis() as u64;
                        stream = Some(std::sync::Arc::new(parking_lot::Mutex::new(s)));
                        crate::types::DecodedImage {
                            width: w,
                            height: h,
                            original_width: w,
                            original_height: h,
                            pixels: first_frame.pixels,
                            icc_profile: None,
                            exif: None,
                        }
                    } else {
                        error!("[Worker {}]   └─ Anim Stream ERROR: No frames", worker_id);
                        return None;
                    }
                }
                Err(e) => {
                    error!("[Worker {}]   └─ Anim Stream ERROR: {:?}", worker_id, e);
                    return None;
                }
            }
        } else if super::decoders::gif::is_gif(&raw_data) {
            if let Some((w, h, frames)) = super::decoders::gif::get_info(&raw_data) {
                if frames > 1 {
                    match super::decoders::gif::create_stream(raw_data) {
                        Ok(mut s) => {
                            is_animated = true;
                            decoder_name = "GIF (Animated)";
                            if let Some(first_frame) = s.next_frame() {
                                stream = Some(std::sync::Arc::new(parking_lot::Mutex::new(s)));
                                first_frame_delay_ms = first_frame.delay.as_millis() as u64;
                                crate::types::DecodedImage {
                                    width: w,
                                    height: h,
                                    original_width: w,
                                    original_height: h,
                                    pixels: first_frame.pixels,
                                    icc_profile: None,
                                    exif: None,
                                }
                            } else {
                                error!(
                                    "[Worker {}]   └─ GIF Anim Stream ERROR: No frames",
                                    worker_id
                                );
                                return None;
                            }
                        }
                        Err(e) => {
                            error!("[Worker {}]   └─ GIF Anim Stream ERROR: {:?}", worker_id, e);
                            return None;
                        }
                    }
                } else {
                    let (res, name) = super::decode::decode_bytes(&raw_data, job.mip);
                    decoder_name = name;
                    match res {
                        Ok(img) => img,
                        Err(e) => {
                            error!("[Worker {}]   └─ Decode ERROR ({}): {}", worker_id, name, e);
                            return None;
                        }
                    }
                }
            } else {
                let (res, name) = super::decode::decode_bytes(&raw_data, job.mip);
                decoder_name = name;
                match res {
                    Ok(img) => img,
                    Err(e) => {
                        error!("[Worker {}]   └─ Decode ERROR ({}): {}", worker_id, name, e);
                        return None;
                    }
                }
            }
        } else {
            let (res, name) = super::decode::decode_bytes(&raw_data, job.mip);
            decoder_name = name;
            match res {
                Ok(img) => img,
                Err(e) => {
                    error!("[Worker {}]   └─ Decode ERROR ({}): {}", worker_id, name, e);
                    return None;
                }
            }
        };

        let resample_start_inner = std::time::Instant::now();
        tracing::debug!(
            "[Worker {}][Resample][{}] EXECUTE: {} | mip={:?}, skip_resample={}, input={}x{}",
            worker_id,
            job.reason,
            job.page_name,
            job.mip,
            job.skip_resample,
            decoded.width,
            decoded.height
        );

        let final_image = if is_animated || job.skip_resample {
            decoded
        } else {
            super::resample::apply_mip(decoded, job.mip)
        };

        (final_image, resample_start_inner.elapsed())
    };

    let (final_image, resample_duration) = decoded_result;
    let decode_ms = decode_start.elapsed().as_secs_f32() * 1000.0;
    let resample_ms = resample_duration.as_secs_f32() * 1000.0;

    if !is_animated && !job.skip_resample {
        super::log_resample_info(
            worker_id,
            &job.page_name,
            job.mip,
            final_image.width,
            final_image.height,
            resample_ms,
        );
    }
    let worker_total_ms = worker_start.elapsed().as_secs_f32() * 1000.0;
    super::log_decode_info(super::DecodeLogInfo {
        worker_id,
        name: &job.page_name,
        decoder: decoder_name,
        orig_w: final_image.original_width,
        orig_h: final_image.original_height,
        dec_w: final_image.width,
        dec_h: final_image.height,
        queue_wait_ms,
        read_ms,
        decode_ms,
        resample_ms,
        worker_total_ms,
    });
    Some(DecodeResult {
        doc_id: job.doc_id,
        page_id: job.page_id,
        page_name: job.page_name,
        mip: job.mip,
        image: final_image,
        is_animated,
        stream,
        decoder_name,
        decode_ms,
        resample_ms,
        queue_wait_ms,
        read_ms,
        worker_total_ms,
        first_frame_delay_ms,
    })
}
