use axum::Router;
use crate::handlers;
use crate::state::AppState;

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .merge(handlers::create_routes())
        .with_state(state)
}