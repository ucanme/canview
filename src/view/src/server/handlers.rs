//! HTTP request handlers

use crate::server::SharedState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct TokenQuery {
    pub token: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub app: &'static str,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Validate token from query params
fn validate_token(state: &SharedState, query: &TokenQuery) -> Result<(), Response> {
    match &query.token {
        Some(t) if state.token_manager.validate(t) => Ok(()),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid or missing token".into(),
            }),
        )
            .into_response()),
    }
}

/// GET /api/health — no auth required
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        app: "CANVIEW",
    })
}

/// GET /api/libraries?token=xxx — list all signal libraries
pub async fn list_libraries(
    State(state): State<Arc<SharedState>>,
    Query(query): Query<TokenQuery>,
) -> Result<impl IntoResponse, Response> {
    validate_token(&state, &query)?;

    let libraries = state.libraries.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to read libraries".into(),
            }),
        )
            .into_response()
    })?;

    Ok(Json(libraries.clone()))
}

/// GET /api/libraries/{lib_id}/versions/{version_name}/files/{channel_id}?token=xxx
/// Download a database file for a specific channel in a library version
pub async fn download_database_file(
    State(state): State<Arc<SharedState>>,
    Path((lib_id, version_name, channel_id)): Path<(String, String, u16)>,
    Query(query): Query<TokenQuery>,
) -> Result<impl IntoResponse, Response> {
    validate_token(&state, &query)?;

    let libraries = state.libraries.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to read libraries".into(),
            }),
        )
            .into_response()
    })?;

    // Find library
    let library = libraries.iter().find(|l| l.id == lib_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Library '{}' not found", lib_id),
            }),
        )
            .into_response()
    })?;

    // Find version
    let version = library
        .get_version(&version_name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Version '{}' not found", version_name),
                }),
            )
                .into_response()
        })?;

    // Find channel database config
    let channel_db = version
        .get_channel_database(channel_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Channel {} not found in version '{}'", channel_id, version_name),
                }),
            )
                .into_response()
        })?;

    // Read the database file
    let file_path = std::path::Path::new(&channel_db.database_path);
    let content = std::fs::read(file_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to read file: {}", e),
            }),
        )
            .into_response()
    })?;

    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("database.dbc");

    Ok((
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/octet-stream".to_string(),
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        content,
    ))
}
