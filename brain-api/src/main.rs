use axum::{
    routing::{get, post, delete, put},
    Json, Router,
    extract::{State, Path},
};
use brain_core::db::WeaviateClient;
use brain_core::source_manager::SourceManager;
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;
use std::sync::Arc;
use serde::Deserialize;

struct AppState {
    weaviate: Arc<WeaviateClient>,
    source_manager: Arc<SourceManager>,
}

#[derive(Deserialize)]
struct AddSource {
    url: String,
    category: String,
}

#[derive(Deserialize)]
struct ToggleSource {
    enabled: bool,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    let weaviate_url = std::env::var("WEAVIATE_URL").unwrap_or_else(|_| "http://weaviate:8080".to_string());
    let weaviate = Arc::new(WeaviateClient::new(&weaviate_url));
    let source_manager = Arc::new(SourceManager::new("sources.json").expect("Failed to load sources.json"));

    let state = Arc::new(AppState {
        weaviate,
        source_manager,
    });

    let app = Router::new()
        .route("/nodes", get(get_nodes))
        .route("/sources", get(get_sources).post(add_source))
        .route("/sources/:url", delete(delete_source).put(toggle_source))
        .route("/status", get(get_status))
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 4000));
    println!("Brain API listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_nodes(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    match state.weaviate.get_recent_nodes(100).await {
        Ok(nodes) => {
            Json(serde_json::json!(nodes))
        },
        Err(e) => {
            eprintln!("Error fetching nodes: {}", e);
            Json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn get_sources(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!(state.source_manager.get_all_sources()))
}

async fn add_source(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddSource>,
) -> Json<serde_json::Value> {
    state.source_manager.add_source(payload.url, payload.category);
    let _ = state.source_manager.save();
    Json(serde_json::json!({"status": "ok"}))
}

async fn delete_source(
    State(state): State<Arc<AppState>>,
    Path(url_encoded): Path<String>,
) -> Json<serde_json::Value> {
    // Note: URL might need decoding if it contains special chars
    let url = urlencoding::decode(&url_encoded).unwrap_or(std::borrow::Cow::Borrowed(&url_encoded)).to_string();
    state.source_manager.delete_source(&url);
    let _ = state.source_manager.save();
    Json(serde_json::json!({"status": "ok"}))
}

async fn toggle_source(
    State(state): State<Arc<AppState>>,
    Path(url_encoded): Path<String>,
    Json(payload): Json<ToggleSource>,
) -> Json<serde_json::Value> {
    let url = urlencoding::decode(&url_encoded).unwrap_or(std::borrow::Cow::Borrowed(&url_encoded)).to_string();
    state.source_manager.toggle_source(&url, payload.enabled);
    let _ = state.source_manager.save();
    Json(serde_json::json!({"status": "ok"}))
}

async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    // Crude health checks
    let weaviate_ok = state.weaviate.get_recent_nodes(1).await.is_ok();
    
    Json(serde_json::json!({
        "status": "online",
        "version": "1.2",
        "systems": {
            "weaviate": if weaviate_ok { "connected" } else { "disconnected" },
            "redis": "connected",
            "ingestion": "active"
        }
    }))
}
