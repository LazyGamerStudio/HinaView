use std::time::{Duration, Instant};

struct WarningMessage {
    text: String,
    expires_at: Instant,
}

pub struct WarningOverlay {
    messages: Vec<WarningMessage>,
    duration: Duration,
    dirty: bool,
}

impl WarningOverlay {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            duration: Duration::from_secs(5), // Longer exposure as per design
            dirty: false,
        }
    }

    pub fn show(&mut self, message: impl Into<String>) {
        self.messages.push(WarningMessage {
            text: message.into(),
            expires_at: Instant::now() + self.duration,
        });
        self.dirty = true;
    }

    /// Returns a combined string of up to 5 active warning messages.
    // ... (current_text remains unchanged)
    pub fn current_text(&self) -> Option<String> {
        if self.messages.is_empty() {
            return None;
        }

        let texts: Vec<String> = self
            .messages
            .iter()
            .take(5)
            .map(|m| m.text.clone())
            .collect();

        Some(texts.join("\n"))
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let prev_len = self.messages.len();
        self.messages.retain(|m| now < m.expires_at);
        if self.messages.len() != prev_len {
            self.dirty = true;
        }
    }

    pub fn is_visible(&self) -> bool {
        !self.messages.is_empty()
    }

    pub fn take_dirty(&mut self) -> bool {
        let d = self.dirty;
        self.dirty = false;
        d
    }
}
