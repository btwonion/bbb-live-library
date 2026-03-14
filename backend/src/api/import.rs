use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;

use crate::bbb::importer;
use crate::error::AppError;
use crate::models::Recording;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ImportUrlRequest {
    pub url: String,
    pub title: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/import/trigger", post(trigger_bbb_import))
        .route("/api/import/url", post(import_from_url))
}

/// Triggers a bulk import of recordings from the BBB API.
async fn trigger_bbb_import(
    State(state): State<AppState>,
) -> Result<Json<importer::ImportResult>, AppError> {
    let result = importer::run_bbb_import(&state.db, &state.config).await?;
    Ok(Json(result))
}

/// Imports a single recording from a user-provided URL.
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
