// Middleware
use axum::{
    http::{HeaderValue, Request},
    response::Response,
    middleware::Next,
    body::Body,
};
use tower_http::cors::CorsLayer;

/// CORS layer configured for specific allowed origins.
///
/// In development, origins matching `http://localhost:*` and `http://127.0.0.1:*` are allowed.
/// In production, override `COMPLIANCE_ALLOWED_ORIGINS` env var (comma-separated).
pub fn cors_layer() -> CorsLayer {
    let origins = std::env::var("COMPLIANCE_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173,http://127.0.0.1:3000".to_string());

    let allowed_origins: Vec<HeaderValue> = origins
        .split(',')
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() { return None; }
            HeaderValue::from_str(trimmed).ok()
        })
        .collect();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            "GET".parse().unwrap(),
            "POST".parse().unwrap(),
            "PUT".parse().unwrap(),
            "DELETE".parse().unwrap(),
            "PATCH".parse().unwrap(),
            "OPTIONS".parse().unwrap(),
        ])
        .allow_headers([
            "authorization".parse().unwrap(),
            "content-type".parse().unwrap(),
            "x-request-id".parse().unwrap(),
        ])
}

/// Request ID: extracts `X-Request-Id` from the request or generates a UUID v4,
/// then inserts it into the request extensions and echoes it on the response headers.
pub async fn request_id_layer(mut request: Request<Body>, next: Next) -> Response {
    // Extract or generate request ID
    let request_id = {
        let parts = &request.headers().get("x-request-id");
        match parts.and_then(|v| v.to_str().ok()) {
            Some(id) if !id.is_empty() => id.to_string(),
            _ => {
                use uuid::Uuid;
                Uuid::new_v4().to_string()
            }
        }
    };

    // Insert into request extensions for downstream handlers
    request.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;

    // Echo the request ID on the response
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", value);
    }

    response
}

/// Typed wrapper for the request ID, usable in handler extractors.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}