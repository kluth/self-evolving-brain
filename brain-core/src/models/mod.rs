use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub source_category: String,
    pub source_specifics: String,
    pub timestamp: DateTime<Utc>,
    pub reliability_score: f64,
    pub is_fake_news: bool,
    pub entities: Vec<String>,
    pub location: Option<String>,
    pub language: String,
    pub version: String,
    pub raw_metadata: String, // Serialized JSON
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_knowledge_node_serialization() {
        let node = KnowledgeNode {
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            summary: Some("Test Summary".to_string()),
            source_category: "Test Category".to_string(),
            source_specifics: "Test Source".to_string(),
            timestamp: Utc.with_ymd_and_hms(2024, 5, 20, 12, 0, 0).unwrap(),
            reliability_score: 0.95,
            is_fake_news: false,
            entities: vec!["Entity1".to_string(), "Entity2".to_string()],
            location: Some("Berlin, Germany".to_string()),
            language: "en".to_string(),
            version: "1.1".to_string(),
            raw_metadata: "{}".to_string(),
        };

        let serialized = serde_json::to_string(&node).unwrap();
        let deserialized: KnowledgeNode = serde_json::from_str(&serialized).unwrap();

        assert_eq!(node.title, deserialized.title);
        assert_eq!(node.location, deserialized.location);
        assert_eq!(node.language, deserialized.language);
        assert_eq!(node.version, deserialized.version);
    }
}
