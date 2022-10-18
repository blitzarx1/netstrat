use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Message {
    payload: String,
    ts: DateTime<Utc>,
}
