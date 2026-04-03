use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;

use crate::bbb::{importer, public};
use crate::error::AppError;
use crate::models::Recording;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ImportUrlRequest {
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportPublicBbbRequest {
    pub url: String,
    pub title: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/import/url", post(import_from_url))
        .route("/api/import/bbb-public", post(import_public_bbb))
}

/// Imports a single recording from a user-provided direct video URL.
async fn import_from_url(
    State(state): State<AppState>,
    Json(body): Json<ImportUrlRequest>,
) -> Result<Json<Recording>, AppError> {
    if body.url.is_empty() {
        return Err(AppError::BadRequest("URL is required".to_string()));
    }

    let recording =
        importer::import_from_url(&state.db, &state.config, &body.url, body.title.as_deref())
            .await?;

    Ok(Json(recording))
}

/// Imports a recording from a public BBB server URL.
async fn import_public_bbb(
    State(state): State<AppState>,
    Json(body): Json<ImportPublicBbbRequest>,
) -> Result<Json<Recording>, AppError> {
    if body.url.is_empty() {
        return Err(AppError::BadRequest("URL is required".to_string()));
    }

    let (server_url, record_id) = public::parse_bbb_url(&body.url)
        .map_err(|e| AppError::BadRequest(format!("Invalid BBB URL: {e}")))?;

    let recording = importer::import_public_bbb(
        &state.db,
        &state.config,
        &server_url,
        &record_id,
        body.title.as_deref(),
    )
    .await?;

    Ok(Json(recording))
}
