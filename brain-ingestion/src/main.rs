mod fact_checker;
mod sources;
mod queue;
mod consolidation;
mod intelligence;

use brain_core::db::WeaviateClient;
use brain_core::models::KnowledgeNode;
use brain_cache::HybridCache;
use fact_checker::FactChecker;
use intelligence::IntelligenceEngine;
use queue::producer::QueueProducer;
use queue::consumer::QueueConsumer;
use queue::QueueItem;
use sources::news::RssFetcher;
use sources::google::GoogleAdapter;
use consolidation::MemoryCompactor;
use brain_core::source_manager::SourceManager;
use anyhow::Result;
use std::sync::Arc;
use chrono::{DateTime, Utc};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let weaviate_url = std::env::var("WEAVIATE_URL").unwrap_or_else(|_| "http://weaviate:8080".to_string());
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://redis:6379/".to_string());
    let stream_name = "knowledge_ingestion";

    let weaviate = Arc::new(WeaviateClient::new(&weaviate_url));
    let cache = Arc::new(HybridCache::new(&redis_url)?);
    let fact_checker = Arc::new(FactChecker::new());
    let producer = Arc::new(QueueProducer::new(&redis_url)?);
    let consumer = Arc::new(QueueConsumer::new(&redis_url)?);
    let compactor = Arc::new(MemoryCompactor::new(weaviate.clone()));
    let source_manager = Arc::new(SourceManager::new("sources.json")?);
    let intel_engine = Arc::new(IntelligenceEngine::new());

    println!("==========================================");
    println!("   Self-Evolving Brain Ingestion Worker   ");
    println!("==========================================");
    println!("Weaviate: {}", weaviate_url);
    println!("Redis:    {}", redis_url);

    // Task 0: Google Integration (Optional)
    let google_secret_path = std::env::var("GOOGLE_SECRET_PATH").unwrap_or_else(|_| "client_secret.json".to_string());
    let google_adapter = Arc::new(GoogleAdapter::new(&google_secret_path).await?);
    
    // Initialize Schema
    weaviate.init_schema().await?;

    // Task 1: RSS Producer
    let prod_clone = producer.clone();
    let sm_clone = source_manager.clone();
    tokio::spawn(async move {
        let fetcher = Arc::new(RssFetcher::new());
        
        loop {
            let active_sources = sm_clone.get_active_sources();
            println!("[Producer] Processing {} active feeds in parallel...", active_sources.len());
            
            use futures::StreamExt;
            let feeds_stream = futures::stream::iter(active_sources);
            
            feeds_stream.for_each_concurrent(10, |source| {
                let fetcher = fetcher.clone();
                let prod = prod_clone.clone();
                let sm = sm_clone.clone();
                let stream_name = stream_name.to_string();
                
                async move {
                    match fetcher.fetch_feed(&source.url).await {
                        Ok(entries) => {
                            sm.report_success(&source.url);
                            for entry in entries {
                                let item = QueueItem {
                                    title: entry.title.map(|t| t.content).unwrap_or_default(),
                                    content: entry.summary.map(|s| s.content).unwrap_or_default(),
                                    source_category: source.category.clone(),
                                    source_specifics: source.url.clone(),
                                    timestamp: entry.published.map(|p| p.to_rfc3339()).unwrap_or_else(|| Utc::now().to_rfc3339()),
                                    raw_metadata: "{}".to_string(),
                                };
                                let _ = prod.push(&stream_name, &item).await;
                            }
                        },
                        Err(e) => {
                            eprintln!("[Producer] Error fetching {}: {}", source.url, e);
                            sm.report_failure(&source.url, &e.to_string());
                        }
                    }
                }
            }).await;

            if let Err(e) = sm_clone.save() {
                eprintln!("[Producer] Error saving source health: {}", e);
            }

            println!("[Producer] Completed sweep. Sleeping for 1 hour.");
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    });

    // Task 2: Consumer
    let cons_clone = consumer.clone();
    let cache_clone = cache.clone();
    let weaviate_clone = weaviate.clone();
    let fc_clone = fact_checker.clone();
    let intel_clone = intel_engine.clone();

    tokio::spawn(async move {
        println!("[Consumer] Starting ingestion consumer...");
        loop {
            match cons_clone.consume(stream_name).await {
                Ok(items) => {
                    for (id, item) in items {
                        let cache_key = format!("ingest:{}", item.title);
                        if cache_clone.get(&cache_key).await.ok().flatten().is_some() {
                            let _ = cons_clone.ack(stream_name, &id).await;
                            continue;
                        }

                        println!("[Consumer] Processing Knowledge: {}", item.title);

                        // Intel Engine: Normalization & Extraction
                        let intel_res = match intel_clone.process(&item.title, &item.content).await {
                            Ok(res) => res,
                            Err(e) => {
                                eprintln!("[Consumer] Intel Engine error: {}", e);
                                continue;
                            }
                        };

                        let fc_res = fc_clone.check(&intel_res.normalized_title, &intel_res.normalized_content).await.unwrap_or(fact_checker::FactCheckResult {
                            is_fake_news: false,
                            reliability_score: 0.5,
                        });

                        let summary = Some(intel_res.normalized_content.chars().take(200).collect());
                        let node = KnowledgeNode {
                            title: intel_res.normalized_title,
                            content: intel_res.normalized_content,
                            summary,
                            source_category: item.source_category,
                            source_specifics: item.source_specifics,
                            timestamp: DateTime::parse_from_rfc3339(&item.timestamp).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                            reliability_score: fc_res.reliability_score,
                            is_fake_news: fc_res.is_fake_news,
                            entities: intel_res.entities,
                            location: intel_res.location,
                            language: intel_res.detected_language,
                            version: "1.2".to_string(),
                            raw_metadata: item.raw_metadata,
                        };

                        match weaviate_clone.insert_node(&node).await {
                            Ok(_) => {
                                let _ = cache_clone.set(&cache_key, "1", 86400 * 7).await;
                                let _ = cons_clone.ack(stream_name, &id).await;
                            },
                            Err(e) => eprintln!("[Consumer] Weaviate error: {}", e),
                        }
                    }
                },
                Err(e) => {
                    eprintln!("[Consumer] Error reading stream: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // Task 3: Google Producer (Optional)
    if google_adapter.hub_gmail.is_some() {
        println!("[Google] Starting personal context ingestion...");
        let google_clone = google_adapter.clone();
        let prod_clone = producer.clone();
        tokio::spawn(async move {
            loop {
                // Gmail Ingestion
                println!("[Google] Checking for new emails...");
                if let Ok(messages) = google_clone.fetch_messages().await {
                    for msg in messages {
                        let item = QueueItem {
                            title: format!("Gmail: {}", msg.id.unwrap_or_default()),
                            content: "Gmail content placeholder".to_string(),
                            source_category: "Personal".to_string(),
                            source_specifics: "gmail".to_string(),
                            timestamp: Utc::now().to_rfc3339(),
                            raw_metadata: "{}".to_string(),
                        };
                        let _ = prod_clone.push(stream_name, &item).await;
                    }
                }

                // Calendar Ingestion
                println!("[Google] Checking for new calendar events...");
                if let Ok(events) = google_clone.fetch_calendar_events().await {
                    for event in events {
                        let item = QueueItem {
                            title: format!("Calendar: {}", event.summary.unwrap_or_default()),
                            content: event.description.unwrap_or_default(),
                            source_category: "Personal".to_string(),
                            source_specifics: "google_calendar".to_string(),
                            timestamp: event.start.and_then(|s| s.date_time).map(|dt| dt.to_rfc3339()).unwrap_or_else(|| Utc::now().to_rfc3339()),
                            raw_metadata: "{}".to_string(),
                        };
                        let _ = prod_clone.push(stream_name, &item).await;
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            }
        });
    }

    // Task 4: Compactor (Periodic Cleanup)
    let compactor_clone = compactor.clone();
    tokio::spawn(async move {
        loop {
            // Run compaction every 6 hours
            tokio::time::sleep(tokio::time::Duration::from_secs(21600)).await;
            let _ = compactor_clone.run_compaction_cycle().await;
        }
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
