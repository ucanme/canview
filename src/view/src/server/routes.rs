//! HTTP route definitions

use crate::server::SharedState;
use axum::{
    Router,
    routing::get,
};
use std::sync::Arc;

/// Create the API router with all endpoints
pub fn create_router(state: Arc<SharedState>) -> Router {
    Router::new()
        .route("/api/health", get(super::handlers::health))
        .route("/api/libraries", get(super::handlers::list_libraries))
        .route(
            "/api/libraries/{lib_id}/versions/{version_name}/files/{channel_id}",
            get(super::handlers::download_database_file),
        )
        .with_state(state)
}
