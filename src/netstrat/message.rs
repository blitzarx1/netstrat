use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Message {
    payload: String,
    ts: DateTime<Utc>,
}

impl Message {
    pub fn new(payload: String) -> Self {
        Self {
            payload,
            ts: Utc::now(),
        }
    }
}
