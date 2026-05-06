use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::fs;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    pub url: String,
    pub category: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub failure_count: u32,
    #[serde(default)]
    pub last_error: Option<String>,
}

fn default_enabled() -> bool { true }

pub struct SourceManager {
    path: String,
    sources: Arc<RwLock<Vec<Source>>>,
}

impl SourceManager {
    pub fn new(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let raw_sources: Vec<serde_json::Value> = serde_json::from_str(&content)?;
        
        let sources: Vec<Source> = raw_sources.into_iter().map(|s| {
            Source {
                url: s["url"].as_str().unwrap_or_default().to_string(),
                category: s["category"].as_str().unwrap_or("Global_News").to_string(),
                enabled: s["enabled"].as_bool().unwrap_or(true),
                failure_count: s["failure_count"].as_u64().unwrap_or(0) as u32,
                last_error: s["last_error"].as_str().map(|e| e.to_string()),
            }
        }).collect();

        Ok(Self {
            path: path.to_string(),
            sources: Arc::new(RwLock::new(sources)),
        })
    }

    pub fn get_active_sources(&self) -> Vec<Source> {
        self.sources.read().unwrap().iter().filter(|s| s.enabled).cloned().collect()
    }

    pub fn report_failure(&self, url: &str, error: &str) {
        let mut sources = self.sources.write().unwrap();
        if let Some(source) = sources.iter_mut().find(|s| s.url == url) {
            source.failure_count += 1;
            source.last_error = Some(error.to_string());
            if source.failure_count >= 5 {
                source.enabled = false;
                println!("[SourceManager] Disabling unhealthy source: {}", url);
            }
        }
    }

    pub fn report_success(&self, url: &str) {
        let mut sources = self.sources.write().unwrap();
        if let Some(source) = sources.iter_mut().find(|s| s.url == url) {
            if source.failure_count > 0 {
                source.failure_count = 0;
                source.last_error = None;
            }
        }
    }

    pub fn add_source(&self, url: String, category: String) {
        let mut sources = self.sources.write().unwrap();
        if !sources.iter().any(|s| s.url == url) {
            sources.push(Source {
                url,
                category,
                enabled: true,
                failure_count: 0,
                last_error: None,
            });
        }
    }

    pub fn delete_source(&self, url: &str) {
        let mut sources = self.sources.write().unwrap();
        sources.retain(|s| s.url != url);
    }

    pub fn toggle_source(&self, url: &str, enabled: bool) {
        let mut sources = self.sources.write().unwrap();
        if let Some(source) = sources.iter_mut().find(|s| s.url == url) {
            source.enabled = enabled;
            if enabled {
                source.failure_count = 0;
                source.last_error = None;
            }
        }
    }

    pub fn get_all_sources(&self) -> Vec<Source> {
        self.sources.read().unwrap().clone()
    }

    pub fn save(&self) -> Result<()> {
        let sources = self.sources.read().unwrap();
        let content = serde_json::to_string_pretty(&*sources)?;
        fs::write(&self.path, content)?;
        Ok(())
    }
}
