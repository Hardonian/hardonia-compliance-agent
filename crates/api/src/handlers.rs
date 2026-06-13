// Health check handler
use axum::{response::Json, routing::get, Router};
use serde_json::json;

pub fn health_routes() -> Router {
    Router::new().route("/health", get(health_check))
}

/// Structured health check response with version and status.
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "hardonia-compliance-agent",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}