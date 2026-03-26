use std::time::{Duration, Instant};

const DURATION_DEFAULT: Duration = Duration::from_secs(3);
const DURATION_SHORT: Duration = Duration::from_millis(200);

struct ToastMessage {
    text: String,
    expires_at: Instant,
}

pub struct ToastOverlay {
    current: Option<ToastMessage>,
    duration: Duration,
    dirty: bool,
}

impl ToastOverlay {
    pub fn new() -> Self {
        Self {
            current: None,
            duration: DURATION_DEFAULT,
            dirty: false,
        }
    }

    pub fn show(&mut self, message: impl Into<String>, flag: u32) {
        let duration = if flag == 0 {
            DURATION_SHORT
        } else {
            self.duration
        };
        let text = message.into();

        // Only set dirty if the text actually changed or a new toast is shown
        if self.current.as_ref().map(|m| &m.text) != Some(&text) {
            self.dirty = true;
        }

        self.current = Some(ToastMessage {
            text,
            expires_at: Instant::now() + duration,
        });
    }

    pub fn current_text(&self) -> Option<&str> {
        self.current.as_ref().map(|m| m.text.as_str())
    }

    pub fn update(&mut self) {
        if let Some(current) = &self.current
            && Instant::now() >= current.expires_at
        {
            self.current = None;
            self.dirty = true; // Mark dirty when it disappears
        }
    }

    pub fn is_visible(&self) -> bool {
        self.current.is_some()
    }

    pub fn take_dirty(&mut self) -> bool {
        let d = self.dirty;
        self.dirty = false;
        d
    }
}
