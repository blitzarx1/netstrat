use chrono::{Date, DateTime, Utc};

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

    pub fn payload(&self) -> String {
        self.payload.clone()
    }

    pub fn ts(&self) -> DateTime<Utc> {
        self.ts
    }
}
