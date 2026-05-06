pub mod producer;
pub mod consumer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueItem {
    pub title: String,
    pub content: String,
    pub source_category: String,
    pub source_specifics: String,
    pub timestamp: String, // ISO 8601
    pub raw_metadata: String,
}
