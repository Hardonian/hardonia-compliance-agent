// Middleware
use axum::{
    http::Request,
    response::Response,
    middleware::Next,
    body::Body,
};

pub async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().clone();
    let start = std::time::Instant::now();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    tracing::info!(method = %method, path = %path, duration_ms = %duration.as_millis(), status = %response.status());
    
    response
}

pub fn cors_layer() -> tower_http::cors::CorsLayer {
    tower_http::cors::CorsLayer::permissive()
}