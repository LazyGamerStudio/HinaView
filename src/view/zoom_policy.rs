const ZOOM_PERCENT_STEP: f32 = 5.0;
const MIN_ZOOM_PERCENT: f32 = 5.0;
const MAX_ZOOM_PERCENT: f32 = 800.0;

fn clamp_percent(percent: f32) -> f32 {
    percent.clamp(MIN_ZOOM_PERCENT, MAX_ZOOM_PERCENT)
}

fn is_multiple_of_step(percent: f32) -> bool {
    let rem = percent % ZOOM_PERCENT_STEP;
    rem.abs() < 0.001 || (ZOOM_PERCENT_STEP - rem).abs() < 0.001
}

pub fn zoom_in_step(current_zoom: f32) -> f32 {
    let current_percent = clamp_percent(current_zoom * 100.0);
    let next_percent = if is_multiple_of_step(current_percent) {
        current_percent + ZOOM_PERCENT_STEP
    } else {
        (current_percent / ZOOM_PERCENT_STEP).ceil() * ZOOM_PERCENT_STEP
    };

    clamp_percent(next_percent) / 100.0
}

pub fn zoom_out_step(current_zoom: f32) -> f32 {
    let current_percent = clamp_percent(current_zoom * 100.0);
    let next_percent = if is_multiple_of_step(current_percent) {
        current_percent - ZOOM_PERCENT_STEP
    } else {
        (current_percent / ZOOM_PERCENT_STEP).floor() * ZOOM_PERCENT_STEP
    };

    clamp_percent(next_percent) / 100.0
}

#[cfg(test)]
mod tests {
    use super::{zoom_in_step, zoom_out_step};

    #[test]
    fn zoom_in_rounds_up_to_next_5_percent() {
        assert!((zoom_in_step(0.125) - 0.15).abs() < 0.0001);
        assert!((zoom_in_step(0.20) - 0.25).abs() < 0.0001);
    }

    #[test]
    fn zoom_out_rounds_down_to_prev_5_percent() {
        assert!((zoom_out_step(0.925) - 0.90).abs() < 0.0001);
        assert!((zoom_out_step(0.90) - 0.85).abs() < 0.0001);
    }
}
