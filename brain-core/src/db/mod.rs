use reqwest::Client;
use serde_json::json;
use anyhow::Result;
use crate::models::KnowledgeNode;

pub struct WeaviateClient {
    client: Client,
    endpoint: String,
}

impl WeaviateClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.trim_end_matches('/').to_string(),
        }
    }

    pub async fn init_schema(&self) -> Result<()> {
        println!("Checking Weaviate schema at {}...", self.endpoint);
        let schema = json!({
            "class": "KnowledgeNode",
            "description": "A unified node of knowledge from personal, local, or global sources.",
            "vectorizer": "none",
            "properties": [
                { "name": "title", "dataType": ["text"], "indexSearchable": true, "tokenization": "word" },
                { "name": "content", "dataType": ["text"], "indexSearchable": true, "tokenization": "word" },
                { "name": "summary", "dataType": ["text"] },
                { "name": "source_category", "dataType": ["text"] },
                { "name": "source_specifics", "dataType": ["text"] },
                { "name": "timestamp", "dataType": ["date"] },
                { "name": "reliability_score", "dataType": ["number"] },
                { "name": "is_fake_news", "dataType": ["boolean"] },
                { "name": "entities", "dataType": ["text[]"] },
                { "name": "location", "dataType": ["text"] },
                { "name": "language", "dataType": ["text"] },
                { "name": "version", "dataType": ["text"] },
                { "name": "raw_metadata", "dataType": ["text"] }
            ]
        });

        let url = format!("{}/v1/schema", self.endpoint);
        let check_url = format!("{}/KnowledgeNode", url);
        let resp = self.client.get(&check_url).send().await?;

        if resp.status().is_success() {
            println!("Weaviate schema 'KnowledgeNode' is ready.");
            return Ok(());
        }

        println!("Initializing new Weaviate schema 'KnowledgeNode'...");
        let resp = self.client.post(&url).json(&schema).send().await?;
        if resp.status().is_success() {
            println!("Weaviate schema initialized successfully.");
            Ok(())
        } else {
            let error_text = resp.text().await?;
            Err(anyhow::anyhow!("Failed to initialize schema: {}", error_text))
        }
    }

    pub async fn insert_node(&self, node: &KnowledgeNode) -> Result<()> {
        println!("[DB] Inserting node: title={}, version={}", node.title, node.version);

        let url = format!("{}/v1/objects", self.endpoint);
        let body = json!({
            "class": "KnowledgeNode",
            "properties": {
                "title": node.title,
                "content": node.content,
                "summary": node.summary,
                "source_category": node.source_category,
                "source_specifics": node.source_specifics,
                "timestamp": node.timestamp.to_rfc3339(),
                "reliability_score": node.reliability_score,
                "is_fake_news": node.is_fake_news,
                "entities": node.entities,
                "location": node.location,
                "language": node.language,
                "version": node.version,
                "raw_metadata": node.raw_metadata
            }
        });

        let resp = self.client.post(&url).json(&body).send().await?;
        if resp.status().is_success() {
            #[cfg(debug_assertions)]
            println!("Successfully inserted: {}", node.title);
            Ok(())
        } else {
            let error_text = resp.text().await?;
            eprintln!("Failed to insert node '{}': {}", node.title, error_text);
            Err(anyhow::anyhow!("Failed to insert node: {}", error_text))
        }
    }

    pub async fn delete_nodes(&self, ids: Vec<String>) -> Result<()> {
        for id in ids {
            let url = format!("{}/v1/objects/KnowledgeNode/{}", self.endpoint, id);
            let resp = self.client.delete(&url).send().await?;
            if !resp.status().is_success() {
                let error_text = resp.text().await?;
                eprintln!("Failed to delete node '{}': {}", id, error_text);
            }
        }
        Ok(())
    }

    pub async fn query_similar_nodes(&self, _title: &str) -> Result<Vec<(String, KnowledgeNode)>> {
        // Placeholder for GraphQL 'nearText' query
        Ok(vec![])
    }

    pub async fn get_recent_nodes(&self, limit: usize) -> Result<Vec<KnowledgeNode>> {
        let url = format!("{}/v1/objects?class=KnowledgeNode&limit={}", self.endpoint, limit);
        let resp = self.client.get(&url).send().await?;
        
        if resp.status().is_success() {
            let body: serde_json::Value = resp.json().await?;
            let objects = body["objects"].as_array().ok_or_else(|| anyhow::anyhow!("Invalid response"))?;
            
            let mut nodes = Vec::new();
            for obj in objects {
                let props = &obj["properties"];
                let node = KnowledgeNode {
                    title: props["title"].as_str().unwrap_or_default().to_string(),
                    content: props["content"].as_str().unwrap_or_default().to_string(),
                    summary: props["summary"].as_str().map(|s| s.to_string()),
                    source_category: props["source_category"].as_str().unwrap_or_default().to_string(),
                    source_specifics: props["source_specifics"].as_str().unwrap_or_default().to_string(),
                    timestamp: chrono::DateTime::parse_from_rfc3339(props["timestamp"].as_str().unwrap_or_default())
                        .unwrap_or_else(|_| chrono::Utc::now().into()).with_timezone(&chrono::Utc),
                    reliability_score: props["reliability_score"].as_f64().unwrap_or(0.0),
                    is_fake_news: props["is_fake_news"].as_bool().unwrap_or(false),
                    entities: props["entities"].as_array().unwrap_or(&vec![]).iter().map(|v| v.as_str().unwrap_or_default().to_string()).collect(),
                    location: props["location"].as_str().map(|s| s.to_string()),
                    language: props["language"].as_str().unwrap_or("en").to_string(),
                    version: props["version"].as_str().unwrap_or("MISSING").to_string(),
                    raw_metadata: props["raw_metadata"].as_str().unwrap_or_default().to_string(),
                };
                nodes.push(node);
            }
            Ok(nodes)
        } else {
            let error_text = resp.text().await?;
            Err(anyhow::anyhow!("Failed to fetch nodes: {}", error_text))
        }
    }
}
